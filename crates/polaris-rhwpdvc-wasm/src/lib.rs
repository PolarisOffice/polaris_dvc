//! WASM bindings — single `validate` entry point.
//!
//! Mirrors the CLI's surface: accept HWPX bytes + a rule spec, return a
//! DVC-shaped JSON array. The return shape is `OutputOption::AllOption`
//! so every conditional field (TableID/IsInTable/UseStyle/IsInShape/
//! UseHyperlink) is present — JS consumers can filter as needed.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use polaris_rhwpdvc_core::engine::{validate as run, CheckProfile, EngineOptions};
use polaris_rhwpdvc_core::output::OutputOption;
use polaris_rhwpdvc_core::rules::schema::RuleSpec;

/// Runtime options accepted by [`validate`]. All fields optional.
///
/// ```js
/// validate(bytes, spec, { dvcStrict: true, stopOnFirst: false });
/// ```
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ValidateOpts {
    /// Map to `EngineOptions::profile = DvcStrict`. Only emits violations
    /// that upstream `DVC.exe` also checks.
    dvc_strict: bool,
    /// Map to `EngineOptions::stop_on_first`.
    stop_on_first: bool,
}

#[wasm_bindgen]
pub fn validate(hwpx: &[u8], spec: JsValue, opts: JsValue) -> Result<JsValue, JsError> {
    let spec: RuleSpec = if spec.is_undefined() || spec.is_null() {
        RuleSpec::default()
    } else {
        serde_wasm_bindgen::from_value(spec).map_err(|e| JsError::new(&e.to_string()))?
    };

    let runtime_opts: ValidateOpts = if opts.is_undefined() || opts.is_null() {
        ValidateOpts::default()
    } else {
        serde_wasm_bindgen::from_value(opts).map_err(|e| JsError::new(&e.to_string()))?
    };

    let doc = match polaris_rhwpdvc_format::parse(hwpx).map_err(|e| JsError::new(&e.to_string()))? {
        polaris_rhwpdvc_format::Document::Hwpx(d) => d,
    };

    let engine_opts = EngineOptions {
        stop_on_first: runtime_opts.stop_on_first,
        profile: if runtime_opts.dvc_strict {
            CheckProfile::DvcStrict
        } else {
            CheckProfile::Extended
        },
    };

    let report = run(&doc, &spec, &engine_opts);
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
