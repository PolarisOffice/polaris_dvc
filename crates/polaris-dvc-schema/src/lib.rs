//! polaris-dvc-schema: KS X 6101 (OWPML) XSD model + XML validator.
//!
//! Two layers:
//!
//!   1. [`model`] — Rust data types representing the subset of XSD we
//!      care about (elements, attributes, simple types / enums,
//!      complex-type children). **Namespace-agnostic**: we key
//!      everything by local name since OWPML has released under
//!      multiple namespace URIs (`2011/*`, `2016/*`, `2024/*`) that
//!      share the same structure.
//!
//!   2. [`validator`] — walks an XML document (via `quick-xml`) and
//!      compares each element against the corresponding `ElementDecl`,
//!      emitting typed [`SchemaViolation`]s.
//!
//! The third layer — the actual compiled schema content — lives in
//! [`generated_owpml`], a Rust source file produced by
//! `tools/gen-owpml/` from the standards-document XSDs under
//! `standards/` (see `standards/README.md` for licensing notes).
//! The generated file is committed so end users don't need the
//! source XSDs to build.
//!
//! # Example
//!
//! ```no_run
//! use polaris_dvc_schema::{validate_xml, OwpmlRoot};
//!
//! let xml = br#"<?xml version="1.0"?><hh:head xmlns:hh="h"/>"#;
//! let violations = validate_xml(xml, OwpmlRoot::Head);
//! for v in violations {
//!     eprintln!("{:?}: {}", v.code(), v.message);
//! }
//! ```

pub mod model;
pub mod validator;

// The generated content — element/attribute/enum tables derived from
// KS X 6101 XSDs by tools/gen-owpml/. This file is committed so users
// can build without the copyrighted standards/ materials; regenerate
// only when the source XSDs change.
pub mod generated_owpml;

pub use model::{AttributeDecl, ElementDecl, OwpmlRoot, SchemaModel, SimpleType, ViolationCode};
pub use validator::{validate_xml, SchemaViolation};
