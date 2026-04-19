// polaris_rhwpdvc — WASM demo glue.
//
// Import path differs between local dev (served from repo root, so the
// wasm pkg lives at `../../crates/...`) and the GitHub Pages build (the
// workflow flattens everything into `site/` and rewrites this import to
// `./pkg/polaris_rhwpdvc.js`). The rewrite happens in
// `.github/workflows/pages.yml`; keep the local-dev string as-is.

import init, {
  validate,
  validateXml,
} from "../../crates/polaris-rhwpdvc-wasm/pkg/polaris_rhwpdvc.js";

const $ = (sel) => document.querySelector(sel);

// -------------------------------------------------------------------
// Paths — differ between local dev (served from repo root) and the
// deployed site (all golden fixtures co-located under ./golden/).
// We detect the local-dev case from the URL path.
// -------------------------------------------------------------------
const IS_LOCAL_DEV = location.pathname.includes("/examples/web/");
const GOLDEN_BASE = IS_LOCAL_DEV ? "../../testdata/golden" : "./golden";

// Quickstart presets — tiny, hand-written specs used outside the
// golden-case flow. Kept separate from golden presets so users can
// experiment without loading a full case.
const PRESETS = {
  minimal: {},
  charshape: {
    charshape: {
      font: ["바탕", "돋움", "굴림", "맑은 고딕"],
      fontsize: { min: 10, max: 12 },
      bold: false,
    },
  },
  table: {
    table: {
      border: [{ position: 1, bordertype: 1 }],
      "bgfill-type": 1,
    },
  },
  all: {
    charshape: {
      font: ["바탕", "돋움", "굴림"],
      fontsize: { min: 10, max: 12 },
      bold: false,
      italic: false,
      ratio: 100,
    },
    parashape: { linespacingvalue: 160, indent: 0 },
    table: {
      border: [{ position: 1, bordertype: 1 }],
      "table-in-table": false,
    },
    style: { permission: false },
    hyperlink: { permission: false },
    macro: { permission: false },
    specialcharacter: { minimum: 32, maximum: 1048575 },
  },
};

// Mutable state
let hwpxBytes = null;
let hwpxName = null;
let wasmReady = false;
let lastResult = null; // { kind: "json"|"xml", payload: any|string, violations: number }
let activeGolden = null; // case name when loaded from a golden preset

// -------------------------------------------------------------------
// Small utilities
// -------------------------------------------------------------------
function setStatus(text, cls = "") {
  const el = $("#status");
  el.textContent = text;
  el.className = cls;
}

function humanBytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function downloadBlob(bytes, filename, mime) {
  const blob = new Blob([bytes], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.rel = "noopener";
  document.body.appendChild(a);
  a.click();
  a.remove();
  // Let the browser start the download before revoking.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function prettyJson(v) {
  return JSON.stringify(v, null, 2);
}

function updateRunButton() {
  $("#run").disabled = !(wasmReady && hwpxBytes);
}

// -------------------------------------------------------------------
// Preset (hand-written) + spec I/O
// -------------------------------------------------------------------
function loadPreset(name) {
  const spec = PRESETS[name] ?? {};
  $("#spec").value = prettyJson(spec);
  $("#spec-info").textContent = `preset: ${name}`;
  activeGolden = null;
  $("#diff-expected-btn").style.display = "none";
}

async function handleSpecFile(file) {
  if (!file) return;
  const text = await file.text();
  try {
    const parsed = JSON.parse(text);
    $("#spec").value = prettyJson(parsed);
    $("#spec-info").textContent = `loaded: ${file.name} (${humanBytes(text.length)})`;
    activeGolden = null;
    $("#diff-expected-btn").style.display = "none";
  } catch (e) {
    $("#spec-info").textContent = `${file.name} is not valid JSON: ${e.message}`;
  }
}

function saveSpec() {
  const text = $("#spec").value || "{}";
  const name = activeGolden ? `${activeGolden}-spec.json` : "spec.json";
  downloadBlob(text, name, "application/json");
}

// -------------------------------------------------------------------
// Golden preset (manifest + triplet fetch)
// -------------------------------------------------------------------
async function loadGoldenManifest() {
  const sel = $("#golden-preset");
  try {
    const res = await fetch(`${GOLDEN_BASE}/manifest.json`, {
      cache: "no-cache",
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const cases = await res.json();
    for (const c of cases) {
      const opt = document.createElement("option");
      opt.value = c.name;
      opt.textContent = c.label;
      sel.appendChild(opt);
    }
    sel.disabled = cases.length === 0;
  } catch (e) {
    // Local dev without the manifest file is a common case; leave the
    // dropdown disabled with a hint and fall back to the hand-written
    // presets.
    sel.disabled = true;
    const label = IS_LOCAL_DEV
      ? "(local dev: run `cargo test -p polaris-rhwpdvc-core --test golden` to materialize fixtures, or use the live site)"
      : "(no manifest)";
    sel.options[0].textContent = `— ${label} —`;
    console.warn("golden manifest fetch failed:", e);
  }
}

async function loadGoldenCase(name) {
  if (!name) {
    activeGolden = null;
    $("#diff-expected-btn").style.display = "none";
    return;
  }
  setStatus(`Loading golden case ${name}…`);
  try {
    const [specRes, docRes] = await Promise.all([
      fetch(`${GOLDEN_BASE}/${name}/spec.json`, { cache: "no-cache" }),
      fetch(`${GOLDEN_BASE}/${name}/doc.hwpx`, { cache: "no-cache" }),
    ]);
    if (!specRes.ok || !docRes.ok) {
      throw new Error(
        `fetch failed: spec=${specRes.status} doc=${docRes.status}`,
      );
    }
    const specText = await specRes.text();
    const specObj = JSON.parse(specText);
    $("#spec").value = prettyJson(specObj);
    $("#spec-info").textContent = `golden: ${name}/spec.json`;

    const docBuf = await docRes.arrayBuffer();
    hwpxBytes = new Uint8Array(docBuf);
    hwpxName = `${name}/doc.hwpx`;
    $("#doc-info").textContent =
      `Loaded golden: ${hwpxName} (${humanBytes(docBuf.byteLength)})`;

    activeGolden = name;
    $("#diff-expected-btn").style.display = "";
    updateRunButton();
    setStatus(`Golden case ${name} ready. Hit Validate.`, "ok");
  } catch (e) {
    setStatus(`Failed to load golden ${name}: ${e.message}`, "err");
  }
}

async function diffAgainstExpected() {
  if (!activeGolden || !lastResult) return;
  const ext = lastResult.kind === "xml" ? "xml" : "json";
  const url = `${GOLDEN_BASE}/${activeGolden}/expected.${ext}`;
  try {
    const res = await fetch(url, { cache: "no-cache" });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const expected = await res.text();
    const actual =
      lastResult.kind === "xml"
        ? lastResult.payload
        : prettyJson(lastResult.payload) + "\n";
    const match = actual.trim() === expected.trim();
    const container = $("#results");
    const banner = match
      ? `<div class="results-summary"><span class="count ok">✅ Matches expected.${ext} byte-exact.</span></div>`
      : `<div class="results-summary"><span class="count err">❌ Differs from expected.${ext}</span></div>
         <details open>
           <summary>Actual vs expected</summary>
           <pre>${escapeHtml(diffPreview(actual, expected))}</pre>
         </details>`;
    container.insertAdjacentHTML("afterbegin", banner);
  } catch (e) {
    setStatus(`Diff failed: ${e.message}`, "err");
  }
}

function diffPreview(a, b) {
  // Poor man's diff: show actual on top, expected below, marking the
  // first differing line. Good enough for spot-checking byte parity.
  const al = a.split("\n");
  const bl = b.split("\n");
  const max = Math.max(al.length, bl.length);
  let firstDiff = -1;
  for (let i = 0; i < max; i++) {
    if (al[i] !== bl[i]) {
      firstDiff = i;
      break;
    }
  }
  const marker = firstDiff < 0 ? "(no diff)" : `first diff @ line ${firstDiff + 1}`;
  return `=== actual (${al.length} lines) ===\n${a}\n=== expected (${bl.length} lines) ===\n${b}\n=== ${marker} ===`;
}

// -------------------------------------------------------------------
// Doc (hwpx) I/O
// -------------------------------------------------------------------
async function handleDocFile(file) {
  if (!file) return;
  const buf = await file.arrayBuffer();
  hwpxBytes = new Uint8Array(buf);
  hwpxName = file.name;
  $("#doc-info").textContent = `Loaded: ${file.name} (${humanBytes(buf.byteLength)})`;
  // User-dropped file invalidates any active-golden match context.
  activeGolden = null;
  $("#diff-expected-btn").style.display = "none";
  updateRunButton();
}

// -------------------------------------------------------------------
// Validate
// -------------------------------------------------------------------
function currentOpts() {
  return {
    dvcStrict: $("#profile").value === "strict",
    stopOnFirst: $("#stop-on-first").checked,
    outputOption: $("#output-option").value,
  };
}

function runValidation() {
  if (!hwpxBytes) return;
  let spec;
  try {
    spec = JSON.parse($("#spec").value || "{}");
  } catch (e) {
    setStatus(`Spec JSON parse error: ${e.message}`, "err");
    return;
  }
  const opts = currentOpts();
  const format = $("#format").value;
  setStatus("Validating…");
  try {
    const t0 = performance.now();
    if (format === "xml") {
      const xml = validateXml(hwpxBytes, spec, opts);
      const dt = performance.now() - t0;
      // Rough violation count by counting <violation …/> elements.
      const vcount = (xml.match(/<violation\b/g) || []).length;
      lastResult = { kind: "xml", payload: xml, violations: vcount };
      renderXmlResults(xml, vcount, opts, dt);
    } else {
      const result = validate(hwpxBytes, spec, opts);
      const dt = performance.now() - t0;
      lastResult = {
        kind: "json",
        payload: result,
        violations: Array.isArray(result) ? result.length : 0,
      };
      renderJsonResults(result, opts, dt);
    }
    $("#result-toolbar").style.display = "";
  } catch (e) {
    setStatus(`Validation failed: ${e.message || e}`, "err");
    $("#results").innerHTML =
      `<div class="empty">Error: ${escapeHtml(String(e.message || e))}</div>`;
    lastResult = null;
  }
}

function renderJsonResults(list, opts, ms) {
  const container = $("#results");
  const mode = opts.dvcStrict ? "dvc-strict" : "extended";
  setStatus(`Done in ${ms.toFixed(1)} ms (${mode} · ${opts.outputOption}).`, "ok");
  if (!Array.isArray(list)) {
    container.innerHTML = `<div class="empty">Unexpected result shape.</div>`;
    return;
  }
  if (list.length === 0) {
    container.innerHTML = `
      <div class="results-summary">
        <span class="count ok">0 violations — document matches the spec.</span>
      </div>
      <details class="raw" open>
        <summary>Raw JSON</summary>
        <pre>${escapeHtml(prettyJson(list))}</pre>
      </details>`;
    return;
  }

  const groups = new Map();
  for (const v of list) {
    const key = v.ErrorCode;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(v);
  }

  const rows = list
    .map((v) => {
      const loc = `p.${v.PageNo ?? "?"}/l.${v.LineNo ?? "?"}`;
      const table =
        v.IsInTable === true
          ? `table ${v.TableID ?? "?"} (${v.TableRow ?? 0},${v.TableCol ?? 0})`
          : v.IsInTable === false
            ? "—"
            : "";
      const flags = [
        v.IsInTableInTable ? "nested-table" : null,
        v.UseStyle ? "style" : null,
        v.UseHyperlink ? "hyperlink" : null,
        v.IsInShape ? "shape" : null,
      ]
        .filter(Boolean)
        .join(", ");
      return `
        <tr>
          <td class="error-code">${v.ErrorCode}</td>
          <td class="loc">${loc}</td>
          <td class="loc">char=${v.CharIDRef ?? "?"} para=${v.ParaPrIDRef ?? "?"}</td>
          <td>${table}</td>
          <td>${escapeHtml(v.errorText || "")}</td>
          <td class="loc">${flags}</td>
        </tr>`;
    })
    .join("");

  container.innerHTML = `
    <div class="results-summary">
      <span class="count err">${list.length} violation(s)</span>
      <span>across ${groups.size} error code(s)</span>
    </div>
    <table class="violations">
      <thead>
        <tr>
          <th>Code</th>
          <th>Location</th>
          <th>Refs</th>
          <th>Table</th>
          <th>Message</th>
          <th>Flags</th>
        </tr>
      </thead>
      <tbody>${rows}</tbody>
    </table>
    <details class="raw">
      <summary>Raw JSON (DVC-shaped)</summary>
      <pre>${escapeHtml(prettyJson(list))}</pre>
    </details>`;
}

function renderXmlResults(xml, count, opts, ms) {
  const container = $("#results");
  const mode = opts.dvcStrict ? "dvc-strict" : "extended";
  setStatus(`Done in ${ms.toFixed(1)} ms (${mode} · ${opts.outputOption}).`, "ok");
  const cls = count === 0 ? "ok" : "err";
  const summary =
    count === 0
      ? "0 violations — document matches the spec."
      : `${count} violation(s) (XML)`;
  container.innerHTML = `
    <div class="results-summary">
      <span class="count ${cls}">${summary}</span>
    </div>
    <pre>${escapeHtml(xml)}</pre>`;
}

// -------------------------------------------------------------------
// Downloads
// -------------------------------------------------------------------
function downloadJson() {
  if (!lastResult) return;
  const payload =
    lastResult.kind === "json"
      ? lastResult.payload
      : "[]"; // should not happen — button hidden in XML mode
  const body =
    typeof payload === "string" ? payload : prettyJson(payload);
  const base = activeGolden ?? (hwpxName || "report").replace(/\.hwpx$/, "");
  downloadBlob(body + "\n", `${base}.expected.json`, "application/json");
}

function downloadXml() {
  if (!lastResult) return;
  let xml;
  if (lastResult.kind === "xml") {
    xml = lastResult.payload;
  } else {
    // Re-run under XML to get the same report as XML. The user may have
    // just validated under JSON mode; this keeps the button useful.
    let spec;
    try {
      spec = JSON.parse($("#spec").value || "{}");
    } catch {
      return;
    }
    xml = validateXml(hwpxBytes, spec, currentOpts());
  }
  const base = activeGolden ?? (hwpxName || "report").replace(/\.hwpx$/, "");
  downloadBlob(xml, `${base}.expected.xml`, "application/xml");
}

// -------------------------------------------------------------------
// Wire up
// -------------------------------------------------------------------
function wireDropZone() {
  const drop = $("#drop");
  const input = $("#doc-file-input");
  drop.addEventListener("click", () => input.click());
  input.addEventListener("change", (e) => handleDocFile(e.target.files[0]));
  drop.addEventListener("dragover", (e) => {
    e.preventDefault();
    drop.classList.add("dragover");
  });
  drop.addEventListener("dragleave", () => drop.classList.remove("dragover"));
  drop.addEventListener("drop", (e) => {
    e.preventDefault();
    drop.classList.remove("dragover");
    handleDocFile(e.dataTransfer.files[0]);
  });
}

function wirePresets() {
  for (const btn of document.querySelectorAll(".toolbar [data-preset]")) {
    btn.addEventListener("click", () => loadPreset(btn.dataset.preset));
  }
}

function wireSpecIO() {
  const fileInput = $("#spec-file-input");
  $("#load-spec-btn").addEventListener("click", () => fileInput.click());
  fileInput.addEventListener("change", (e) => handleSpecFile(e.target.files[0]));
  $("#save-spec-btn").addEventListener("click", saveSpec);
}

function wireResultButtons() {
  $("#download-json-btn").addEventListener("click", downloadJson);
  $("#download-xml-btn").addEventListener("click", downloadXml);
  $("#diff-expected-btn").addEventListener("click", diffAgainstExpected);
}

function wireGoldenPreset() {
  $("#golden-preset").addEventListener("change", (e) =>
    loadGoldenCase(e.target.value),
  );
}

async function main() {
  loadPreset("charshape");
  wireDropZone();
  wirePresets();
  wireSpecIO();
  wireResultButtons();
  wireGoldenPreset();
  $("#run").addEventListener("click", runValidation);

  // Kick off WASM init + golden manifest load in parallel.
  const [wasmResult] = await Promise.allSettled([init(), loadGoldenManifest()]);
  if (wasmResult.status === "fulfilled") {
    wasmReady = true;
    setStatus("Ready. Load a golden case or drop an .hwpx file.");
    updateRunButton();
  } else {
    setStatus(
      `WASM init failed: ${wasmResult.reason?.message || wasmResult.reason}`,
      "err",
    );
  }
}

main();
