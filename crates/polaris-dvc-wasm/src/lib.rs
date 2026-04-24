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

use polaris_dvc_core::engine::{validate as run, CheckProfile, EngineOptions};
use polaris_dvc_core::error_codes::ErrorCode;
use polaris_dvc_core::output::OutputOption;
use polaris_dvc_core::rules::schema::RuleSpec;

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
) -> Result<(polaris_dvc_core::report::Report, OutputOption), JsError> {
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

    // `Document` currently has a single `Hwpx` variant (HWP5 is a
    // reserved slot that `polaris_dvc_format::sniff` routes through
    // but `parse` doesn't produce yet). A `let` pattern is infallible
    // today; if/when the enum grows, this line will stop compiling
    // and force every downstream site to handle the new variant
    // explicitly — which is exactly what we want.
    let polaris_dvc_format::Document::Hwpx(doc) =
        polaris_dvc_format::parse(hwpx).map_err(|e| JsError::new(&e.to_string()))?;

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

/// Enumerate every ZIP entry in an HWPX file, without running the
/// validator. Returns an array of `{ path, size, compression,
/// isDirectory }` objects in the order they appear in the ZIP
/// central directory — the web demo feeds this into its file-tree
/// explorer. Works even when the document fails full OWPML parsing,
/// so "the file is broken, but let me look inside it" still works.
#[wasm_bindgen(js_name = listZipEntries)]
pub fn list_zip_entries(hwpx: &[u8]) -> Result<JsValue, JsError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Entry {
        path: String,
        size: u64,
        compression: &'static str,
        is_directory: bool,
    }
    let entries =
        polaris_dvc_hwpx::list_zip_entries(hwpx).map_err(|e| JsError::new(&e.to_string()))?;
    let mapped: Vec<Entry> = entries
        .into_iter()
        .map(|e| Entry {
            path: e.path,
            size: e.size,
            compression: e.compression,
            is_directory: e.is_directory,
        })
        .collect();
    let ser = serde_wasm_bindgen::Serializer::json_compatible();
    mapped
        .serialize(&ser)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Read one ZIP entry's raw bytes by path. Returns a `Uint8Array`.
/// The JS caller decides whether to decode as text (UTF-8 for XML /
/// HPF) or keep as a blob (for binary assets like images). Pairs with
/// [`list_zip_entries`] to power click-to-view in the demo.
#[wasm_bindgen(js_name = readZipEntry)]
pub fn read_zip_entry(hwpx: &[u8], path: &str) -> Result<Vec<u8>, JsError> {
    polaris_dvc_hwpx::read_zip_entry(hwpx, path).map_err(|e| JsError::new(&e.to_string()))
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
