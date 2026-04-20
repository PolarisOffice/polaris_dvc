//! End-to-end integration: synthetic HWPX bytes → parse → validate →
//! DVC-compatible JSON output. Exercises the full stack without relying
//! on filesystem or external tooling.

use std::io::{Cursor, Write};

use polaris_rhwpdvc_core::engine::{validate, EngineOptions};
use polaris_rhwpdvc_core::output::OutputOption;
use polaris_rhwpdvc_core::rules::schema::RuleSpec;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Build a minimal HWPX with two paragraphs:
///
/// - id=0, charPr 0 (font 바탕, 10pt, no bold), text "clean"
/// - id=1, charPr 1 (font 바탕, 12pt, bold), text "dirty"
///
/// Against a spec requiring 바탕 / 10pt / not bold, paragraph 1 triggers
/// two violations (fontsize 1001 + bold 1009) on the "dirty" run.
fn build_hwpx() -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut zip = ZipWriter::new(cursor);
        let opts = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file("mimetype", opts).unwrap();
        zip.write_all(b"application/hwp+zip").unwrap();

        zip.start_file("Contents/content.hpf", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf/" version="1.0">
  <opf:manifest>
    <opf:item id="header" href="Contents/header.xml" media-type="application/xml"/>
    <opf:item id="sec0" href="Contents/section0.xml" media-type="application/xml"/>
  </opf:manifest>
  <opf:spine>
    <opf:itemref idref="sec0"/>
  </opf:spine>
</opf:package>"#,
        )
        .unwrap();

        zip.start_file("Contents/header.xml", opts).unwrap();
        zip.write_all(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<hh:head xmlns:hh="h">
  <hh:refList>
    <hh:fontfaces>
      <hh:fontface lang="HANGUL">
        <hh:font id="0" face="바탕" type="TTF"/>
      </hh:fontface>
    </hh:fontfaces>
    <hh:charProperties itemCnt="2">
      <hh:charPr id="0" height="1000" textColor="#000000">
        <hh:fontRef hangul="0" latin="0"/>
      </hh:charPr>
      <hh:charPr id="1" height="1200" textColor="#000000">
        <hh:fontRef hangul="0" latin="0"/>
        <hh:bold/>
      </hh:charPr>
    </hh:charProperties>
    <hh:paraProperties itemCnt="1">
      <hh:paraPr id="0">
        <hh:align horizontal="JUSTIFY" vertical="BASELINE"/>
        <hh:lineSpacing type="PERCENT" value="160"/>
      </hh:paraPr>
    </hh:paraProperties>
  </hh:refList>
</hh:head>"##
                .as_bytes(),
        )
        .unwrap();

        zip.start_file("Contents/section0.xml", opts).unwrap();
        zip.write_all(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<hs:sec xmlns:hs=\"s\" xmlns:hp=\"p\">\n\
  <hp:p id=\"0\" paraPrIDRef=\"0\" styleIDRef=\"0\">\n\
    <hp:run charPrIDRef=\"0\"><hp:t>clean</hp:t></hp:run>\n\
    <hp:linesegarray><hp:lineseg textpos=\"0\" vertpos=\"0\" vertsize=\"1000\" horzpos=\"0\" horzsize=\"42520\"/></hp:linesegarray>\n\
  </hp:p>\n\
  <hp:p id=\"1\" paraPrIDRef=\"0\" styleIDRef=\"0\">\n\
    <hp:run charPrIDRef=\"1\"><hp:t>dirty</hp:t></hp:run>\n\
    <hp:linesegarray><hp:lineseg textpos=\"0\" vertpos=\"1200\" vertsize=\"1200\" horzpos=\"0\" horzsize=\"42520\"/></hp:linesegarray>\n\
  </hp:p>\n\
</hs:sec>"
                .as_bytes(),
        )
        .unwrap();

        zip.finish().unwrap();
    }
    buf
}

#[test]
fn full_pipeline_detects_expected_violations() {
    let bytes = build_hwpx();
    let doc = polaris_rhwpdvc_hwpx::open_bytes(&bytes).expect("parse synthetic HWPX");
    let spec: RuleSpec = serde_json::from_str(
        r#"{"charshape":{"font":"바탕","fontsize":10,"bold":false},
             "parashape":{"linespacingvalue":160}}"#,
    )
    .unwrap();

    let report = validate(&doc, &spec, &EngineOptions::default());
    assert_eq!(
        report.violations.len(),
        2,
        "violations: {:?}",
        report.violations
    );

    let codes: Vec<u32> = report
        .violations
        .iter()
        .map(|v| v.error_code.value())
        .collect();
    assert!(
        codes.contains(&1001),
        "expected 1001 fontsize, got {codes:?}"
    );
    assert!(codes.contains(&1009), "expected 1009 bold, got {codes:?}");

    // Every violation should carry the offending run's text, matching
    // upstream DVC semantics (errorText == document text at the site).
    for v in &report.violations {
        assert_eq!(v.text, "dirty");
        assert_eq!(v.char_pr_id_ref, 1);
        assert_eq!(v.para_pr_id_ref, 0);
    }
}

#[test]
fn full_pipeline_json_output_shape() {
    let bytes = build_hwpx();
    let doc = polaris_rhwpdvc_hwpx::open_bytes(&bytes).unwrap();
    let spec: RuleSpec = serde_json::from_str(r#"{"charshape":{"fontsize":10}}"#).unwrap();
    let report = validate(&doc, &spec, &EngineOptions::default());

    let json = report.to_json_value(OutputOption::AllOption);
    let arr = json.as_array().expect("top-level must be an array");
    assert_eq!(arr.len(), 1);
    let v = &arr[0];
    assert_eq!(v["CharIDRef"], 1);
    assert_eq!(v["ParaPrIDRef"], 0);
    assert_eq!(v["errorText"], "dirty");
    assert_eq!(v["ErrorCode"], 1001);
    // AllOption pulls in every conditional block.
    for k in [
        "TableID",
        "IsInTable",
        "UseStyle",
        "IsInShape",
        "UseHyperlink",
    ] {
        assert!(v.get(k).is_some(), "missing {k}");
    }
}

#[test]
fn clean_document_produces_empty_array() {
    let bytes = build_hwpx();
    let doc = polaris_rhwpdvc_hwpx::open_bytes(&bytes).unwrap();
    // Spec only constrains something that's already fine.
    let spec: RuleSpec = serde_json::from_str(r#"{"charshape":{"font":"바탕"}}"#).unwrap();
    let report = validate(&doc, &spec, &EngineOptions::default());
    assert!(report.violations.is_empty());
    let json = report.to_json_value(OutputOption::Default);
    assert_eq!(json.as_array().unwrap().len(), 0);
}
