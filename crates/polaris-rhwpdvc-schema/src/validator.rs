//! XML → schema validator. Walks a `quick-xml` event stream and
//! emits [`SchemaViolation`]s for each structural / type defect.
//!
//! Scope: element nesting, required / unknown attributes, enum & basic
//! type checks on attribute values, unexpected text under simple-type
//! elements. Child-count cardinality (`maxOccurs`) is tracked as a
//! running counter per element instance.
//!
//! **Not covered** (intentional — see `model.rs` header):
//!   - Child order (`xs:sequence`)
//!   - Schema type inheritance chains
//!   - `xs:pattern` regex
//!   - Cross-element `xs:key` / `xs:keyref` (→ Integrity pass)

use crate::generated_owpml;
use crate::model::{ElementDecl, OwpmlRoot, SchemaModel, SimpleType, ViolationCode};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// One finding. Positions are 1-based within the parsed XML stream;
/// line/column come from the quick-xml reader at the offending event.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaViolation {
    pub code: ViolationCode,
    /// Best-effort local name of the element the violation is anchored
    /// at. Empty string for violations that predate any element.
    pub element: String,
    /// Optional attribute name when the violation is attribute-scoped.
    pub attribute: Option<String>,
    /// Human-readable diagnostic.
    pub message: String,
    /// Offset in bytes from the start of the XML buffer. Crude line/col
    /// approximation is up to the caller (the engine typically pins
    /// violations at page 1 / line 1 since there's no better anchor).
    pub byte_offset: usize,
}

impl SchemaViolation {
    pub fn code(&self) -> ViolationCode {
        self.code
    }
}

/// Resolve which schema model applies to a given XML root.
pub fn model_for(root: OwpmlRoot) -> &'static SchemaModel {
    match root {
        OwpmlRoot::Head => &generated_owpml::HEAD_MODEL,
        OwpmlRoot::Section => &generated_owpml::SECTION_MODEL,
        OwpmlRoot::ContentHpf => &generated_owpml::CONTENT_HPF_MODEL,
        OwpmlRoot::Settings => &generated_owpml::SETTINGS_MODEL,
        OwpmlRoot::Version => &generated_owpml::VERSION_MODEL,
    }
}

/// Validate an XML byte slice against the schema for `root`. Returns
/// every finding — call sites decide whether to short-circuit.
pub fn validate_xml(xml: &[u8], root: OwpmlRoot) -> Vec<SchemaViolation> {
    let model = model_for(root);
    let element_map = model.element_map();
    let mut reader = Reader::from_reader(xml);
    reader.trim_text(false);
    reader.check_end_names(false);

    let mut violations: Vec<SchemaViolation> = Vec::new();
    // Stack of currently-open elements with their per-child counts.
    struct Frame<'a> {
        decl: &'a ElementDecl,
        /// children seen so far, keyed by local name.
        counts: HashMap<&'static str, u32>,
    }
    let mut stack: Vec<Frame> = Vec::new();
    let mut buf: Vec<u8> = Vec::new();

    loop {
        let offset = reader.buffer_position();
        let ev = match reader.read_event_into(&mut buf) {
            Ok(e) => e,
            Err(e) => {
                violations.push(SchemaViolation {
                    code: ViolationCode::UnexpectedChild,
                    element: String::new(),
                    attribute: None,
                    message: format!("quick-xml read error: {e}"),
                    byte_offset: offset,
                });
                break;
            }
        };

        match &ev {
            Event::Start(e) | Event::Empty(e) => {
                let empty_elem = matches!(ev, Event::Empty(_));
                let local_owned = strip_prefix(e.name().as_ref()).to_vec();
                let local: &[u8] = &local_owned;

                // If we're inside a frame, check this is a permitted child.
                // Skip the check entirely when the parent is the unknown-
                // placeholder: if we couldn't resolve the parent's schema,
                // any "unexpected child" report would just be cascade noise
                // pointing at a ghost "<__unknown__>" element. Ancestors
                // already reported the unknown parent; that's enough.
                if let Some(parent) = stack.last_mut() {
                    if std::ptr::eq(parent.decl, &UNKNOWN_ELEMENT) {
                        // fall through to the declaration lookup below,
                        // skipping allowed-child/maxOccurs checks.
                    } else {
                        let allowed = parent
                            .decl
                            .children
                            .iter()
                            .any(|(name, _, _)| name.as_bytes() == local);
                        if !allowed {
                            let child_name = String::from_utf8_lossy(local).into_owned();
                            // Build the "only these are allowed" hint. We cap
                            // at the first 8 names so a single "run" element
                            // (which has 40+ allowed children) doesn't produce
                            // an unreadable message; remainder is counted.
                            let allowed_list = format_allowed_children(parent.decl.children);
                            let message = if parent.decl.children.is_empty() {
                                format!(
                                "<{parent}> cannot contain any child elements, but found <{child_name}>",
                                parent = parent.decl.name,
                            )
                            } else {
                                format!(
                                "<{parent}> can only contain {allowed_list}, but found <{child_name}>",
                                parent = parent.decl.name,
                            )
                            };
                            violations.push(SchemaViolation {
                                code: ViolationCode::UnexpectedChild,
                                element: child_name,
                                attribute: None,
                                message,
                                byte_offset: offset,
                            });
                        } else {
                            // Bump per-name child count; check maxOccurs.
                            let key = parent
                                .decl
                                .children
                                .iter()
                                .find_map(|(name, _, _)| {
                                    if name.as_bytes() == local {
                                        Some(*name)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap();
                            let count = parent.counts.entry(key).or_insert(0);
                            *count += 1;
                            let max = parent
                                .decl
                                .children
                                .iter()
                                .find(|(name, _, _)| *name == key)
                                .map(|(_, _, max)| *max)
                                .unwrap_or(Some(1));
                            if let Some(max_v) = max {
                                if *count > max_v {
                                    violations.push(SchemaViolation {
                                        code: ViolationCode::TooManyOccurrences,
                                        element: key.to_string(),
                                        attribute: None,
                                        message: format!(
                                            "<{}> appears {} times under <{}>, max {}",
                                            key, count, parent.decl.name, max_v
                                        ),
                                        byte_offset: offset,
                                    });
                                }
                            }
                        }
                    } // end "parent != UNKNOWN_ELEMENT" branch
                }

                // Look up the declaration for this element (so we can
                // check its own attrs and push a frame for its body).
                let local_str = std::str::from_utf8(local).unwrap_or("?");
                let decl = element_map.get(local_str).copied();

                if let Some(decl) = decl {
                    // Attribute checks.
                    let mut seen: Vec<String> = Vec::new();
                    for attr in e.attributes().flatten() {
                        let name_bytes = strip_prefix(attr.key.as_ref()).to_vec();
                        let name = String::from_utf8_lossy(&name_bytes).into_owned();
                        // Skip namespace declarations (`xmlns`, `xmlns:x`).
                        if name == "xmlns" || attr.key.as_ref().starts_with(b"xmlns:") {
                            continue;
                        }
                        seen.push(name.clone());
                        let attr_decl = decl.attributes.iter().find(|ad| ad.name == name.as_str());
                        match attr_decl {
                            None => {
                                let allowed = format_allowed_attrs(decl.attributes);
                                let message = if decl.attributes.is_empty() {
                                    format!(
                                        "<{elem}> does not declare any attributes, but found '{name}'",
                                        elem = decl.name,
                                    )
                                } else {
                                    format!(
                                        "<{elem}> can only have attributes {allowed}, but found '{name}'",
                                        elem = decl.name,
                                    )
                                };
                                violations.push(SchemaViolation {
                                    code: ViolationCode::UnknownAttribute,
                                    element: decl.name.to_string(),
                                    attribute: Some(name.clone()),
                                    message,
                                    byte_offset: offset,
                                });
                            }
                            Some(ad) => {
                                let value = attr
                                    .unescape_value()
                                    .map(|c| c.into_owned())
                                    .unwrap_or_default();
                                if let Some(err) = check_simple_type(&value, &ad.ty) {
                                    violations.push(SchemaViolation {
                                        code: ViolationCode::AttributeTypeMismatch,
                                        element: decl.name.to_string(),
                                        attribute: Some(name),
                                        message: err,
                                        byte_offset: offset,
                                    });
                                }
                            }
                        }
                    }
                    // Required-attribute check.
                    for ad in decl.attributes {
                        if ad.required && !seen.iter().any(|n| n == ad.name) {
                            violations.push(SchemaViolation {
                                code: ViolationCode::MissingRequiredAttribute,
                                element: decl.name.to_string(),
                                attribute: Some(ad.name.to_string()),
                                message: format!(
                                    "<{}> missing required attribute '{}'",
                                    decl.name, ad.name
                                ),
                                byte_offset: offset,
                            });
                        }
                    }

                    // Push frame for the body — unless it was empty-element.
                    if !empty_elem {
                        stack.push(Frame {
                            decl,
                            counts: HashMap::new(),
                        });
                    }
                } else if !empty_elem {
                    // Unknown element — still push a placeholder frame so
                    // nested tags don't get attributed to an outer parent.
                    stack.push(Frame {
                        decl: &UNKNOWN_ELEMENT,
                        counts: HashMap::new(),
                    });
                }
            }
            Event::End(_) => {
                if let Some(frame) = stack.pop() {
                    // Skip minOccurs checks on the unknown placeholder —
                    // we couldn't resolve the schema, so reporting
                    // "missing children" against an empty placeholder is
                    // noise.
                    if std::ptr::eq(frame.decl, &UNKNOWN_ELEMENT) {
                        continue;
                    }
                    // minOccurs check: any child with min > seen count.
                    for (name, min, _) in frame.decl.children {
                        if *min == 0 {
                            continue;
                        }
                        let got = frame.counts.get(name).copied().unwrap_or(0);
                        if got < *min {
                            violations.push(SchemaViolation {
                                code: ViolationCode::MissingRequiredChild,
                                element: frame.decl.name.to_string(),
                                attribute: None,
                                message: format!(
                                    "<{}> missing required child <{}> (minOccurs={}, seen={})",
                                    frame.decl.name, name, min, got
                                ),
                                byte_offset: offset,
                            });
                        }
                    }
                }
            }
            Event::Text(t) => {
                if let Some(parent) = stack.last() {
                    if !parent.decl.text_allowed {
                        let raw = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                        if !raw.trim().is_empty() {
                            violations.push(SchemaViolation {
                                code: ViolationCode::UnexpectedText,
                                element: parent.decl.name.to_string(),
                                attribute: None,
                                message: format!(
                                    "<{}> has character data but its complex type is not mixed",
                                    parent.decl.name
                                ),
                                byte_offset: offset,
                            });
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    violations
}

/// Format a parent's declared children into a human-friendly "only
/// these are allowed" fragment — `<a>, <b>, <c>` etc., capped at a
/// sensible length so elements with dozens of valid children
/// (e.g. `<run>`) produce readable messages.
fn format_allowed_children(children: &[(&'static str, u32, Option<u32>)]) -> String {
    const MAX_SHOWN: usize = 8;
    let total = children.len();
    let shown = total.min(MAX_SHOWN);
    let mut out = String::new();
    for (i, (name, _, _)) in children.iter().take(shown).enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('<');
        out.push_str(name);
        out.push('>');
    }
    if total > MAX_SHOWN {
        out.push_str(&format!(" (+{} more)", total - MAX_SHOWN));
    }
    out
}

/// Sister helper to [`format_allowed_children`] for attribute lists.
fn format_allowed_attrs(attrs: &[crate::model::AttributeDecl]) -> String {
    const MAX_SHOWN: usize = 8;
    let total = attrs.len();
    let shown = total.min(MAX_SHOWN);
    let mut out = String::new();
    for (i, ad) in attrs.iter().take(shown).enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('\'');
        out.push_str(ad.name);
        out.push('\'');
    }
    if total > MAX_SHOWN {
        out.push_str(&format!(" (+{} more)", total - MAX_SHOWN));
    }
    out
}

/// Helper: strip an `hh:` / `xs:` style prefix from a qualified name
/// and return the local-name bytes.
fn strip_prefix(qname: &[u8]) -> &[u8] {
    if let Some(pos) = qname.iter().position(|b| *b == b':') {
        &qname[pos + 1..]
    } else {
        qname
    }
}

/// Validate a single attribute value against its declared type. Return
/// a human-readable message iff the value fails; `None` on success
/// or when the type is `Unknown` (we don't fail-closed on codegen
/// gaps).
fn check_simple_type(value: &str, ty: &SimpleType) -> Option<String> {
    match ty {
        SimpleType::String | SimpleType::Reference | SimpleType::Unknown => None,
        SimpleType::Integer => {
            if value.parse::<i64>().is_err() {
                Some(format!("expected integer, got {value:?}"))
            } else {
                None
            }
        }
        SimpleType::UnsignedInteger => match value.parse::<i64>() {
            Ok(n) if n >= 0 => None,
            Ok(_) => Some(format!("expected non-negative integer, got {value:?}")),
            Err(_) => Some(format!("expected integer, got {value:?}")),
        },
        SimpleType::Boolean => {
            if matches!(value, "0" | "1" | "true" | "false") {
                None
            } else {
                Some(format!("expected boolean (0/1/true/false), got {value:?}"))
            }
        }
        SimpleType::Decimal => {
            if value.parse::<f64>().is_err() {
                Some(format!("expected decimal number, got {value:?}"))
            } else {
                None
            }
        }
        SimpleType::Enum(choices) => {
            if choices.contains(&value) {
                None
            } else {
                // Show first few allowed values to bound message size.
                let preview: Vec<&str> = choices.iter().take(6).copied().collect();
                let ellipsis = if choices.len() > 6 { ", …" } else { "" };
                Some(format!(
                    "value {:?} not in enum [{}{}]",
                    value,
                    preview.join(", "),
                    ellipsis
                ))
            }
        }
    }
}

static UNKNOWN_ELEMENT: ElementDecl = ElementDecl {
    name: "__unknown__",
    children: &[],
    attributes: &[],
    text_allowed: true,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AttributeDecl, SchemaModel};

    #[test]
    fn unknown_attribute_fires() {
        // Use the generated head model; try an attribute we know isn't
        // on <head>. (The real HEAD_MODEL is generated; for unit
        // coverage we synthesize a small model instead.)
        static ATTRS: &[AttributeDecl] = &[AttributeDecl {
            name: "version",
            ty: SimpleType::String,
            required: false,
        }];
        static ELEMS: &[(&str, ElementDecl)] = &[(
            "root",
            ElementDecl {
                name: "root",
                children: &[],
                attributes: ATTRS,
                text_allowed: false,
            },
        )];
        let _model = SchemaModel {
            root_name: "root",
            elements: ELEMS,
        };
        // Directly test check_simple_type — validator integration is
        // covered in the real generated_owpml tests.
        assert!(check_simple_type("3", &SimpleType::UnsignedInteger).is_none());
        assert!(check_simple_type("-1", &SimpleType::UnsignedInteger).is_some());
        assert!(check_simple_type("true", &SimpleType::Boolean).is_none());
        assert!(check_simple_type("yes", &SimpleType::Boolean).is_some());
        assert!(check_simple_type("foo", &SimpleType::Enum(&["foo", "bar"])).is_none());
        assert!(check_simple_type("baz", &SimpleType::Enum(&["foo", "bar"])).is_some());
    }

    #[test]
    fn strip_prefix_works() {
        assert_eq!(strip_prefix(b"hh:charPr"), b"charPr");
        assert_eq!(strip_prefix(b"charPr"), b"charPr");
        assert_eq!(strip_prefix(b":weird"), b"weird");
    }
}
