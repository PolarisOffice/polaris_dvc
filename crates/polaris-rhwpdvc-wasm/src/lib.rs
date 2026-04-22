//! WASM bindings for the polaris HWPX validator.
//!
//! Two entry points mirroring the CLI's JSON/XML outputs:
//!
//! ```js
//! validate(bytes, spec, { dvcStrict, stopOnFirst, outputOption });  // → array
//! validateXml(bytes, spec, opts);                                   // → string
//! ```
//!
//! `opts` is optional; missing fields fall back to defaults (Extended
//! profile, stop on nothing, `OutputOption::AllOption`).

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use polaris_rhwpdvc_core::engine::{validate as run, CheckProfile, EngineOptions};
use polaris_rhwpdvc_core::error_codes::ErrorCode;
use polaris_rhwpdvc_core::output::OutputOption;
use polaris_rhwpdvc_core::rules::schema::RuleSpec;

/// Runtime options accepted by [`validate`] and [`validate_xml`].
/// All fields optional.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ValidateOpts {
    /// Map to `EngineOptions::profile = DvcStrict`. Only emits violations
    /// that upstream `DVC.exe` also checks.
    dvc_strict: bool,
    /// Map to `EngineOptions::stop_on_first`.
    stop_on_first: bool,
    /// Opt into JID 13000 KS X 6101 schema conformance checks. Default
    /// off — bootstrap schema subset fires many findings on elements
    /// it doesn't yet cover.
    enable_schema: bool,
    /// Which conditional fields are emitted on each violation. Accepts
    /// `"default"` / `"table"` / `"tableDetail"` (or `"table-detail"`) /
    /// `"style"` / `"shape"` / `"hyperlink"` / `"all"`. Missing or
    /// unrecognized falls back to `"all"` to preserve the historical
    /// WASM default.
    output_option: Option<String>,
}

fn parse_output_option(s: Option<&str>) -> OutputOption {
    match s.unwrap_or("all").to_ascii_lowercase().as_str() {
        "default" => OutputOption::Default,
        "table" => OutputOption::Table,
        "tabledetail" | "table-detail" | "table_detail" => OutputOption::TableDetail,
        "style" => OutputOption::Style,
        "shape" => OutputOption::Shape,
        "hyperlink" => OutputOption::Hyperlink,
        _ => OutputOption::AllOption,
    }
}

/// Shared setup: decode spec + opts, parse doc, run the engine, return the
/// `Report` alongside the chosen output option so callers can shape the
/// final serialization themselves.
fn prepare(
    hwpx: &[u8],
    spec: JsValue,
    opts: JsValue,
) -> Result<(polaris_rhwpdvc_core::report::Report, OutputOption), JsError> {
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
        enable_schema: runtime_opts.enable_schema,
    };

    let out_opt = parse_output_option(runtime_opts.output_option.as_deref());
    let report = run(&doc, &spec, &engine_opts);
    Ok((report, out_opt))
}

/// DVC-shaped JSON output. Returns an array-valued `JsValue`.
#[wasm_bindgen]
pub fn validate(hwpx: &[u8], spec: JsValue, opts: JsValue) -> Result<JsValue, JsError> {
    let (report, out_opt) = prepare(hwpx, spec, opts)?;
    let payload = report.to_json_value(out_opt);
    // Default serde-wasm-bindgen serializer emits serde_json::Value as
    // an externally-tagged enum (`{"Object": {...}}`). The JSON-compatible
    // serializer flattens it to plain JS objects/arrays, which is what
    // browser consumers expect.
    let ser = serde_wasm_bindgen::Serializer::json_compatible();
    payload
        .serialize(&ser)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Look up the human-readable description for an `ErrorCode` numeric
/// value. Mirrors `ErrorCode::text()` from core — single source of truth
/// for JID → message mapping, so the web UI never drifts from the CLI.
///
/// ```js
/// describeError(11010); // "mimetype is not the first ZIP entry"
/// describeError(1001);  // "Font size does not match specification"
/// describeError(99999); // "Rule violation" (generic fallback)
/// ```
#[wasm_bindgen(js_name = describeError)]
pub fn describe_error(code: u32) -> String {
    ErrorCode::new(code).text().to_string()
}

/// Same as [`validate`] but returns the XML document string. This is a
/// polaris extension — upstream DVC never implemented XML output — so
/// callers using `dvcStrict: true` get an error mirroring the CLI's
/// upstream-parity behavior.
#[wasm_bindgen(js_name = validateXml)]
pub fn validate_xml(hwpx: &[u8], spec: JsValue, opts: JsValue) -> Result<String, JsError> {
    // Peek at the strict flag before running the engine so we fail fast
    // with an upstream-matching message (cheap string compare on opts).
    if let Ok(parsed) = serde_wasm_bindgen::from_value::<ValidateOpts>(opts.clone()) {
        if parsed.dvc_strict {
            return Err(JsError::new(
                "--format=xml is not yet implemented (upstream DVC parity)",
            ));
        }
    }
    let (report, out_opt) = prepare(hwpx, spec, opts)?;
    Ok(report.to_xml_string(out_opt))
}
