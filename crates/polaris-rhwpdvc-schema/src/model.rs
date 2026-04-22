//! Schema model — the Rust-native shape of an OWPML XSD subset.
//!
//! Design constraint: **cover 80 % of real-world XML defects with < 5 %
//! of XSD's complexity**. We intentionally skip:
//!
//!   - `xs:group`, `xs:attributeGroup` (flattened during codegen)
//!   - `xs:unique`, `xs:key`, `xs:keyref` (cross-ref is integrity's job)
//!   - `xs:extension` / `xs:restriction` derivation trees (flattened)
//!   - `xs:pattern` regex (emitted as informational only)
//!   - `xs:choice`, `xs:sequence`, `xs:all` ordering (treated as
//!     an unordered set of allowed children, with min/max cardinality)
//!
//! This is a pragmatic well-formedness checker, not a full XSD
//! validator. The 20 % of edge cases we miss are primarily about
//! ordering and schema inheritance, which downstream Phase-4 invariant
//! checks can cover structurally where they matter.
//!
//! Namespace handling: **local names only**. Every key in
//! [`SchemaModel::elements`] is the element's local name (`charPr`, not
//! `hh:charPr`). The validator strips prefixes before lookup, so a
//! document using `xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head"`
//! and one using the 2024 namespace both resolve to the same
//! `ElementDecl`.

use std::collections::BTreeMap;

/// Which top-level OWPML document a validator pass targets. Each entry
/// corresponds to one of the XML files inside an HWPX ZIP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwpmlRoot {
    /// `Contents/header.xml` — `<hh:head>` root. Font tables, char/
    /// paragraph property tables, styles, border fills, numbering.
    Head,
    /// `Contents/section*.xml` — `<hs:sec>` root. Body paragraphs,
    /// tables, runs.
    Section,
    /// `Contents/content.hpf` — OPF package (manifest + spine).
    /// Validated via a small bundled schema (not from the KSX XSDs).
    ContentHpf,
    /// `settings.xml` — `<ha:HWPApplicationSetting>` root. Application-
    /// level preferences.
    Settings,
    /// `version.xml` — `<ha:HCFVersion>` root. File-format version.
    Version,
}

impl OwpmlRoot {
    pub fn local_name(self) -> &'static str {
        match self {
            Self::Head => "head",
            Self::Section => "sec",
            Self::ContentHpf => "package",
            Self::Settings => "HWPApplicationSetting",
            Self::Version => "HCFVersion",
        }
    }
}

/// One XSD `<xs:simpleType>` — covers enum and basic type restrictions.
/// Complex derived types are flattened to their base during codegen.
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleType {
    /// Arbitrary string — no constraint.
    String,
    /// Integer, 32-bit signed unless otherwise stated.
    Integer,
    /// Non-negative integer / XML Schema `xs:unsignedInt`-ish.
    UnsignedInteger,
    /// Boolean — XSD `xs:boolean`. Accepts `true`/`false`/`0`/`1`.
    Boolean,
    /// Floating-point numeric (XSD `xs:double`, `xs:decimal`).
    Decimal,
    /// Enumeration — one of a fixed string set. Comparisons are
    /// case-sensitive per XSD. Slice (not Vec) so values can live in
    /// static storage for the generated schema.
    Enum(&'static [&'static str]),
    /// IDRef / IDRefs / NCName etc. — treat as opaque string for now;
    /// cross-ref resolution is the Integrity pass's job, not Schema's.
    Reference,
    /// Fallback for types our codegen couldn't resolve (rare — usually
    /// means a union or complex restriction). Not flagged as violation
    /// when the value type is uncertain; we emit a schema-violation
    /// only when a clear constraint is broken.
    Unknown,
}

/// One attribute declaration on an element. Maps to `<xs:attribute
/// name="…" type="…" use="required|optional"/>`.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeDecl {
    pub name: &'static str,
    pub ty: SimpleType,
    pub required: bool,
}

/// One element declaration. Maps to `<xs:element name="…"
/// type="ComplexType">` after flattening. `children` holds the local
/// names the element is allowed to contain (with cardinality);
/// `attributes` holds the declared attrs; `text_allowed` reflects
/// whether the complex type has `mixed="true"` or is simple-content.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementDecl {
    pub name: &'static str,
    /// Map of `local_name -> (min_occurs, max_occurs)`. `max = None`
    /// represents `unbounded`. Child order is NOT validated — XSD's
    /// `xs:choice`/`xs:sequence` distinction is collapsed.
    pub children: &'static [(&'static str, u32, Option<u32>)],
    pub attributes: &'static [AttributeDecl],
    /// Whether the element may contain character data directly (mixed
    /// content or simple content).
    pub text_allowed: bool,
}

/// A bundle of element declarations keyed by local name, plus the root
/// element name. One `SchemaModel` per [`OwpmlRoot`].
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaModel {
    pub root_name: &'static str,
    /// Every element reachable from `root_name`, keyed by local name.
    /// Flat map because XSD nested types are flattened during codegen.
    pub elements: &'static [(&'static str, ElementDecl)],
}

impl SchemaModel {
    /// Build a one-shot lookup map. Callers that validate many
    /// documents should cache this; the validator builds it per call
    /// for simplicity.
    pub fn element_map(&self) -> BTreeMap<&'static str, &ElementDecl> {
        self.elements.iter().map(|(k, v)| (*k, v)).collect()
    }

    pub fn lookup(&self, local_name: &str) -> Option<&ElementDecl> {
        self.elements.iter().find_map(|(name, decl)| {
            if *name == local_name {
                Some(decl)
            } else {
                None
            }
        })
    }
}

/// Fine-grained violation type. The validator tags every finding with
/// one of these; the engine maps each tag to a JID when surfacing to
/// the `ViolationRecord` stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationCode {
    /// An element appears under a parent where the schema doesn't
    /// declare it (`<charPr>` inside `<p>`).
    UnexpectedChild,
    /// A required child is absent. One emit per missing element.
    MissingRequiredChild,
    /// A child appears more times than `maxOccurs` allows.
    TooManyOccurrences,
    /// An attribute marked `use="required"` is absent.
    MissingRequiredAttribute,
    /// An attribute name isn't declared on this element.
    UnknownAttribute,
    /// Attribute value doesn't match its declared simple-type
    /// constraint (enum out of range, integer isn't an integer, etc.).
    AttributeTypeMismatch,
    /// Element contains character data but the schema says no
    /// text is allowed.
    UnexpectedText,
}

impl ViolationCode {
    /// Short diagnostic label, used by the engine when building
    /// violation messages.
    pub fn label(self) -> &'static str {
        match self {
            Self::UnexpectedChild => "unexpected child element",
            Self::MissingRequiredChild => "missing required child",
            Self::TooManyOccurrences => "child count exceeds schema maxOccurs",
            Self::MissingRequiredAttribute => "missing required attribute",
            Self::UnknownAttribute => "unknown attribute on element",
            Self::AttributeTypeMismatch => "attribute value doesn't match its declared type",
            Self::UnexpectedText => "element contains text where schema allows none",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_model_lookup() {
        static ATTRS: &[AttributeDecl] = &[AttributeDecl {
            name: "id",
            ty: SimpleType::UnsignedInteger,
            required: true,
        }];
        static ELEMENTS: &[(&str, ElementDecl)] = &[(
            "charPr",
            ElementDecl {
                name: "charPr",
                children: &[("fontRef", 1, Some(1))],
                attributes: ATTRS,
                text_allowed: false,
            },
        )];
        let m = SchemaModel {
            root_name: "charPr",
            elements: ELEMENTS,
        };
        assert!(m.lookup("charPr").is_some());
        assert_eq!(m.lookup("charPr").unwrap().attributes[0].name, "id");
        assert!(m.lookup("nope").is_none());
    }
}
