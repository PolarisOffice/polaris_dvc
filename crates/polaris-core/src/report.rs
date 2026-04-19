//! Top-level validation report returned by the engine and CLI.

use serde::{Deserialize, Serialize};

use crate::output::ViolationRecord;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Report {
    pub source: Option<String>,
    pub spec: Option<String>,
    pub violations: Vec<ViolationRecord>,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Summary {
    pub total: u32,
    pub stopped_early: bool,
}

impl Report {
    pub fn empty() -> Self {
        Self {
            source: None,
            spec: None,
            violations: Vec::new(),
            summary: Summary::default(),
        }
    }

    pub fn push(&mut self, v: ViolationRecord) {
        self.violations.push(v);
        self.summary.total = self.violations.len() as u32;
    }
}
