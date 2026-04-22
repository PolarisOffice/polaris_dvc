# Schema drift catalog (KS X 6101 vs real HWPX)

The Schema axis (JID 13000–13999) exists to surface how real HWPX
documents diverge from the formal KS X 6101 OWPML specification.
When we validate a representative HWPX against the generated schema
model, the findings fall into two buckets:

1. **Real divergence** — the document uses something the XSD does
   not declare, or uses a value the XSD does not allow. The validator
   is doing its job; the divergence is the signal.
2. **Codegen false positive** — our translation of the XSD missed a
   detail and flags a construct the spec actually allows. These are
   bugs in `tools/gen-owpml/` and must be fixed, not worked around.

This file catalogs what a full schema run on a representative real
HWPX surfaces, so new contributors can quickly tell which bucket a
new finding belongs in.

## Probe methodology

Two real-world HWPX files were scanned with `enable_schema=true`:

- `form-002.hwpx` — a form-heavy document (checkboxes, radios, etc.).
  1 140 schema findings, 12 unique patterns.
- `2025년 2분기 해외직접투자 (최종).hwpx` — a long text-heavy
  policy document.
  2 136 schema findings, 11 unique patterns.

Each unique pattern was cross-referenced against the source XSDs
under `standards/KSX6101_OWPML/` (not redistributed — see
`standards/README.md`).

## Known divergences (NOT bugs — exactly what we want to report)

| Finding | Anchor | Cause |
|---|---|---|
| `<p> can only contain <run>, but found <linesegarray>` | `section*.xml` | `<hp:linesegarray>` isn't declared anywhere in KS X 6101. Hancom extension to carry cached layout info. |
| `<paraPr> … but found <switch>` / `<tabPr> … but found <switch>` | `header.xml` | `<hp:switch>` / `<hp:case>` / `<hp:default>` is an SVG-style conditional markup mechanism Hancom added. Not in the standard. |
| `<paraPr> missing required child <lineSpacing>` / `<margin>` | `header.xml` | Cascade of `<switch>`: the required children are nested inside `<case>` branches our validator can't enter. |
| `shape="3D"` not in `LineType2` enum | `header.xml` | `LineType2` allows `NONE, SOLID, DOT, DASH, …` — `3D` is a Hancom-only extension value. |
| `letterform="2"` on `charPr` expected boolean | `header.xml` | XSD declares `letterform` as `xs:boolean`; Hancom uses it as a 0/1/2 enum. Mismatch in the standard itself vs. implementation. |
| `<checkBtn>` has attribute `command` | `section*.xml` | `command` is not declared on any button type in KS X 6101. Hancom form-action extension. |
| `<secPr>` has attribute `tabStop` | `section*.xml` | `tabStop` exists on `AbstractFormObjectType` (form controls), not on `SectionDefinitionType`. The sample puts it on a section — either a document defect or an extension. |
| `<item>` has attribute `isEmbedded` | `content.hpf` | Hancom marker for items stored inline in the ZIP. Not in OPF 2.0. |
| `<head>` contains `<trackchageConfig>` / missing `<trackchangeConfig>` | `header.xml` | Real typo in the sample document (`trackchage` vs `trackchange`). Not an extension — a defect. The validator correctly flags both the unexpected child and the missing required one. |

These are stable: regenerating the schema won't change them, and we
deliberately keep them visible (the Schema axis exists to surface
drift — folding extensions into the model defeats the point).

## Codegen bugs found and fixed during this probe

Both surfaced as false positives on real samples. Both are landed
as of commit `9b9ccd3`.

### 1. `FillBrushType` ↔ `fillBrushType` case mismatch

**Symptom.** `<fillBrush>` reported `cannot contain any child
elements, but found <winBrush>` on every use.

**Cause.** KS X 6101 is internally inconsistent. The type is
defined at `HWPMLCoreSchema.xsd:639` as
`<xs:complexType name="FillBrushType">` (capital `F`), but every
reference site (`HWPMLHeaderSchema.xsd:431`,
`HWPMLParaListSchema.xsd:865`, `HWPMLParaListSchema.xsd:2150`)
writes `type="hh:fillBrushType"` with a lowercase `f`. XSD type
names are case-sensitive — a strict processor would reject the
standard itself.

**Fix.** `tools/gen-owpml/src/main.rs::canonical_type_key` falls
back to a case-insensitive match if the exact key isn't found.
Scanning the full corpus turned up exactly one such mismatch
(`FillBrushType`), so the fallback isn't masking any other drift.

### 2. Outer-group `maxOccurs` not propagated to inner elements

**Symptom.** `<ctrl> appears 2 times under <run>, max 1`,
`<tbl> appears 2 times under <run>, max 1`, `<fwSpace> appears 2
times under <t>, max 1`, etc.

**Cause.** Many OWPML types wrap their children in
`<xs:choice minOccurs="0" maxOccurs="unbounded">` while the inner
elements declare `maxOccurs="1"`. In XSD semantics the outer
group's `maxOccurs` multiplies with the inner's. Our codegen took
the inner `maxOccurs` verbatim and ignored the wrapper, so legal
repeated children were reported as over-the-limit.

**Fix.** `collect_children` now takes an `outer_max` parameter and
propagates it via a new `combine_max` helper (`None` / unbounded
absorbs; otherwise `child_max × outer_max`). Nested group
modifiers (`xs:sequence` / `xs:choice` / `xs:all`) compound their
own `maxOccurs` on top of the outer.

## Golden fixture findings

The 44 fixtures under `testdata/golden/` are synthetic — built from
an in-Rust template at `crates/polaris-rhwpdvc-core/tests/support/mod.rs`.
Those live in the repo and ship to the web demo's preset list, so
they should be as schema-clean as the standard allows. One cleanup
pass (commit `4a0*` series) dropped pervasive Hancom-extension
attributes from the template and added the KS X 6101 required
children that were missing:

- Dropped from `<hh:borderFill>`: `slash`, `backSlash`,
  `crookedSlash`, `isCounterSlash`, `isCounterBackSlash` attributes
  (the standard carries that info in `<hh:slash>` / `<hh:backSlash>`
  child elements, which the template already emitted).
- Dropped the non-standard `<hh:layoutCompatibility textWrap=… …/>`
  attribute form in favour of the empty element the XSD declares.
- Fixed `compatibleDocument@targetProgram` from `"HWP2018"` →
  `"HWP201X"` (the XSD enum).
- Added the required `<hh:trackchangeConfig/>` under `<hh:head>`
  and `<hh:autoSpacing/>` under `<hh:paraPr>`.
- Removed Hancom extension attrs from `<hp:secPr>` (`tabStop`),
  `<hp:subList>` (`padding`, `lang`), and `<hh:bullet>` (`useInstWidth`,
  `autoIndent`, `widthAdjust`, …); added the required `useImage`
  attr on bullet.
- Renamed `<hp:outside>` (not declared anywhere in KS X 6101) to
  `<hp:outMargin>` (the standard name for the same concept).
- Fixed `<hp:placement place="DOC_END"/>` → `"END_OF_DOCUMENT"` under
  endnote scope — the enum values differ from the footnote `<placement>`
  (which uses `EACH_COLUMN`).
- Dropped the optional `<hh:forbiddenWordList/>` entirely (it was
  empty and missing the required `itemCnt` attr).

After cleanup: **57 remaining findings across 43 cases**, down from
~1 100 originally. The residue falls into three buckets, all
deliberate:

1. **Hancom-extension attrs the engine needs**
   - `<hp:linesegarray>` under `<hp:p>` — carries layout cache the
     engine's page/line tracker reads. Present in every HWPX our
     template emits (44 cases). Removing it would break page-info
     rule tests.
   - `numberShape` attribute on `<hh:paraHead>` — two cases
     (`17_outlineshape_numtype_mismatch`,
     `18_paranumbullet_numshape_mismatch`) exercise rules that match
     on this Hancom attr; the parser reads it by name.

2. **Intentional rule violations inside test cases**
   - `<hh:fillBrush><hh:winBrush hatchStyle="NONE"…/></hh:fillBrush>`
     in case 25 — "NONE" isn't in the XSD hatchStyle enum, and that
     mismatch is itself the rule this case tests.
   - `numFormat="^1)"` / `numFormat="^2."` in cases 17/18 — same
     pattern: the format string intentionally diverges from the
     NumFormat enum to exercise the format rule.

3. **Scope-wrapper structural quirk**
   - Cases 38/39/40 (shape / footnote / endnote scope) and 09
     (hyperlink) put `<hp:shapeObject>` / `<hp:footnote>` /
     `<hp:endnote>` / `<hp:fieldBegin>` / `<hp:fieldEnd>` directly
     under `<hp:p>`, wrapping a `<hp:run>`. The XSD-correct nesting
     would be `<hp:run><hp:ctrl>…</hp:ctrl></hp:run>`, but our
     parser's scope-depth tracker reads these at whichever nesting
     level they appear. Restructuring the fixture would also require
     a parser change; for now it's noted as a known test-scaffold
     shape.

## What to do when a new finding appears on a real sample

1. Identify the exact element / attribute / enum involved.
2. Search the source XSDs under `standards/KSX6101_OWPML/` for it.
   - If **not found** → real divergence. Optionally add a row to
     the table above.
   - If **found but declared differently** → check whether our
     generated model reflects the XSD faithfully. If the model is
     missing the detail, it's a codegen bug — fix it in
     `tools/gen-owpml/` and regenerate.
3. Never whitelist a divergence by adding it to the hand-maintained
   `emit_content_hpf()` block — that block is OPF-only. The KSX
   XSDs are the single source of truth; the model should reflect
   them exactly.
