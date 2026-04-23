// polaris_dvc — WASM demo glue.
//
// Import path differs between local dev (served from repo root, so the
// wasm pkg lives at `../../crates/...`) and the GitHub Pages build (the
// workflow flattens everything into `site/` and rewrites this import to
// `./pkg/polaris_dvc.js`). The rewrite happens in
// `.github/workflows/pages.yml`; keep the local-dev string as-is.

import init, {
  validate,
  validateXml,
  describeError,
  listZipEntries,
  readZipEntry,
} from "../../crates/polaris-dvc-wasm/pkg/polaris_dvc.js";

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
//
// "Minimal" is an empty spec: the Integrity (JID 11000) and Container
// (JID 12000) axes are always-on in Extended mode, so an empty spec
// still catches orphan refs / bad mimetype / broken BinData sync.
// To run the Schema axis (13000) use the "Enable schema checks"
// option in the Options panel — it's a flag, not a preset.
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

// Axis metadata shared by the UI layer. Each JID range maps to one
// axis; the label and short description appear in the breakdown
// chips and the per-row badge. Keep this in sync with
// `crates/polaris-dvc-core/src/error_codes.rs::Category`.
const AXES = [
  { id: "rule",      label: "Rule",      min: 1000,  max: 7999,  desc: "DVC-compatible rule spec (CharShape / ParaShape / Table / …)" },
  { id: "integrity", label: "Integrity", min: 11000, max: 11999, desc: "Cross-ref / manifest / lineseg consistency (polaris-original)" },
  { id: "container", label: "Container", min: 12000, max: 12999, desc: "ZIP well-formedness (polaris-original)" },
  { id: "schema",    label: "Schema",    min: 13000, max: 13999, desc: "KS X 6101 XSD conformance (polaris-original, opt-in)" },
];
function axisForCode(code) {
  for (const a of AXES) {
    if (code >= a.min && code <= a.max) return a.id;
  }
  return "unknown";
}

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
      ? "(local dev: run `cargo test -p polaris-dvc-core --test golden` to materialize fixtures, or use the live site)"
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
    refreshFileExplorer();
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
  refreshFileExplorer();
}

// -------------------------------------------------------------------
// File explorer (ZIP tree + content viewer)
// -------------------------------------------------------------------
//
// Opens the `#explorer-panel` section whenever a new HWPX file is
// available. Tree is built from `listZipEntries` (pure-Rust helper
// exposed via WASM); clicking a file calls `readZipEntry` on demand
// so we never stash extra copies in JS memory. Violation rows that
// carry a `FileLabel` + `ByteOffset` pair (schema/integrity) get a
// click handler that jumps here and highlights the relevant line.
let treeEntries = [];
let selectedTreePath = null;

function refreshFileExplorer() {
  if (!hwpxBytes) return;
  try {
    treeEntries = listZipEntries(hwpxBytes);
  } catch (e) {
    treeEntries = [];
    $("#tree").innerHTML =
      `<div class="empty">Not a valid ZIP: ${escapeHtml(e.message)}</div>`;
    $("#viewer").innerHTML =
      `<div class="empty">File is not a ZIP container.</div>`;
    return;
  }
  renderFileTree(treeEntries);
  $("#viewer").innerHTML =
    `<div class="empty">Click a file on the left to view its contents.</div>`;
  selectedTreePath = null;
}

// Build a nested tree view from the flat path list. Directory
// markers (paths ending in `/`) are skipped — we synthesize folders
// from the path components, which matches what most ZIP tools do.
function renderFileTree(entries) {
  const tree = $("#tree");
  if (!entries.length) {
    tree.innerHTML = `<div class="empty">No entries.</div>`;
    return;
  }
  const root = { dirs: new Map(), files: [] };
  for (const e of entries) {
    if (e.isDirectory) continue;
    const parts = e.path.split("/");
    let node = root;
    for (let i = 0; i < parts.length - 1; i++) {
      const p = parts[i];
      if (!node.dirs.has(p))
        node.dirs.set(p, { dirs: new Map(), files: [] });
      node = node.dirs.get(p);
    }
    node.files.push({
      name: parts[parts.length - 1],
      path: e.path,
      size: e.size,
      compression: e.compression,
    });
  }
  const html = [];
  function walk(node, depth) {
    for (const [name, sub] of node.dirs) {
      const pad = 8 + depth * 14;
      html.push(
        `<div class="tree-node dir" style="padding-left:${pad}px">` +
          `<span class="tree-icon">📂</span>${escapeHtml(name)}/</div>`,
      );
      walk(sub, depth + 1);
    }
    for (const f of node.files) {
      const pad = 8 + depth * 14;
      const comp = f.compression === "stored" ? "" : ` · ${f.compression}`;
      html.push(
        `<div class="tree-node file" data-path="${escapeHtml(f.path)}" style="padding-left:${pad}px">` +
          `<span class="tree-icon">📄</span>${escapeHtml(f.name)}` +
          `<span class="tree-meta">${humanBytes(f.size)}${comp}</span></div>`,
      );
    }
  }
  walk(root, 0);
  tree.innerHTML = html.join("");
  tree.querySelectorAll(".tree-node.file").forEach((el) => {
    el.addEventListener("click", () => openFileInViewer(el.dataset.path));
  });
}

// Decide whether a file should be rendered with XML syntax
// highlighting. Hancom HWPX uses XML for everything except mimetype
// and binary assets; covers .xml / .hpf (OPF package) / .rels
// (container/relationships) / settings bits.
function looksLikeXml(path, head) {
  if (/\.(xml|hpf|rels)$/i.test(path)) return true;
  // Some files (e.g. settings.xml inside META-INF variants) don't
  // have canonical extensions — sniff the first bytes instead.
  const prefix = head.slice(0, 40).trimStart();
  return prefix.startsWith("<?xml") || prefix.startsWith("<");
}

// Minimal stateful XML tokenizer → colored HTML. Preserves all
// characters verbatim (offset/line math downstream still works);
// every `<span>` is closed before any `\n` so splitting by newline
// produces complete per-line markup. Handles PI, comment, CDATA,
// opening/closing tags, attributes with single- or double-quoted
// values. Unknown / malformed input falls through as plain text
// rather than throwing — we want the viewer to show *something*
// even for broken fragments.
function highlightXml(src) {
  const out = [];
  // Push text wrapped in `<span class="x-<cls>">…</span>`, splitting
  // on newlines so no span ever crosses a line boundary.
  function push(cls, raw) {
    if (!raw) return;
    const parts = raw.split("\n");
    for (let i = 0; i < parts.length; i++) {
      if (i > 0) out.push("\n");
      if (parts[i])
        out.push(`<span class="x-${cls}">${escapeHtml(parts[i])}</span>`);
    }
  }
  function pushRaw(s) {
    if (!s) return;
    // Plain text (outside tags) — still escape, preserve newlines.
    out.push(escapeHtml(s));
  }
  let i = 0;
  const n = src.length;
  while (i < n) {
    const lt = src.indexOf("<", i);
    if (lt === -1) {
      pushRaw(src.slice(i));
      break;
    }
    if (lt > i) pushRaw(src.slice(i, lt));
    if (src.startsWith("<!--", lt)) {
      const end = src.indexOf("-->", lt + 4);
      const stop = end === -1 ? n : end + 3;
      push("comment", src.slice(lt, stop));
      i = stop;
    } else if (src.startsWith("<![CDATA[", lt)) {
      const end = src.indexOf("]]>", lt + 9);
      const stop = end === -1 ? n : end + 3;
      push("cdata", src.slice(lt, stop));
      i = stop;
    } else if (src.startsWith("<?", lt)) {
      const end = src.indexOf("?>", lt + 2);
      const stop = end === -1 ? n : end + 2;
      push("pi", src.slice(lt, stop));
      i = stop;
    } else {
      // Regular tag: find matching `>` that isn't inside a quoted attr.
      let j = lt + 1;
      let inQuote = null;
      while (j < n) {
        const c = src[j];
        if (inQuote) {
          if (c === inQuote) inQuote = null;
        } else if (c === '"' || c === "'") {
          inQuote = c;
        } else if (c === ">") {
          break;
        }
        j++;
      }
      const stop = j < n ? j + 1 : n;
      emitTag(src.slice(lt, stop), push);
      i = stop;
    }
  }
  return out.join("");
}

// Highlight one `<…>` unit. Split off opening punctuation, tag name,
// attributes (name = "value"), and closing punctuation.
function emitTag(raw, push) {
  const open = raw.startsWith("</") ? "</" : "<";
  const close = raw.endsWith("/>") ? "/>" : raw.endsWith(">") ? ">" : "";
  const inner = raw.slice(open.length, raw.length - close.length);
  // Tag name (may be empty on malformed fragments).
  const nameMatch = /^[a-zA-Z_][\w:.-]*/.exec(inner);
  const name = nameMatch ? nameMatch[0] : "";
  let rest = inner.slice(name.length);
  push("punct", open);
  if (name) push("tagname", name);
  // Attributes inside the tag body.
  while (rest.length) {
    const ws = /^\s+/.exec(rest);
    if (ws) {
      push("text", ws[0]);
      rest = rest.slice(ws[0].length);
      continue;
    }
    const attr = /^[a-zA-Z_][\w:.-]*/.exec(rest);
    if (attr) {
      push("attr", attr[0]);
      rest = rest.slice(attr[0].length);
      if (rest[0] === "=") {
        push("punct", "=");
        rest = rest.slice(1);
        const q = rest[0];
        if (q === '"' || q === "'") {
          const endQ = rest.indexOf(q, 1);
          const until = endQ === -1 ? rest.length : endQ + 1;
          push("val", rest.slice(0, until));
          rest = rest.slice(until);
        }
      }
      continue;
    }
    // Anything else — emit as plain text so malformed fragments
    // don't infinite-loop.
    push("text", rest[0]);
    rest = rest.slice(1);
  }
  if (close) push("punct", close);
}

// Sniff image bytes for a renderable MIME type. Returns the mime
// string (`"image/png"`, `"image/jpeg"`, …) on match, `null` when the
// bytes aren't an image the browser can show inline.
//
// Magic-byte detection is the primary signal — file extensions in
// HWPX `BinData/` are often generic (`image1.jpg` even when the
// bytes are actually PNG). We fall back to the extension only for
// formats whose signature overlaps with plain binary (e.g. ICO).
function detectImageMime(bytes, path) {
  const b = bytes;
  if (b.length < 8) return null;
  // PNG: 89 50 4E 47 0D 0A 1A 0A
  if (b[0] === 0x89 && b[1] === 0x50 && b[2] === 0x4e && b[3] === 0x47)
    return "image/png";
  // JPEG: FF D8 FF
  if (b[0] === 0xff && b[1] === 0xd8 && b[2] === 0xff) return "image/jpeg";
  // GIF: 47 49 46 38 ("GIF8")
  if (b[0] === 0x47 && b[1] === 0x49 && b[2] === 0x46 && b[3] === 0x38)
    return "image/gif";
  // BMP: 42 4D ("BM")
  if (b[0] === 0x42 && b[1] === 0x4d) return "image/bmp";
  // WebP: 52 49 46 46 ... 57 45 42 50 ("RIFF" + "WEBP" at offset 8)
  if (
    b[0] === 0x52 &&
    b[1] === 0x49 &&
    b[2] === 0x46 &&
    b[3] === 0x46 &&
    b[8] === 0x57 &&
    b[9] === 0x45 &&
    b[10] === 0x42 &&
    b[11] === 0x50
  )
    return "image/webp";
  // SVG is text-based; detect by extension since the magic-byte pass
  // above already let it through to the text path.
  if (/\.svg$/i.test(path)) return "image/svg+xml";
  return null;
}

// Build a Blob URL from the raw bytes and render inside the viewer.
// We use `Blob + URL.createObjectURL` rather than base64 data URIs so
// the browser doesn't pay the ~33 % encoding overhead on multi-MB
// images. The URL gets revoked when the viewer is re-rendered or the
// page unloads — the tracked list lets us clean up without leaking.
let pendingImageUrls = [];
function revokePendingImageUrls() {
  for (const u of pendingImageUrls) URL.revokeObjectURL(u);
  pendingImageUrls = [];
}
function renderImageInViewer(viewer, path, bytes, mime) {
  revokePendingImageUrls();
  const blob = new Blob([bytes], { type: mime });
  const url = URL.createObjectURL(blob);
  pendingImageUrls.push(url);
  const sizeLabel = `${humanBytes(bytes.length)} · ${mime}`;
  viewer.innerHTML =
    `<div class="viewer-header">${escapeHtml(path)} · ${sizeLabel}</div>` +
    `<div class="image-preview"><img src="${url}" alt="${escapeHtml(path)}" /></div>`;
}

// Render one ZIP entry's contents in the right-hand pane. If
// `highlightOffset` is set (non-null), compute the matching line
// and scroll there with a highlight.
function openFileInViewer(path, highlightOffset = null) {
  if (!hwpxBytes) return;
  selectedTreePath = path;
  $("#tree")
    .querySelectorAll(".tree-node.file")
    .forEach((el) => {
      el.classList.toggle("selected", el.dataset.path === path);
    });
  const viewer = $("#viewer");
  let bytes;
  try {
    bytes = readZipEntry(hwpxBytes, path);
  } catch (e) {
    viewer.innerHTML =
      `<div class="viewer-header">${escapeHtml(path)}</div>` +
      `<div class="empty">Read failed: ${escapeHtml(e.message)}</div>`;
    return;
  }
  // Image detection runs before the text/binary fallback so image
  // assets in `BinData/` render as a thumbnail instead of "binary,
  // not displayed." Checks magic bytes (reliable) with file extension
  // as a hint for the MIME type when bytes are ambiguous.
  const imageMime = detectImageMime(bytes, path);
  if (imageMime) {
    renderImageInViewer(viewer, path, bytes, imageMime);
    return;
  }
  // UTF-8 decode and sniff for binary content (presence of C0 control
  // bytes other than tab/newline/CR in the first 512 bytes).
  const text = new TextDecoder("utf-8", { fatal: false }).decode(bytes);
  const looksBinary = /[\x00-\x08\x0E-\x1F]/.test(text.slice(0, 512));
  if (looksBinary) {
    // Vector formats we detect but can't render natively (WMF, EMF)
    // get a specific hint instead of a generic "binary" message.
    const vectorHint =
      /\.(wmf|emf)$/i.test(path)
        ? " (vector image — WMF/EMF not rendered natively by browsers; download to inspect)"
        : "";
    viewer.innerHTML =
      `<div class="viewer-header">${escapeHtml(path)} — ${humanBytes(bytes.length)} (binary)</div>` +
      `<div class="binary">Binary data. Click-to-view not supported for this entry.${vectorHint}</div>`;
    return;
  }
  // Map byte offset → 1-based line number by counting LFs before it.
  let targetLine = null;
  if (
    highlightOffset != null &&
    Number.isFinite(highlightOffset) &&
    highlightOffset > 0 &&
    highlightOffset < bytes.length
  ) {
    targetLine = 1;
    for (let i = 0; i < highlightOffset; i++) {
      if (bytes[i] === 0x0a) targetLine++;
    }
  }
  // XML files get syntax-colored; other text files render as-is.
  // The highlighter guarantees no span crosses a newline, so the
  // per-line split below still produces complete markup for each row.
  const highlighted = looksLikeXml(path, text) ? highlightXml(text) : escapeHtml(text);
  const lines = highlighted.split("\n");
  const rendered = lines
    .map((l, idx) => {
      const ln = idx + 1;
      const cls = targetLine === ln ? "line highlight" : "line";
      // Render a non-breaking space for empty lines so the row still
      // has height; anything else passes through unchanged (it's
      // already escaped + span-wrapped by `highlightXml` / `escapeHtml`).
      const body = l.length ? l : "&nbsp;";
      return `<div class="${cls}" data-line="${ln}"><span class="ln">${ln}</span><span class="code">${body}</span></div>`;
    })
    .join("");
  const lineInfo = targetLine ? ` — line ${targetLine}` : "";
  viewer.innerHTML =
    `<div class="viewer-header">${escapeHtml(path)} · ${humanBytes(bytes.length)}${lineInfo}</div>` +
    `<div class="viewer-content">${rendered}</div>`;
  if (targetLine && highlightOffset != null) {
    // Defer to rAF: the click handler's panel scroll and our own
    // layout need to settle before we resolve DOM positions for the
    // Range. `selectOffendingRange` handles both the native
    // selection and scrolling the selection into view — using the
    // range's own start node as the scroll anchor so `pre-wrap`
    // line wrapping doesn't misalign the viewport against the
    // visual position of the selection.
    requestAnimationFrame(() => {
      selectOffendingRange(viewer, bytes, text, highlightOffset);
    });
  }
}

// Given the offending byte position (quick-xml's
// `Reader::buffer_position` captured BEFORE `read_event_into`, i.e.
// the position at the START of the tag that triggered the finding),
// build a native browser text selection covering the whole open tag —
// from the `<` up to and including the matching `>` — so the user
// sees exactly the element that violated the rule. Byte→char→(line,
// col)→DOM conversion accounts for multi-byte UTF-8 and for HTML
// entities expanding to single characters in the rendered text nodes.
function selectOffendingRange(viewerEl, bytes, text, startByte) {
  if (startByte < 0 || startByte >= bytes.length) return;
  // Scan forward from startByte for the matching `>`, ignoring `>`
  // characters inside quoted attribute values.
  let endByte = -1;
  let inQuote = 0;
  for (let i = startByte; i < bytes.length; i++) {
    const c = bytes[i];
    if (inQuote) {
      if (c === inQuote) inQuote = 0;
    } else if (c === 0x22 /* " */ || c === 0x27 /* ' */) {
      inQuote = c;
    } else if (c === 0x3e /* > */) {
      endByte = i + 1;
      break;
    }
  }
  if (endByte < 0) return;
  const clampedEnd = Math.min(endByte, bytes.length);

  // 2. Byte offset → character offset (length in JS string).
  const decoder = new TextDecoder("utf-8", { fatal: false });
  const startChar = decoder.decode(bytes.slice(0, startByte)).length;
  const endChar = decoder.decode(bytes.slice(0, clampedEnd)).length;

  // 3. Char offset → (lineIdx, col) in the decoded source.
  const srcLines = text.split("\n");
  function posOf(charIdx) {
    let c = charIdx;
    for (let i = 0; i < srcLines.length; i++) {
      if (c <= srcLines[i].length) return { line: i, col: c };
      c -= srcLines[i].length + 1; // +1 for the `\n`
    }
    const last = srcLines.length - 1;
    return { line: last, col: srcLines[last].length };
  }
  const a = posOf(startChar);
  const b = posOf(endChar);

  // 4. (line,col) → DOM (text node, offset). Walk only the `.code`
  //    span inside the right line div so the gutter's line-number
  //    text nodes don't pollute the count.
  function findDom(line, col) {
    const codeEl = viewerEl.querySelector(
      `.line[data-line="${line + 1}"] .code`,
    );
    if (!codeEl) return null;
    let remaining = col;
    const walker = document.createTreeWalker(codeEl, NodeFilter.SHOW_TEXT);
    let last = null;
    while (walker.nextNode()) {
      const node = walker.currentNode;
      last = node;
      if (remaining <= node.data.length) return { node, offset: remaining };
      remaining -= node.data.length;
    }
    return last ? { node: last, offset: last.data.length } : null;
  }
  const startPos = findDom(a.line, a.col);
  const endPos = findDom(b.line, b.col);
  if (!startPos || !endPos) return;

  // 5. Build the Range + apply selection, then scroll the selection
  //    into view. We use the start node's parent instead of the
  //    containing `.line` div because with `pre-wrap` wrapping, a
  //    single logical line can span many visual rows and centering
  //    the whole line div can leave the actual selection offscreen.
  try {
    const range = document.createRange();
    range.setStart(startPos.node, startPos.offset);
    range.setEnd(endPos.node, endPos.offset);
    const sel = window.getSelection();
    if (sel) {
      sel.removeAllRanges();
      sel.addRange(range);
    }
    const scrollAnchor =
      startPos.node.nodeType === Node.TEXT_NODE
        ? startPos.node.parentElement
        : startPos.node;
    if (scrollAnchor) {
      scrollAnchor.scrollIntoView({ block: "center", inline: "nearest" });
    }
  } catch (_e) {
    // DOM positions can occasionally fall out of sync if the viewer
    // was torn down mid-call; selection is a nicety, not critical.
  }
}

// Map a violation's `FileLabel` ("header.xml", "section0",
// "content.hpf", …) to the actual ZIP path. Returns null if no match
// — caller should fall back to doing nothing.
function fileLabelToZipPath(label) {
  if (!label) return null;
  const known = {
    "header.xml": "Contents/header.xml",
    "content.hpf": "Contents/content.hpf",
    "settings.xml": "Contents/settings.xml",
    "version.xml": "Contents/version.xml",
  };
  if (known[label]) return known[label];
  const secMatch = /^section(\d+)$/.exec(label);
  if (secMatch) return `Contents/section${secMatch[1]}.xml`;
  // Not a known label — try raw path match.
  if (treeEntries.some((e) => e.path === label)) return label;
  return null;
}

// -------------------------------------------------------------------
// Validate
// -------------------------------------------------------------------
function currentOpts() {
  return {
    dvcStrict: $("#profile").value === "strict",
    stopOnFirst: $("#stop-on-first").checked,
    outputOption: $("#output-option").value,
    enableSchema: $("#enable-schema").checked,
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
  const schemaTag = opts.enableSchema ? " · schema+" : "";
  setStatus(
    `Done in ${ms.toFixed(1)} ms (${mode} · ${opts.outputOption}${schemaTag}).`,
    "ok",
  );
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

  // Bin by axis (JID range → rule/integrity/container/schema).
  // Also tally distinct error codes for the summary header.
  const axisCounts = Object.fromEntries(AXES.map((a) => [a.id, 0]));
  axisCounts.unknown = 0;
  const byCode = new Map();
  for (const v of list) {
    const a = axisForCode(v.ErrorCode);
    axisCounts[a] = (axisCounts[a] || 0) + 1;
    if (!byCode.has(v.ErrorCode)) byCode.set(v.ErrorCode, []);
    byCode.get(v.ErrorCode).push(v);
  }

  // Build the axis-breakdown chip row. Only axes that actually
  // have findings get a chip; "unknown" appears if any code fell
  // outside the registered ranges (shouldn't in practice).
  const visibleAxes = AXES.filter((a) => axisCounts[a.id] > 0).map((a) => a.id);
  if (axisCounts.unknown > 0) visibleAxes.push("unknown");
  const chipsHtml = visibleAxes
    .map((axis) => {
      const meta = AXES.find((a) => a.id === axis) || {
        id: "unknown",
        label: "Unknown",
        desc: "JID outside any registered range",
      };
      return `<button type="button" class="axis-chip active" data-axis="${axis}" title="${escapeHtml(meta.desc)}">
        <span class="dot"></span>
        <span>${meta.label}</span>
        <span class="n">${axisCounts[axis]}</span>
      </button>`;
    })
    .join("");

  const rows = list
    .map((v) => {
      const axis = axisForCode(v.ErrorCode);
      const axisMeta = AXES.find((a) => a.id === axis);
      const axisLabel = axisMeta ? axisMeta.label : "Unknown";
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
      // Preferred description source hierarchy:
      //   1. `ErrorString` — specific message from integrity / schema /
      //      container checkers (e.g., "<p> can only contain <run>,
      //      <ctrl>, … but found <charPr>"). Present only for JID
      //      11000+ since DVC-compat categories omit it.
      //   2. `describeError(ErrorCode)` — generic fallback that mirrors
      //      `ErrorCode::text()` from the core crate.
      const desc = v.ErrorString
        ? escapeHtml(v.ErrorString)
        : escapeHtml(describeError(v.ErrorCode));
      const docText = v.errorText
        ? `<div class="doc-text">“${escapeHtml(v.errorText)}”</div>`
        : "";
      // Click-to-locate: rows that carry a file hint become clickable.
      const zipPath = fileLabelToZipPath(v.FileLabel);
      const locAttrs = zipPath
        ? ` class="has-location" data-zip-path="${escapeHtml(zipPath)}" data-byte-offset="${v.ByteOffset ?? 0}" title="Click to open ${escapeHtml(zipPath)} at this position"`
        : "";
      const locCell = zipPath
        ? `<div class="loc">${loc}</div><div class="loc" style="color:var(--accent)">📍 ${escapeHtml(v.FileLabel)}</div>`
        : `<div class="loc">${loc}</div>`;
      return `
        <tr data-axis="${axis}"${locAttrs}>
          <td><span class="axis-badge ${axis}">${axisLabel}</span></td>
          <td class="error-code">${v.ErrorCode}</td>
          <td>${locCell}</td>
          <td class="loc">char=${v.CharIDRef ?? "?"} para=${v.ParaPrIDRef ?? "?"}</td>
          <td>${table}</td>
          <td>
            <div class="desc">${desc}</div>
            ${docText}
          </td>
          <td class="loc">${flags}</td>
        </tr>`;
    })
    .join("");

  container.innerHTML = `
    <div class="results-summary">
      <span class="count err">${list.length} violation(s)</span>
      <span>across ${byCode.size} error code(s) · click an axis chip to filter</span>
    </div>
    <div class="axis-breakdown">${chipsHtml}</div>
    <table class="violations">
      <thead>
        <tr>
          <th>Axis</th>
          <th>Code</th>
          <th>Location</th>
          <th>Refs</th>
          <th>Table</th>
          <th>Description</th>
          <th>Flags</th>
        </tr>
      </thead>
      <tbody>${rows}</tbody>
    </table>
    <details class="raw">
      <summary>Raw JSON (DVC-shaped)</summary>
      <pre>${escapeHtml(prettyJson(list))}</pre>
    </details>`;

  // Wire chip filtering. Multi-select — each chip is an independent
  // toggle. Rows whose axis isn't in the active set get hidden.
  const chipEls = container.querySelectorAll(".axis-chip");
  const refreshFilter = () => {
    const active = new Set(
      Array.from(chipEls)
        .filter((c) => c.classList.contains("active"))
        .map((c) => c.dataset.axis),
    );
    // If the user deactivates every chip, fall back to showing all
    // (no chips active feels like a dead state; interpret as "clear
    // filter").
    const showAll = active.size === 0;
    const trs = container.querySelectorAll("table.violations tbody tr");
    trs.forEach((tr) => {
      tr.style.display =
        showAll || active.has(tr.dataset.axis) ? "" : "none";
    });
  };
  chipEls.forEach((chip) => {
    chip.addEventListener("click", () => {
      chip.classList.toggle("active");
      refreshFilter();
    });
  });

  // Click-to-locate: clicking a violation row with a known file hint
  // jumps to the explorer, opens the file, and native-selects the
  // offending range. Order matters: scroll the panel instantly FIRST
  // (smooth animation + programmatic selection race each other; one
  // will clobber the other), then open the file. The selection is
  // set inside `openFileInViewer` after its own layout settles.
  container.querySelectorAll("tr.has-location").forEach((tr) => {
    tr.addEventListener("click", () => {
      const path = tr.dataset.zipPath;
      const off = Number(tr.dataset.byteOffset) || 0;
      if (!path) return;
      const panel = $("#explorer-panel");
      panel.scrollIntoView({ block: "start" });
      openFileInViewer(path, off);
    });
  });
}

function renderXmlResults(xml, count, opts, ms) {
  const container = $("#results");
  const mode = opts.dvcStrict ? "dvc-strict" : "extended";
  const schemaTag = opts.enableSchema ? " · schema+" : "";
  setStatus(
    `Done in ${ms.toFixed(1)} ms (${mode} · ${opts.outputOption}${schemaTag}).`,
    "ok",
  );
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
  loadPreset("minimal");
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
