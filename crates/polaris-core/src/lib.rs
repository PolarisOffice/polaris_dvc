//! polaris-core: rule engine, error-code registry, and DVC-compatible output model.
//!
//! Upstream reference: hancom-io/dvc (`Source/Checker.*`, `Source/DVCOutputJson.*`,
//! `Source/JsonModel.h`). See `/docs/` and the repo NOTICE file.

pub mod engine;
pub mod error_codes;
pub mod jid_registry;
pub mod output;
pub mod report;
pub mod rules;

pub use error_codes::ErrorCode;
pub use output::ViolationRecord;
pub use report::Report;
