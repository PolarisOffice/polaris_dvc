//! `gen-owpml` — XSD → generated_owpml.rs codegen.
//!
//! Reads the seven KS X 6101 XSD schemas under `standards/KSX6101_OWPML/`
//! and emits `crates/polaris-rhwpdvc-schema/src/generated_owpml.rs`
//! with the full element / attribute / enum coverage.
//!
//! Run from repo root:
//!
//! ```sh
//! cargo run --manifest-path tools/gen-owpml/Cargo.toml
//! ```
//!
//! ## Status
//!
//! **Stub**. Phase 3 landed a hand-curated bootstrap
//! `generated_owpml.rs` with ~15 elements (enough to demonstrate the
//! engine integration end-to-end). The real codegen — flattening
//! `xs:complexType`, `xs:group`, `xs:attributeGroup`; resolving
//! `xs:extension`/`xs:restriction`; threading `ref="..."` across
//! targetNamespace boundaries — lands here as a follow-up.
//!
//! ## Design notes (for the follow-up implementation)
//!
//! 1. **Element set extraction**: walk each XSD looking for every
//!    `<xs:element name="...">` declaration and every nested element
//!    reference. Build a global name → declaration map.
//! 2. **Type resolution**: XSD allows `<xs:element ref="other"/>` —
//!    resolve via the cross-file `targetNamespace`. KS X 6101 uses
//!    seven coordinated namespaces (core / head / paragraph / master-
//!    page / section / history / version); `ref` prefixes resolve
//!    unambiguously across them.
//! 3. **Attribute groups**: flatten `<xs:attributeGroup>` inline where
//!    it's referenced. Avoid emitting group names into the generated
//!    file — callers don't see them.
//! 4. **Enums**: any `<xs:simpleType>` with a single `<xs:restriction
//!    base="xs:string">` + `<xs:enumeration>` children maps to
//!    `SimpleType::Enum(&[...])`. Preserve order for stable codegen.
//! 5. **Primitive types**: `xs:int`, `xs:integer`, `xs:unsignedInt` →
//!    `SimpleType::Integer`/`UnsignedInteger`. `xs:boolean` → Boolean.
//!    `xs:decimal`/`xs:double` → Decimal. `xs:string` / `xs:NCName` /
//!    `xs:IDREF` → String/Reference. Unknown → `SimpleType::Unknown`.
//! 6. **Output stability**: emit elements alphabetically by local name
//!    so minor schema edits produce minimal diffs. Include a leading
//!    comment with the source-XSD filenames and their SHA-256 hashes
//!    so we can tell when the generated file is out of date.
//!
//! The file it produces must satisfy:
//!
//! - `pub static HEAD_MODEL: SchemaModel`
//! - `pub static SECTION_MODEL: SchemaModel`
//! - `pub static CONTENT_HPF_MODEL: SchemaModel`
//! - `pub static SETTINGS_MODEL: SchemaModel`
//! - `pub static VERSION_MODEL: SchemaModel`
//!
//! All types come from `polaris_rhwpdvc_schema::model`; the generator
//! emits only data, not helpers.

use std::path::PathBuf;

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
        eprintln!("the committed generated_owpml.rs stays in place; nothing to do.");
        return 0.into();
    }

    eprintln!(
        "gen-owpml: stub — full codegen not yet implemented.\n\
         Detected XSD source:  {}\n\
         Target output file:   {}\n\
         \n\
         Phase 3 delivered a hand-curated bootstrap generated_owpml.rs\n\
         with ~15 elements (enough for engine integration end-to-end).\n\
         Full XSD walk-and-emit lands as a follow-up; see the module\n\
         docs in this file for the design sketch.",
        xsd_dir.display(),
        out_path.display()
    );
    0.into()
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
