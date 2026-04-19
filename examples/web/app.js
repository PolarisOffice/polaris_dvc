// polaris_rhwpdvc — WASM demo glue.
//
// Expects the server to be started from the repo root so that
// `../../crates/polaris-rhwpdvc-wasm/pkg/` resolves correctly. See the
// README alongside this file.

import init, {
  validate,
} from "../../crates/polaris-rhwpdvc-wasm/pkg/polaris_rhwpdvc.js";

const $ = (sel) => document.querySelector(sel);

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

let hwpxBytes = null;
let wasmReady = false;

function setStatus(text, cls = "") {
  const el = $("#status");
  el.textContent = text;
  el.className = cls;
}

function loadPreset(name) {
  const spec = PRESETS[name] ?? {};
  $("#spec").value = JSON.stringify(spec, null, 2);
}

function renderResults(list) {
  const container = $("#results");
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
        <summary>Raw JSON (DVC-shaped)</summary>
        <pre>${escapeHtml(JSON.stringify(list, null, 2))}</pre>
      </details>`;
    return;
  }

  // Group by ErrorCode so the UI is readable when there are many records.
  const groups = new Map();
  for (const v of list) {
    const key = v.ErrorCode;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(v);
  }

  const rows = list
    .map((v) => {
      const loc = `p.${v.PageNo}/l.${v.LineNo}`;
      const table = v.IsInTable
        ? `table ${v.TableID} (${v.TableRow},${v.TableCol})`
        : "—";
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
          <td class="loc">char=${v.CharIDRef} para=${v.ParaPrIDRef}</td>
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
      <pre>${escapeHtml(JSON.stringify(list, null, 2))}</pre>
    </details>`;
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function humanBytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

function updateRunButton() {
  $("#run").disabled = !(wasmReady && hwpxBytes);
}

async function handleFile(file) {
  if (!file) return;
  const buf = await file.arrayBuffer();
  hwpxBytes = new Uint8Array(buf);
  $("#doc-info").textContent = `Loaded: ${file.name} (${humanBytes(buf.byteLength)})`;
  updateRunButton();
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
  setStatus("Validating…");
  try {
    const t0 = performance.now();
    const result = validate(hwpxBytes, spec);
    const dt = performance.now() - t0;
    setStatus(`Done in ${dt.toFixed(1)} ms.`, "ok");
    renderResults(result);
  } catch (e) {
    setStatus(`Validation failed: ${e.message || e}`, "err");
    $("#results").innerHTML =
      `<div class="empty">Error: ${escapeHtml(String(e.message || e))}</div>`;
  }
}

function wireDropZone() {
  const drop = $("#drop");
  const input = $("#file-input");
  drop.addEventListener("click", () => input.click());
  input.addEventListener("change", (e) => handleFile(e.target.files[0]));
  drop.addEventListener("dragover", (e) => {
    e.preventDefault();
    drop.classList.add("dragover");
  });
  drop.addEventListener("dragleave", () => drop.classList.remove("dragover"));
  drop.addEventListener("drop", (e) => {
    e.preventDefault();
    drop.classList.remove("dragover");
    handleFile(e.dataTransfer.files[0]);
  });
}

function wirePresets() {
  for (const btn of document.querySelectorAll(".spec-presets button")) {
    btn.addEventListener("click", () => loadPreset(btn.dataset.preset));
  }
}

async function main() {
  loadPreset("charshape");
  wireDropZone();
  wirePresets();
  $("#run").addEventListener("click", runValidation);

  try {
    await init();
    wasmReady = true;
    setStatus("Ready. Drop an HWPX file to enable Validate.");
    updateRunButton();
  } catch (e) {
    setStatus(`WASM init failed: ${e.message || e}`, "err");
  }
}

main();
