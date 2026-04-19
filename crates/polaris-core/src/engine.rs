//! Validation engine. Fleshed out in Phase 4.

use crate::report::Report;
use crate::rules::schema::RuleSpec;

#[derive(Default)]
pub struct EngineOptions {
    pub stop_on_first: bool,
}

pub fn validate(
    _doc: &polaris_hwpx::HwpxDocument,
    _spec: &RuleSpec,
    _opts: &EngineOptions,
) -> Report {
    // Phase 4 will implement category-specific checkers here.
    Report::empty()
}
