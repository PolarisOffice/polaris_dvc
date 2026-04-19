//! WASM bindings — single `validate` entry point.
//!
//! Mirrors the CLI's surface: accept HWPX bytes + a rule spec, return a
//! DVC-shaped JSON array. The return shape is `OutputOption::AllOption`
//! so every conditional field (TableID/IsInTable/UseStyle/IsInShape/
//! UseHyperlink) is present — JS consumers can filter as needed.

use serde::Serialize;
use wasm_bindgen::prelude::*;

use polaris_rhwpdvc_core::engine::{validate as run, EngineOptions};
use polaris_rhwpdvc_core::output::OutputOption;
use polaris_rhwpdvc_core::rules::schema::RuleSpec;

#[wasm_bindgen]
pub fn validate(hwpx: &[u8], spec: JsValue) -> Result<JsValue, JsError> {
    let spec: RuleSpec = if spec.is_undefined() || spec.is_null() {
        RuleSpec::default()
    } else {
        serde_wasm_bindgen::from_value(spec).map_err(|e| JsError::new(&e.to_string()))?
    };

    let doc = match polaris_rhwpdvc_format::parse(hwpx).map_err(|e| JsError::new(&e.to_string()))? {
        polaris_rhwpdvc_format::Document::Hwpx(d) => d,
    };

    let report = run(&doc, &spec, &EngineOptions::default());
    let payload = report.to_json_value(OutputOption::AllOption);
    // Default serde-wasm-bindgen serializer emits serde_json::Value as
    // an externally-tagged enum (`{"Object": {...}}`). The JSON-compatible
    // serializer flattens it to plain JS objects/arrays, which is what
    // browser consumers expect.
    let ser = serde_wasm_bindgen::Serializer::json_compatible();
    payload
        .serialize(&ser)
        .map_err(|e| JsError::new(&e.to_string()))
}
