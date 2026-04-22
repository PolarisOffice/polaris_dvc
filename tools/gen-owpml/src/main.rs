//! `gen-owpml` — KS X 6101 XSD → `generated_owpml.rs` codegen.
//!
//! Walks `standards/KSX6101_OWPML/*.xsd`, extracts every element and
//! type declaration, and emits
//! `crates/polaris-rhwpdvc-schema/src/generated_owpml.rs` as a flat
//! table of [`ElementDecl`] entries that the validator consumes
//! without further interpretation.
//!
//! Running (from repo root):
//!
//! ```sh
//! cargo run --manifest-path tools/gen-owpml/Cargo.toml
//! ```
//!
//! ## Pragmatic XSD subset
//!
//! XSD is a big language. We cover the subset that KS X 6101 actually
//! uses:
//!
//! - `<xs:element name="X" type="Y"/>` — simple type reference
//! - `<xs:element name="X"><xs:complexType>…</xs:complexType></xs:element>`
//!   — inline type body
//! - `<xs:complexType name="Y">` — named type, referenced via `type=`
//! - `<xs:sequence>`, `<xs:choice>`, `<xs:all>` — treated as unordered
//!   "allowed children" sets (element order not validated — matches
//!   the validator's ordering-agnostic design)
//! - `<xs:complexContent><xs:extension base="Z">…</xs:extension>` —
//!   merge Z's children+attrs with the extension's additions
//! - `<xs:simpleContent><xs:extension base="…">` — treated as mixed
//!   (text_allowed=true)
//! - `<xs:attribute name="a" type="t" use="required|optional"/>` —
//!   attributes with type resolution
//! - `<xs:attributeGroup name="G">` / `<xs:attributeGroup ref="G"/>` —
//!   expand references inline
//! - `<xs:simpleType>` with `<xs:restriction base="xs:string">
//!   <xs:enumeration value="X"/></xs:restriction>` — enum extraction
//! - `<xs:restriction base="xs:*">` where base is a primitive — map to
//!   SimpleType::{Integer, UnsignedInteger, Boolean, Decimal, String}
//!
//! **Skipped** (documented but not critical for the 80 % target):
//!
//! - `<xs:group>` — not used in KS X 6101
//! - `<xs:key>` / `<xs:keyref>` / `<xs:unique>` — handled by the
//!   Integrity pass structurally, not via XSD
//! - `<xs:pattern>` regex — emit the field as `SimpleType::String`
//! - Union types, list types — emit as `SimpleType::Unknown`
//! - Abstract types / substitution groups — flattened conservatively
//!   (the concrete subtypes appear as their own elements)
//!
//! ## Output organization
//!
//! One `SchemaModel` per root: HEAD (`head`), SECTION (`sec`),
//! CONTENT_HPF (OPF package, retained from the bootstrap — not in the
//! KSX XSDs), SETTINGS (`HWPApplicationSetting`), VERSION
//! (`HCFVersion`). Each model's `elements` array contains every
//! element reachable from that root, flattened. Elements that belong
//! to multiple roots (core primitives like `PointType`, `ColorValue`)
//! are emitted in every model that references them — small
//! duplication, but keeps lookup cheap and the models independent.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use roxmltree::{Document, Node};

#[derive(Debug, Clone, Default)]
struct ComplexTypeBody {
    /// Name of the base type when inheriting via `<xs:extension>` (or
    /// `<xs:restriction>` of complex content). Resolved during
    /// flattening.
    base: Option<String>,
    /// Direct child elements declared under this type.
    children: Vec<ChildDecl>,
    /// Direct attributes declared on this type.
    attributes: Vec<AttrDecl>,
    /// `<xs:attributeGroup ref="…">` references — resolved during
    /// flattening.
    attribute_group_refs: Vec<String>,
    /// Whether this type is simple-content or mixed.
    text_allowed: bool,
}

#[derive(Debug, Clone)]
struct ChildDecl {
    name: String,
    /// Named type reference (with xs-prefix stripped and possibly
    /// carrying a cross-schema namespace prefix like `hc:`). Resolved
    /// to a ComplexTypeBody during flattening.
    type_ref: Option<String>,
    /// When true, the element had an inline `<xs:complexType>` /
    /// `<xs:simpleType>`; we eagerly registered that under a synthetic
    /// type name stored here.
    inline_type_key: Option<String>,
    min_occurs: u32,
    /// `None` represents `unbounded`.
    max_occurs: Option<u32>,
}

#[derive(Debug, Clone)]
struct AttrDecl {
    name: String,
    type_ref: Option<String>,
    /// Inline `<xs:simpleType>` content; key into simple_types.
    inline_type_key: Option<String>,
    required: bool,
}

#[derive(Debug, Clone)]
enum SimpleTypeBody {
    Enum(Vec<String>),
    Primitive(Primitive),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Primitive {
    String,
    Integer,
    UnsignedInteger,
    Boolean,
    Decimal,
    Reference,
}

#[derive(Default, Debug)]
struct SchemaRegistry {
    complex_types: BTreeMap<String, ComplexTypeBody>,
    simple_types: BTreeMap<String, SimpleTypeBody>,
    /// Attribute groups (name → attrs + nested group refs).
    attribute_groups: BTreeMap<String, (Vec<AttrDecl>, Vec<String>)>,
    /// Top-level elements keyed by local name. Each value is a type
    /// reference name into `complex_types` (may be a synthetic name
    /// for inline bodies).
    top_elements: BTreeMap<String, String>,
    /// Monotonic counter for synthetic type names (inline complex/
    /// simple types).
    synth_counter: u32,
}

fn main() -> std::process::ExitCode {
    let root = match find_repo_root() {
        Some(r) => r,
        None => {
            eprintln!("gen-owpml: couldn't locate repo root");
            return 1.into();
        }
    };
    let xsd_dir = root.join("standards").join("KSX6101_OWPML");
    let out_path = root
        .join("crates")
        .join("polaris-rhwpdvc-schema")
        .join("src")
        .join("generated_owpml.rs");

    if !xsd_dir.exists() {
        eprintln!(
            "gen-owpml: standards/KSX6101_OWPML/ not found at {}",
            xsd_dir.display()
        );
        eprintln!("see standards/README.md for how to obtain the source materials.");
        return 1.into();
    }

    let mut reg = SchemaRegistry::default();
    let xsd_paths: Vec<PathBuf> = fs::read_dir(&xsd_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "xsd")
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();

    let mut xsd_names = Vec::new();
    for p in &xsd_paths {
        xsd_names.push(p.file_name().unwrap().to_string_lossy().into_owned());
        if let Err(e) = ingest_xsd(p, &mut reg) {
            eprintln!("gen-owpml: {}: {e}", p.display());
            return 1.into();
        }
    }
    xsd_names.sort();

    eprintln!(
        "gen-owpml: parsed {} XSDs → {} elements, {} complex types, {} simple types, {} attr groups",
        xsd_paths.len(),
        reg.top_elements.len(),
        reg.complex_types.len(),
        reg.simple_types.len(),
        reg.attribute_groups.len(),
    );

    let rust = emit_rust(&reg, &xsd_names);
    // Pipe through rustfmt so the committed output is canonical-
    // formatted — otherwise `cargo fmt --check` in CI fails on every
    // regeneration. `rustfmt` ships with every stable toolchain; if
    // the subprocess fails for any reason we fall back to the raw
    // emission and log a warning so the developer can run `cargo
    // fmt` manually.
    let final_src = match rustfmt(&rust) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "gen-owpml: rustfmt failed ({e}); writing unformatted output. \
                 Run `cargo fmt -p polaris-rhwpdvc-schema` after this.",
            );
            rust
        }
    };
    fs::write(&out_path, final_src).expect("write generated_owpml.rs");
    eprintln!(
        "gen-owpml: wrote {} ({} bytes)",
        out_path.display(),
        fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0)
    );
    0.into()
}

/// Run `rustfmt --edition=2021` on a source string and return the
/// formatted result. Shells out because `prettyplease` / `syn`
/// don't handle all the syntactic forms we emit.
fn rustfmt(src: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("rustfmt")
        .arg("--edition=2021")
        .arg("--emit=stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn rustfmt: {e}"))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "rustfmt stdin unavailable".to_string())?
        .write_all(src.as_bytes())
        .map_err(|e| format!("write to rustfmt: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("wait rustfmt: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "rustfmt exit {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("rustfmt non-utf8 output: {e}"))
}

fn find_repo_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("crates").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

// ─── Ingest (XSD → registry) ─────────────────────────────────────────

fn ingest_xsd(path: &Path, reg: &mut SchemaRegistry) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc = Document::parse(&text).map_err(|e| e.to_string())?;
    let root = doc.root_element();

    for child in root.children().filter(Node::is_element) {
        match local_name(child) {
            "element" => {
                if let Some(name) = attr(child, "name") {
                    let key = register_element_body(child, reg);
                    reg.top_elements.insert(name.to_string(), key);
                }
            }
            "complexType" => {
                if let Some(name) = attr(child, "name") {
                    let body = parse_complex_type(child, reg);
                    reg.complex_types.insert(name.to_string(), body);
                }
            }
            "simpleType" => {
                if let Some(name) = attr(child, "name") {
                    let body = parse_simple_type(child);
                    reg.simple_types.insert(name.to_string(), body);
                }
            }
            "attributeGroup" => {
                if let Some(name) = attr(child, "name") {
                    let (attrs, refs) = parse_attr_group_body(child, reg);
                    reg.attribute_groups.insert(name.to_string(), (attrs, refs));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Register a complex/simple type body for an element (named or inline)
/// and return the key under which it's stored in the registry.
fn register_element_body(elem: Node, reg: &mut SchemaRegistry) -> String {
    // Case 1: `type="…"` attribute — just refer to the named type.
    if let Some(t) = attr(elem, "type") {
        return strip_prefix(t).to_string();
    }
    // Case 2: inline `<xs:complexType>` child.
    for c in elem.children().filter(Node::is_element) {
        match local_name(c) {
            "complexType" => {
                reg.synth_counter += 1;
                let key = format!(
                    "__inline_ct_{}_{}",
                    attr(elem, "name").unwrap_or("anon"),
                    reg.synth_counter
                );
                let body = parse_complex_type(c, reg);
                reg.complex_types.insert(key.clone(), body);
                return key;
            }
            "simpleType" => {
                reg.synth_counter += 1;
                let key = format!(
                    "__inline_st_{}_{}",
                    attr(elem, "name").unwrap_or("anon"),
                    reg.synth_counter
                );
                let body = parse_simple_type(c);
                reg.simple_types.insert(key.clone(), body);
                return key;
            }
            _ => {}
        }
    }
    // Case 3: neither — treat as open (string) placeholder.
    format!("__open_{}", attr(elem, "name").unwrap_or("anon"))
}

fn parse_complex_type(ct: Node, reg: &mut SchemaRegistry) -> ComplexTypeBody {
    let mut body = ComplexTypeBody::default();
    for c in ct.children().filter(Node::is_element) {
        match local_name(c) {
            "sequence" | "all" => {
                collect_children(c, &mut body.children, reg, false);
            }
            "choice" => {
                // xs:choice = "one of these"; we don't model ordering,
                // so the correct conservative flattening is to make
                // every branch optional (min=0). The containing
                // sequence's own min/max already decides whether any
                // choice branch is required at all.
                collect_children(c, &mut body.children, reg, true);
            }
            "attribute" => {
                if let Some(a) = parse_attribute(c, reg) {
                    body.attributes.push(a);
                }
            }
            "attributeGroup" => {
                if let Some(r) = attr(c,"ref") {
                    body.attribute_group_refs.push(strip_prefix(r).to_string());
                }
            }
            "simpleContent" => {
                body.text_allowed = true;
                // Also inherit base + attrs from extension/restriction
                for cc in c.children().filter(Node::is_element) {
                    if matches!(local_name(cc), "extension" | "restriction") {
                        if let Some(base) = attr(cc,"base") {
                            body.base = Some(strip_prefix(base).to_string());
                        }
                        for ccc in cc.children().filter(Node::is_element) {
                            match local_name(ccc) {
                                "attribute" => {
                                    if let Some(a) = parse_attribute(ccc, reg) {
                                        body.attributes.push(a);
                                    }
                                }
                                "attributeGroup" => {
                                    if let Some(r) = attr(ccc,"ref") {
                                        body.attribute_group_refs
                                            .push(strip_prefix(r).to_string());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            "complexContent" => {
                for cc in c.children().filter(Node::is_element) {
                    if matches!(local_name(cc), "extension" | "restriction") {
                        if let Some(base) = attr(cc,"base") {
                            body.base = Some(strip_prefix(base).to_string());
                        }
                        for ccc in cc.children().filter(Node::is_element) {
                            match local_name(ccc) {
                                "sequence" | "all" => {
                                    collect_children(ccc, &mut body.children, reg, false);
                                }
                                "choice" => {
                                    collect_children(ccc, &mut body.children, reg, true);
                                }
                                "attribute" => {
                                    if let Some(a) = parse_attribute(ccc, reg) {
                                        body.attributes.push(a);
                                    }
                                }
                                "attributeGroup" => {
                                    if let Some(r) = attr(ccc,"ref") {
                                        body.attribute_group_refs
                                            .push(strip_prefix(r).to_string());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    // Mixed content marker on the element itself.
    if attr(ct, "mixed") == Some("true") {
        body.text_allowed = true;
    }
    body
}

fn collect_children(
    seq: Node,
    out: &mut Vec<ChildDecl>,
    reg: &mut SchemaRegistry,
    inside_choice: bool,
) {
    for c in seq.children().filter(Node::is_element) {
        match local_name(c) {
            "element" => {
                if let Some(name) = attr(c,"name") {
                    let type_ref = attr(c,"type").map(|t| strip_prefix(t).to_string());
                    // If no `type=` attr, check for inline type.
                    let inline_type_key = if type_ref.is_none() {
                        let key = register_element_body(c, reg);
                        if key.starts_with("__inline_") {
                            Some(key)
                        } else if key.starts_with("__open_") {
                            None
                        } else {
                            // Register returned a named type — treat as type_ref.
                            None
                        }
                    } else {
                        None
                    };
                    // When type_ref was missing AND register returned a
                    // non-inline key (shouldn't happen often), keep the
                    // synthetic key as the type_ref.
                    let final_type_ref = type_ref.or_else(|| {
                        if inline_type_key.is_some() {
                            inline_type_key.clone()
                        } else {
                            None
                        }
                    });
                    let raw_min = parse_occurs(attr(c,"minOccurs"), 1);
                    // Inside an xs:choice, every branch is effectively
                    // optional — exactly one satisfies the choice,
                    // which we can't express per-child. Force min=0.
                    let min_occurs = if inside_choice { 0 } else { raw_min };
                    out.push(ChildDecl {
                        name: name.to_string(),
                        type_ref: final_type_ref,
                        inline_type_key: None,
                        min_occurs,
                        max_occurs: parse_max_occurs(attr(c,"maxOccurs")),
                    });
                } else if let Some(r) = attr(c,"ref") {
                    // Ref to another element — key by local name. We
                    // don't have the element's type, so just record the
                    // local name with unknown cardinality semantics.
                    let raw_min = parse_occurs(attr(c,"minOccurs"), 1);
                    let min_occurs = if inside_choice { 0 } else { raw_min };
                    out.push(ChildDecl {
                        name: strip_prefix(r).to_string(),
                        type_ref: None,
                        inline_type_key: None,
                        min_occurs,
                        max_occurs: parse_max_occurs(attr(c,"maxOccurs")),
                    });
                }
            }
            // Nested group modifiers — flatten.
            "sequence" | "all" => collect_children(c, out, reg, inside_choice),
            "choice" => collect_children(c, out, reg, true),
            _ => {}
        }
    }
}

fn parse_attribute(attr_node: Node, reg: &mut SchemaRegistry) -> Option<AttrDecl> {
    let name = attr(attr_node, "name")?.to_string();
    let type_ref = attr(attr_node, "type").map(|t| strip_prefix(t).to_string());
    let required = attr(attr_node, "use") == Some("required");

    // Inline `<xs:simpleType>` — register it.
    let inline_type_key = if type_ref.is_none() {
        let mut key = None;
        for c in attr_node.children().filter(Node::is_element) {
            if local_name(c) == "simpleType" {
                reg.synth_counter += 1;
                let synth = format!("__inline_attr_st_{}_{}", name, reg.synth_counter);
                let body = parse_simple_type(c);
                reg.simple_types.insert(synth.clone(), body);
                key = Some(synth);
            }
        }
        key
    } else {
        None
    };

    Some(AttrDecl {
        name,
        type_ref,
        inline_type_key,
        required,
    })
}

fn parse_attr_group_body(
    node: Node,
    reg: &mut SchemaRegistry,
) -> (Vec<AttrDecl>, Vec<String>) {
    let mut attrs = Vec::new();
    let mut refs = Vec::new();
    for c in node.children().filter(Node::is_element) {
        match local_name(c) {
            "attribute" => {
                if let Some(a) = parse_attribute(c, reg) {
                    attrs.push(a);
                }
            }
            "attributeGroup" => {
                if let Some(r) = attr(c,"ref") {
                    refs.push(strip_prefix(r).to_string());
                }
            }
            _ => {}
        }
    }
    (attrs, refs)
}

fn parse_simple_type(st: Node) -> SimpleTypeBody {
    for c in st.children().filter(Node::is_element) {
        if local_name(c) == "restriction" {
            let base = attr(c,"base").map(strip_prefix).unwrap_or("xs:string");
            let enums: Vec<String> = c
                .children()
                .filter(Node::is_element)
                .filter(|n| local_name(*n) == "enumeration")
                .filter_map(|n| attr(n, "value").map(str::to_string))
                .collect();
            if !enums.is_empty() {
                return SimpleTypeBody::Enum(enums);
            }
            return SimpleTypeBody::Primitive(map_primitive(base));
        }
        if matches!(local_name(c), "union" | "list") {
            return SimpleTypeBody::Unknown;
        }
    }
    SimpleTypeBody::Unknown
}

fn map_primitive(base: &str) -> Primitive {
    match base {
        "xs:integer" | "xs:int" | "xs:long" | "xs:short" => Primitive::Integer,
        "xs:positiveInteger" | "xs:nonNegativeInteger" | "xs:unsignedInt"
        | "xs:unsignedLong" | "xs:unsignedShort" | "xs:unsignedByte" => {
            Primitive::UnsignedInteger
        }
        "xs:boolean" => Primitive::Boolean,
        "xs:decimal" | "xs:double" | "xs:float" => Primitive::Decimal,
        "xs:IDREF" | "xs:IDREFS" | "xs:ID" | "xs:NCName" | "xs:Name" | "xs:token" => {
            Primitive::Reference
        }
        _ => Primitive::String,
    }
}

// ─── Flattening ──────────────────────────────────────────────────────

/// Flattened element declaration ready for emission. Every inline
/// type has been resolved; every extension chain has been merged;
/// every attributeGroup ref has been expanded.
#[derive(Debug)]
struct FlatElement {
    #[allow(dead_code)]
    name: String,
    children: Vec<(String, String, u32, Option<u32>)>, // (local_name, type_ref, min, max)
    attributes: Vec<FlatAttr>,
    text_allowed: bool,
}

#[derive(Debug)]
struct FlatAttr {
    name: String,
    ty: FlatType,
    required: bool,
}

#[derive(Debug, Clone)]
enum FlatType {
    String,
    Integer,
    UnsignedInteger,
    Boolean,
    Decimal,
    Reference,
    Enum(Vec<String>),
    Unknown,
}

/// Flatten a complex-type body to its direct (children, attributes,
/// text) by chasing extensions + attribute groups. `visited` guards
/// against circular inheritance (shouldn't happen in OWPML but cheap
/// insurance).
fn flatten_complex(
    key: &str,
    reg: &SchemaRegistry,
    visited: &mut BTreeSet<String>,
) -> ComplexTypeBody {
    if !visited.insert(key.to_string()) {
        return ComplexTypeBody::default();
    }
    let Some(ct) = reg.complex_types.get(key) else {
        return ComplexTypeBody::default();
    };
    let mut merged = ct.clone();

    // Absorb base type (extension / restriction) first.
    if let Some(base) = &ct.base {
        let base_flat = flatten_complex(base, reg, visited);
        // Base children come first, then this type's (xs:extension
        // semantics). Attributes similarly.
        let mut base_children = base_flat.children;
        base_children.extend(merged.children.drain(..));
        merged.children = base_children;
        let mut base_attrs = base_flat.attributes;
        base_attrs.extend(merged.attributes.drain(..));
        merged.attributes = base_attrs;
        if base_flat.text_allowed {
            merged.text_allowed = true;
        }
    }

    // Expand attributeGroup refs (one level; groups can themselves
    // reference groups, handled by recursion).
    let ag_refs: Vec<String> = merged.attribute_group_refs.drain(..).collect();
    for g in ag_refs {
        expand_attr_group(&g, reg, &mut merged.attributes);
    }

    merged
}

fn expand_attr_group(name: &str, reg: &SchemaRegistry, out: &mut Vec<AttrDecl>) {
    let Some((attrs, refs)) = reg.attribute_groups.get(name) else {
        return;
    };
    out.extend(attrs.iter().cloned());
    for r in refs {
        expand_attr_group(r, reg, out);
    }
}

fn resolve_attr_type(ad: &AttrDecl, reg: &SchemaRegistry) -> FlatType {
    if let Some(key) = &ad.inline_type_key {
        return simple_to_flat(reg.simple_types.get(key));
    }
    if let Some(tref) = &ad.type_ref {
        // Primitive?
        let mapped = map_primitive(&format!("xs:{}", tref.trim_start_matches("xs:")));
        if tref.starts_with("xs:") || primitive_only(&mapped) {
            return primitive_to_flat(mapped);
        }
        if let Some(st) = reg.simple_types.get(tref) {
            return simple_to_flat(Some(st));
        }
    }
    FlatType::String
}

fn primitive_only(p: &Primitive) -> bool {
    !matches!(p, Primitive::String | Primitive::Reference)
}

fn simple_to_flat(body: Option<&SimpleTypeBody>) -> FlatType {
    match body {
        Some(SimpleTypeBody::Enum(v)) => FlatType::Enum(v.clone()),
        Some(SimpleTypeBody::Primitive(p)) => primitive_to_flat(*p),
        Some(SimpleTypeBody::Unknown) | None => FlatType::Unknown,
    }
}

fn primitive_to_flat(p: Primitive) -> FlatType {
    match p {
        Primitive::String => FlatType::String,
        Primitive::Integer => FlatType::Integer,
        Primitive::UnsignedInteger => FlatType::UnsignedInteger,
        Primitive::Boolean => FlatType::Boolean,
        Primitive::Decimal => FlatType::Decimal,
        Primitive::Reference => FlatType::Reference,
    }
}

/// Build the type-keyed FlatElement table reachable from a root type.
/// Returns `(root_type_key, element_map)` where `element_map` is keyed
/// by type key (named complexType or synthetic `__inline_…`).
///
/// ## Context-sensitive resolution
///
/// XSD lets the same local name be declared with different bodies in
/// different parents. KS X 6101 hits this in several places — most
/// notably `<offset>`, declared three times:
///
///   - under `<fontface>` with attrs `hangul`, `latin`, …
///   - under `<borderFill>` with attrs `left`, `right`, `top`, `bottom`
///   - under shape types with attrs `x`, `y`
///
/// Rather than collapse by local name (lossy) or merge the union
/// (lossy in the other direction — masks cross-context contamination),
/// we key on **type** and carry a `type_ref` alongside each child
/// name. The validator consults the parent's children list to pick
/// the correct decl for the child it's currently processing, so each
/// `<offset>` context gets its own true allowed-attribute set.
///
/// Cycle guard: we visit each type_key exactly once. Multiple
/// elements can share a type (OPF items all share `ItemType` etc.),
/// and that's fine — we just don't re-flatten the body.
fn build_flat_model(
    root_local_name: &str,
    reg: &SchemaRegistry,
) -> Option<(String, String, BTreeMap<String, FlatElement>)> {
    let root_type_key = reg.top_elements.get(root_local_name)?.clone();
    let mut out: BTreeMap<String, FlatElement> = BTreeMap::new();
    // Work queue: (type_key_to_flatten, element_local_name_for_name_field)
    let mut queue: Vec<(String, String)> =
        vec![(root_type_key.clone(), root_local_name.to_string())];
    let mut seen: BTreeSet<String> = BTreeSet::new();

    while let Some((type_key, elem_name)) = queue.pop() {
        if !seen.insert(type_key.clone()) {
            continue;
        }
        let body = flatten_complex(&type_key, reg, &mut BTreeSet::new());

        let new_children: Vec<(String, String, u32, Option<u32>)> = body
            .children
            .iter()
            .map(|c| {
                let type_ref = c
                    .type_ref
                    .clone()
                    .or_else(|| c.inline_type_key.clone())
                    .unwrap_or_default(); // empty string = "unknown type"
                (c.name.clone(), type_ref, c.min_occurs, c.max_occurs)
            })
            .collect();

        let new_attrs: Vec<FlatAttr> = body
            .attributes
            .iter()
            .map(|a| FlatAttr {
                name: a.name.clone(),
                ty: resolve_attr_type(a, reg),
                required: a.required,
            })
            .collect();

        out.insert(
            type_key.clone(),
            FlatElement {
                name: elem_name,
                children: new_children,
                attributes: new_attrs,
                text_allowed: body.text_allowed,
            },
        );

        // Enqueue each child's type for flattening. The same type
        // referenced multiple times from different parents is fine —
        // the `seen` guard collapses the repeat visits.
        for c in &body.children {
            let child_type = c
                .type_ref
                .clone()
                .or_else(|| c.inline_type_key.clone())
                .unwrap_or_default();
            if !child_type.is_empty() {
                queue.push((child_type, c.name.clone()));
            }
        }
    }

    Some((root_local_name.to_string(), root_type_key, out))
}

// ─── Emit (registry → generated_owpml.rs) ────────────────────────────

fn emit_rust(reg: &SchemaRegistry, xsd_names: &[String]) -> String {
    let roots = [
        ("head", "HEAD_MODEL", "HEAD_ELEMENTS"),
        ("sec", "SECTION_MODEL", "SECTION_ELEMENTS"),
        ("HWPApplicationSetting", "SETTINGS_MODEL", "SETTINGS_ELEMENTS"),
        ("HCFVersion", "VERSION_MODEL", "VERSION_ELEMENTS"),
    ];

    let mut s = String::new();
    s.push_str(&format!(
        "//! `generated_owpml.rs` — automatically generated by `tools/gen-owpml/`.\n\
         //!\n\
         //! Source XSDs: {}\n\
         //!\n\
         //! Do not edit by hand. Re-run the generator after any standards\n\
         //! update:\n\
         //!\n\
         //! ```sh\n\
         //! cargo run --manifest-path tools/gen-owpml/Cargo.toml\n\
         //! ```\n\
         //!\n\
         //! This file is derivative factual data from KS X 6101 (element\n\
         //! names, attribute names, enum values, cardinality bounds). The\n\
         //! standard's documentation text is NOT copied — only structural\n\
         //! facts. See `standards/README.md` for licensing notes.\n\n",
        xsd_names.join(", ")
    ));
    s.push_str("#![allow(clippy::all)]\n");
    s.push_str("#![allow(dead_code)]\n");
    s.push_str("#![allow(unused_imports)]\n\n");
    s.push_str("use crate::model::{AttributeDecl, ElementDecl, SchemaModel, SimpleType};\n\n");

    // CONTENT_HPF_MODEL is retained from the bootstrap — OPF package
    // is not in the KSX XSDs. We emit a minimal block for it below.
    s.push_str(&emit_content_hpf());

    for (root_name, model_const, array_const) in roots {
        match build_flat_model(root_name, reg) {
            Some((_, root_type, map)) if !map.is_empty() => {
                s.push_str(&emit_model(
                    root_name,
                    &root_type,
                    model_const,
                    array_const,
                    &map,
                ));
            }
            _ => {
                // Placeholder — root not found. Fall back to a minimal
                // single-element model so downstream code still links.
                let fallback_type = format!("__missing_{root_name}");
                s.push_str(&format!(
                    "// root element {root_name:?} not found in source XSDs — empty placeholder.\n"
                ));
                s.push_str(&format!(
                    "static {array_const}: &[(&str, ElementDecl)] = &[(\n\
                     \"{fallback_type}\",\n\
                     ElementDecl {{\n\
                     \tname: \"{root_name}\",\n\
                     \tchildren: &[],\n\
                     \tattributes: &[],\n\
                     \ttext_allowed: true,\n\
                     }},\n\
                     )];\n\
                     pub static {model_const}: SchemaModel = SchemaModel {{\n\
                     \troot_name: \"{root_name}\",\n\
                     \troot_type: \"{fallback_type}\",\n\
                     \telements: {array_const},\n\
                     }};\n\n",
                ));
            }
        }
    }

    s
}

fn emit_model(
    root_name: &str,
    root_type: &str,
    model_const: &str,
    array_const: &str,
    map: &BTreeMap<String, FlatElement>,
) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "// ──────────────────────────────────────────────────────────────────\n\
         // {model_const} — root element <{root_name}> (auto-generated)\n\
         // ──────────────────────────────────────────────────────────────────\n\n"
    ));
    s.push_str(&format!(
        "static {array_const}: &[(&str, ElementDecl)] = &[\n"
    ));
    for (type_key, elem) in map {
        s.push_str(&format!(
            "    (\"{esc_key}\", ElementDecl {{\n\
             \t\tname: \"{esc_name}\",\n\
             \t\tchildren: &[{children}],\n\
             \t\tattributes: &[{attrs}],\n\
             \t\ttext_allowed: {text},\n\
             \t}}),\n",
            esc_key = escape_str(type_key),
            esc_name = escape_str(&elem.name),
            children = emit_children(&elem.children),
            attrs = emit_attrs(&elem.attributes),
            text = elem.text_allowed,
        ));
    }
    s.push_str("];\n\n");
    s.push_str(&format!(
        "pub static {model_const}: SchemaModel = SchemaModel {{\n\
         \troot_name: \"{root_name}\",\n\
         \troot_type: \"{root_type}\",\n\
         \telements: {array_const},\n\
         }};\n\n",
        root_type = escape_str(root_type)
    ));
    s
}

fn emit_children(children: &[(String, String, u32, Option<u32>)]) -> String {
    if children.is_empty() {
        return String::new();
    }
    let mut s = String::from("\n");
    for (name, type_ref, min, max) in children {
        let max_str = match max {
            None => "None".to_string(),
            Some(n) => format!("Some({n})"),
        };
        s.push_str(&format!(
            "\t\t\t(\"{}\", \"{}\", {}, {}),\n",
            escape_str(name),
            escape_str(type_ref),
            min,
            max_str
        ));
    }
    s.push_str("\t\t");
    s
}

fn emit_attrs(attrs: &[FlatAttr]) -> String {
    if attrs.is_empty() {
        return String::new();
    }
    let mut s = String::from("\n");
    for a in attrs {
        let ty = emit_type(&a.ty);
        s.push_str(&format!(
            "\t\t\tAttributeDecl {{ name: \"{}\", ty: {}, required: {} }},\n",
            escape_str(&a.name),
            ty,
            a.required
        ));
    }
    s.push_str("\t\t");
    s
}

fn emit_type(ty: &FlatType) -> String {
    match ty {
        FlatType::String => "SimpleType::String".to_string(),
        FlatType::Integer => "SimpleType::Integer".to_string(),
        FlatType::UnsignedInteger => "SimpleType::UnsignedInteger".to_string(),
        FlatType::Boolean => "SimpleType::Boolean".to_string(),
        FlatType::Decimal => "SimpleType::Decimal".to_string(),
        FlatType::Reference => "SimpleType::Reference".to_string(),
        FlatType::Unknown => "SimpleType::Unknown".to_string(),
        FlatType::Enum(v) => {
            let parts: Vec<String> = v
                .iter()
                .map(|s| format!("\"{}\"", escape_str(s)))
                .collect();
            format!("SimpleType::Enum(&[{}])", parts.join(", "))
        }
    }
}

fn emit_content_hpf() -> String {
    // Retained from the bootstrap — OPF manifest isn't part of
    // KS X 6101 proper but we still validate it. Hand-coded entries
    // kept here to avoid splitting the generated file's coverage.
    //
    // Covers OPF 2.0 package shape: `<metadata>` with DC-namespace
    // children + `<meta>` extension tags, `<manifest>` with `<item>`,
    // `<spine>` with `<itemref>`. We match by local name, so the
    // `dc:` prefix on metadata children is stripped away.
    // Type keys mirror local names here — OPF 2.0 doesn't reuse the
    // same element name in different contexts, so the trivial 1:1
    // mapping is fine. Children entries are (name, type_ref, min, max).
    r##"// ──────────────────────────────────────────────────────────────────
// CONTENT_HPF_MODEL — Contents/content.hpf (OPF package)
// Hand-maintained block; OPF isn't part of KS X 6101.
// ──────────────────────────────────────────────────────────────────

static CONTENT_HPF_ELEMENTS: &[(&str, ElementDecl)] = &[
    ("package", ElementDecl {
        name: "package",
        children: &[
            ("metadata", "metadata", 0, Some(1)),
            ("manifest", "manifest", 1, Some(1)),
            ("spine", "spine", 0, Some(1)),
            ("guide", "guide", 0, Some(1)),
        ],
        attributes: &[
            AttributeDecl { name: "version", ty: SimpleType::String, required: false },
            AttributeDecl { name: "unique-identifier", ty: SimpleType::String, required: false },
            AttributeDecl { name: "id", ty: SimpleType::String, required: false },
        ],
        text_allowed: false,
    }),
    // OPF <metadata> block with Dublin Core children. dc: prefix is
    // stripped during validation so we match on local names only.
    ("metadata", ElementDecl {
        name: "metadata",
        children: &[
            ("title",       "title",       0, None),
            ("creator",     "creator",     0, None),
            ("subject",     "subject",     0, None),
            ("description", "description", 0, None),
            ("publisher",   "publisher",   0, None),
            ("contributor", "contributor", 0, None),
            ("date",        "date",        0, None),
            ("type",        "type",        0, None),
            ("format",      "format",      0, None),
            ("identifier",  "identifier",  0, None),
            ("source",      "source",      0, None),
            ("language",    "language",    0, None),
            ("relation",    "relation",    0, None),
            ("coverage",    "coverage",    0, None),
            ("rights",      "rights",      0, None),
            ("meta",        "meta",        0, None),
        ],
        attributes: &[],
        text_allowed: true,
    }),
    // DC elements — all text-only content with no declared attribute
    // set (OPF lets vendors attach arbitrary xml:lang / id, which we
    // don't want to flag as "unknown attribute").
    ("title",        ElementDecl { name: "title",        children: &[], attributes: &[], text_allowed: true }),
    ("creator",      ElementDecl { name: "creator",      children: &[], attributes: &[], text_allowed: true }),
    ("subject",      ElementDecl { name: "subject",      children: &[], attributes: &[], text_allowed: true }),
    ("description",  ElementDecl { name: "description",  children: &[], attributes: &[], text_allowed: true }),
    ("publisher",    ElementDecl { name: "publisher",    children: &[], attributes: &[], text_allowed: true }),
    ("contributor", ElementDecl { name: "contributor",  children: &[], attributes: &[], text_allowed: true }),
    ("date",         ElementDecl { name: "date",         children: &[], attributes: &[], text_allowed: true }),
    ("type",         ElementDecl { name: "type",         children: &[], attributes: &[], text_allowed: true }),
    ("format",       ElementDecl { name: "format",       children: &[], attributes: &[], text_allowed: true }),
    ("identifier",   ElementDecl { name: "identifier",   children: &[], attributes: &[], text_allowed: true }),
    ("source",       ElementDecl { name: "source",       children: &[], attributes: &[], text_allowed: true }),
    ("language",     ElementDecl { name: "language",     children: &[], attributes: &[], text_allowed: true }),
    ("relation",     ElementDecl { name: "relation",     children: &[], attributes: &[], text_allowed: true }),
    ("coverage",     ElementDecl { name: "coverage",     children: &[], attributes: &[], text_allowed: true }),
    ("rights",       ElementDecl { name: "rights",       children: &[], attributes: &[], text_allowed: true }),
    ("meta", ElementDecl {
        name: "meta",
        children: &[],
        attributes: &[
            AttributeDecl { name: "name",     ty: SimpleType::String, required: false },
            AttributeDecl { name: "content",  ty: SimpleType::String, required: false },
            AttributeDecl { name: "property", ty: SimpleType::String, required: false },
            AttributeDecl { name: "scheme",   ty: SimpleType::String, required: false },
        ],
        text_allowed: true,
    }),
    ("manifest", ElementDecl {
        name: "manifest",
        children: &[("item", "item", 1, None)],
        attributes: &[],
        text_allowed: false,
    }),
    // `<opf:item>` — strict OPF 2.0 core attributes only (id, href,
    // media-type). We deliberately don't declare Hancom extensions
    // (`isEmbedded`, …) so the validator reports them as divergences
    // from the standard — that's the whole point of this axis.
    ("item", ElementDecl {
        name: "item",
        children: &[],
        attributes: &[
            AttributeDecl { name: "id",    ty: SimpleType::String, required: true },
            AttributeDecl { name: "href",  ty: SimpleType::String, required: true },
            AttributeDecl { name: "media-type", ty: SimpleType::String, required: false },
        ],
        text_allowed: false,
    }),
    ("spine", ElementDecl {
        name: "spine",
        children: &[("itemref", "itemref", 0, None)],
        attributes: &[
            AttributeDecl { name: "toc", ty: SimpleType::String, required: false },
        ],
        text_allowed: false,
    }),
    ("itemref", ElementDecl {
        name: "itemref",
        children: &[],
        attributes: &[
            AttributeDecl { name: "idref",  ty: SimpleType::String, required: true },
            AttributeDecl { name: "linear", ty: SimpleType::String, required: false },
        ],
        text_allowed: false,
    }),
    ("guide", ElementDecl {
        name: "guide",
        children: &[("reference", "reference", 0, None)],
        attributes: &[],
        text_allowed: false,
    }),
    ("reference", ElementDecl {
        name: "reference",
        children: &[],
        attributes: &[
            AttributeDecl { name: "type",  ty: SimpleType::String, required: false },
            AttributeDecl { name: "title", ty: SimpleType::String, required: false },
            AttributeDecl { name: "href",  ty: SimpleType::String, required: false },
        ],
        text_allowed: false,
    }),
];

pub static CONTENT_HPF_MODEL: SchemaModel = SchemaModel {
    root_name: "package",
    root_type: "package",
    elements: CONTENT_HPF_ELEMENTS,
};

"##
    .to_string()
}

// ─── Small helpers ───────────────────────────────────────────────────

fn local_name<'a, 'input>(n: Node<'a, 'input>) -> &'input str {
    n.tag_name().name()
}

fn attr<'a, 'input>(n: Node<'a, 'input>, name: &str) -> Option<&'a str>
where
    'input: 'a,
{
    n.attribute(name)
}

fn strip_prefix(qname: &str) -> &str {
    match qname.find(':') {
        Some(idx) => &qname[idx + 1..],
        None => qname,
    }
}

fn parse_occurs(v: Option<&str>, default: u32) -> u32 {
    v.and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn parse_max_occurs(v: Option<&str>) -> Option<u32> {
    match v {
        None => Some(1),
        Some("unbounded") => None,
        Some(s) => s.parse().ok(),
    }
}

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
