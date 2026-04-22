//! Integration tests — validator against the bootstrap schemas.

use polaris_rhwpdvc_schema::{validate_xml, OwpmlRoot, ViolationCode};

#[test]
fn minimal_head_passes() {
    // Under the real KS X 6101 HEAD schema, <head> requires:
    //   - attributes: version, secCnt
    //   - children (min 1 each): refList, beginNum, trackchangeConfig
    // We include just enough of each to clear the minimum bar.
    let xml = br#"<?xml version="1.0"?>
<hh:head xmlns:hh="h" version="1.0" secCnt="1">
  <hh:beginNum page="1" footnote="1" endnote="1" pic="1" tbl="1" equation="1"/>
  <hh:refList/>
  <hh:trackchangeConfig/>
</hh:head>"#;
    let violations = validate_xml(xml, OwpmlRoot::Head);
    assert!(
        violations.is_empty(),
        "expected no violations, got: {violations:?}"
    );
}

#[test]
fn unknown_attribute_on_char_pr_fires() {
    let xml = r#"<?xml version="1.0"?>
<hh:head xmlns:hh="h">
  <hh:refList>
    <hh:charProperties>
      <hh:charPr id="0" NOT_A_THING="bogus" height="1000">
        <hh:fontRef hangul="0"/>
      </hh:charPr>
    </hh:charProperties>
  </hh:refList>
</hh:head>"#
        .as_bytes();
    let violations = validate_xml(xml, OwpmlRoot::Head);
    assert!(
        violations
            .iter()
            .any(|v| v.code == ViolationCode::UnknownAttribute && v.element == "charPr"),
        "expected UnknownAttribute on charPr, got: {violations:?}"
    );
}

#[test]
fn enum_value_out_of_range_fires() {
    // fontface lang is an enum — "WRONG" isn't in the allowed set.
    let xml = r#"<?xml version="1.0"?>
<hh:head xmlns:hh="h">
  <hh:refList>
    <hh:fontfaces>
      <hh:fontface lang="WRONG">
        <hh:font id="0" face="barum"/>
      </hh:fontface>
    </hh:fontfaces>
  </hh:refList>
</hh:head>"#
        .as_bytes();
    let violations = validate_xml(xml, OwpmlRoot::Head);
    assert!(
        violations
            .iter()
            .any(|v| v.code == ViolationCode::AttributeTypeMismatch
                && v.attribute.as_deref() == Some("lang")),
        "expected AttributeTypeMismatch on fontface@lang, got: {violations:?}"
    );
}

#[test]
fn missing_required_attribute_fires() {
    // <hh:font> requires both id and face.
    let xml = r#"<?xml version="1.0"?>
<hh:head xmlns:hh="h">
  <hh:refList>
    <hh:fontfaces>
      <hh:fontface lang="HANGUL">
        <hh:font id="0"/>
      </hh:fontface>
    </hh:fontfaces>
  </hh:refList>
</hh:head>"#
        .as_bytes();
    let violations = validate_xml(xml, OwpmlRoot::Head);
    assert!(
        violations
            .iter()
            .any(|v| v.code == ViolationCode::MissingRequiredAttribute
                && v.attribute.as_deref() == Some("face")),
        "expected MissingRequiredAttribute for face, got: {violations:?}"
    );
}

#[test]
fn integer_type_mismatch_fires() {
    // charPr@id must be UnsignedInteger.
    let xml = r#"<?xml version="1.0"?>
<hh:head xmlns:hh="h">
  <hh:refList>
    <hh:charProperties>
      <hh:charPr id="not-a-number" height="1000">
        <hh:fontRef hangul="0"/>
      </hh:charPr>
    </hh:charProperties>
  </hh:refList>
</hh:head>"#
        .as_bytes();
    let violations = validate_xml(xml, OwpmlRoot::Head);
    assert!(
        violations
            .iter()
            .any(|v| v.code == ViolationCode::AttributeTypeMismatch
                && v.attribute.as_deref() == Some("id")),
        "expected AttributeTypeMismatch on charPr@id, got: {violations:?}"
    );
}

#[test]
fn section_run_with_text_passes() {
    // Text node "hello" inside <hp:t> — schema allows text on <t>.
    let xml = br#"<?xml version="1.0"?>
<hs:sec xmlns:hs="s" xmlns:hp="p">
  <hp:p id="0" paraPrIDRef="0">
    <hp:run charPrIDRef="0"><hp:t>hello</hp:t></hp:run>
  </hp:p>
</hs:sec>"#;
    let violations = validate_xml(xml, OwpmlRoot::Section);
    assert!(
        violations.is_empty(),
        "expected no violations, got: {violations:?}"
    );
}
