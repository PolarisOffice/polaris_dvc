//! WASM bindings — single `validate` entry point. Phase 6 of the plan.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn validate(hwpx: &[u8], spec: JsValue) -> Result<JsValue, JsError> {
    let spec: polaris_core::rules::schema::RuleSpec = match spec.is_undefined() || spec.is_null() {
        true => polaris_core::rules::schema::RuleSpec::default(),
        false => serde_wasm_bindgen::from_value(spec).map_err(|e| JsError::new(&e.to_string()))?,
    };

    let doc = match polaris_format::parse(hwpx).map_err(|e| JsError::new(&e.to_string()))? {
        polaris_format::Document::Hwpx(d) => d,
    };

    let opts = polaris_core::engine::EngineOptions::default();
    let report = polaris_core::engine::validate(&doc, &spec, &opts);
    serde_wasm_bindgen::to_value(&report).map_err(|e| JsError::new(&e.to_string()))
}
