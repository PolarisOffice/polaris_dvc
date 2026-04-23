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

/// One element declaration. Maps to one `<xs:complexType>` (named or
/// inline). `children` holds the allowed child elements, each paired
/// with the `type_ref` the child's own declaration is stored under.
/// That type_ref is what gives us **context-sensitive** validation:
/// when the same local name is declared differently in different
/// parents (e.g. `<offset>` has `{left,right,top,bottom}` under
/// `<borderFill>` but `{x,y}` under a shape), the child list of each
/// parent carries the type_ref pointing to the right declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementDecl {
    /// Local name this element appears as in XML. Purely diagnostic —
    /// does NOT drive lookup; `type_ref` does.
    pub name: &'static str,
    /// Allowed children: `(local_name, type_ref, min_occurs,
    /// max_occurs)`. `type_ref` is the key into
    /// [`SchemaModel::elements`] for that child's declaration; an
    /// empty string means "unknown type" (open to anything) and the
    /// validator will treat the child as a placeholder frame without
    /// pushing further constraints. Child **order** is not validated
    /// — XSD `xs:choice`/`xs:sequence` distinctions are collapsed.
    pub children: &'static [(&'static str, &'static str, u32, Option<u32>)],
    pub attributes: &'static [AttributeDecl],
    /// Whether the element may contain character data directly (mixed
    /// content or simple content).
    pub text_allowed: bool,
}

/// A bundle of element declarations keyed by **type key** (named
/// complexType name, or a synthetic `__inline_…` key for anonymous
/// types). Each parent element's `children` list carries the type_ref
/// to follow at lookup time; this is how we keep context-sensitive
/// validation without giving up a flat lookup table.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaModel {
    /// XML local name of the document root (e.g. `"head"`, `"sec"`).
    /// Used to shape diagnostic messages and to match the opening tag.
    pub root_name: &'static str,
    /// Type key under which the root element's declaration is stored
    /// in `elements`. Validator seeds the frame stack with this.
    pub root_type: &'static str,
    /// All element declarations reachable from the root, keyed by
    /// type key. Multiple entries may share the same local `name`
    /// when the XSD declares the same element name in different
    /// contexts — that's exactly the property we keep.
    pub elements: &'static [(&'static str, ElementDecl)],
}

impl SchemaModel {
    /// Build a one-shot lookup map. Callers that validate many
    /// documents should cache this; the validator builds it per call
    /// for simplicity.
    pub fn element_map(&self) -> BTreeMap<&'static str, &ElementDecl> {
        self.elements.iter().map(|(k, v)| (*k, v)).collect()
    }

    /// Fetch the root element's declaration.
    pub fn root_decl(&self) -> Option<&ElementDecl> {
        self.elements
            .iter()
            .find_map(|(k, v)| if *k == self.root_type { Some(v) } else { None })
    }

    /// Fetch an element declaration by type key.
    pub fn lookup(&self, type_key: &str) -> Option<&ElementDecl> {
        self.elements
            .iter()
            .find_map(|(k, v)| if *k == type_key { Some(v) } else { None })
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
            "CharShapeType",
            ElementDecl {
                name: "charPr",
                children: &[("fontRef", "FontRefType", 1, Some(1))],
                attributes: ATTRS,
                text_allowed: false,
            },
        )];
        let m = SchemaModel {
            root_name: "charPr",
            root_type: "CharShapeType",
            elements: ELEMENTS,
        };
        assert!(m.lookup("CharShapeType").is_some());
        assert_eq!(m.lookup("CharShapeType").unwrap().attributes[0].name, "id");
        assert!(m.lookup("nope").is_none());
        assert_eq!(m.root_decl().unwrap().name, "charPr");
    }
}
