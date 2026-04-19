# Web demo

Minimal browser UI around `polaris-rhwpdvc-wasm`. Drop an `.hwpx`, edit
the rule spec, click **Validate**, see the DVC-shaped violation list.

## Run locally

The page loads the WASM module via ES modules with a relative path
(`../../crates/polaris-rhwpdvc-wasm/pkg/polaris_rhwpdvc.js`), so the
server has to be started from the **repo root**.

```sh
# 1. Build the WASM package (if you haven't already):
wasm-pack build crates/polaris-rhwpdvc-wasm --target web --out-name polaris_rhwpdvc

# 2. Serve the repo root on any static HTTP server:
python3 -m http.server 8080
# or:   npx serve .

# 3. Open the demo:
#    http://localhost:8080/examples/web/
```

A `file://` open won't work — browsers refuse to instantiate WebAssembly
from non-http(s) origins.

## What you get

- **Drag-and-drop** HWPX file load (or click to pick).
- **Rule spec editor** with four quickstart presets:
  - *Minimal* — empty `{}`, no violations possible.
  - *CharShape* — font allowlist + fontsize range + bold-forbidden.
  - *Table* — border + bgfill.
  - *All categories* — a broader sweep across char/para/table/style/
    hyperlink/macro/specialcharacter.
- **Results table** grouped view with `ErrorCode`, page/line, table
  context, message, and flag columns. Raw DVC-shaped JSON is tucked into
  a `<details>` below the table.

## Feeding one of the golden fixtures

The committed `testdata/golden/*/doc.hwpx` files are real HWPX ZIPs
produced from the in-Rust fixture template — any of them work as input.

```
testdata/golden/24_table_bgfill_type_mismatch/doc.hwpx  →  pair with
testdata/golden/24_table_bgfill_type_mismatch/spec.json
```

Paste the spec into the textarea, drop the doc.hwpx, and the page
should show the same violation the Rust golden test expects.
