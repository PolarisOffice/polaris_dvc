//! Parse a `Contents/sectionN.xml` file into paragraphs + runs.
//!
//! Scope for now:
//! - `<hp:p id paraPrIDRef styleIDRef>` paragraphs.
//! - `<hp:run charPrIDRef>` children, accumulating `<hp:t>` text into a
//!   single per-run string. `<hp:ctrl>` and other control objects are
//!   skipped — they'll be handled by their own validators later.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::container::local_name;
use crate::types::{LineSeg, Paragraph, Run, Section, Table};
use crate::HwpxError;

pub fn parse_section(xml: &str) -> Result<Section, HwpxError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(false);

    let mut section = Section::default();
    let mut buf = Vec::new();

    let mut cur_para: Option<Paragraph> = None;
    let mut cur_run: Option<Run> = None;
    let mut in_text = false;
    // LIFO stack of open field scopes. Each `<hp:fieldBegin>` pushes its
    // `type` attribute; each `<hp:fieldEnd>` pops. A run is hyperlinked
    // when any entry on the stack is "HYPERLINK".
    let mut field_stack: Vec<String> = Vec::new();
    let is_in_hyperlink =
        |stack: &[String]| stack.iter().any(|t| t.eq_ignore_ascii_case("HYPERLINK"));
    // Depth of open `<hp:tbl>` elements. Increments on Start/Empty, decrements
    // on End. Used to assign `nesting_depth` so a table inside another table
    // (upstream `isInTableInTable`) is flagged.
    let mut table_depth: u32 = 0;

    loop {
        let event = reader.read_event_into(&mut buf);
        let is_self_closing = matches!(event, Ok(Event::Empty(_)));
        match event {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = local_name(e.name().as_ref());
                match name.as_str() {
                    "p" => {
                        let mut p = Paragraph::default();
                        for attr in e.attributes().flatten() {
                            let k = local_name(attr.key.as_ref());
                            let v = attr
                                .decode_and_unescape_value(&reader)
                                .map(|c| c.into_owned())
                                .unwrap_or_default();
                            match k.as_str() {
                                "id" => p.id = v.parse().unwrap_or(0),
                                "paraPrIDRef" => p.para_pr_id_ref = v.parse().unwrap_or(0),
                                "styleIDRef" => p.style_id_ref = v.parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                        cur_para = Some(p);
                        if is_self_closing {
                            if let Some(p) = cur_para.take() {
                                section.paragraphs.push(p);
                            }
                        }
                    }
                    "run" => {
                        let mut r = Run {
                            is_hyperlink: is_in_hyperlink(&field_stack),
                            ..Run::default()
                        };
                        for attr in e.attributes().flatten() {
                            let k = local_name(attr.key.as_ref());
                            if k == "charPrIDRef" {
                                let v = attr
                                    .decode_and_unescape_value(&reader)
                                    .map(|c| c.into_owned())
                                    .unwrap_or_default();
                                r.char_pr_id_ref = v.parse().unwrap_or(0);
                            }
                        }
                        cur_run = Some(r);
                        if is_self_closing {
                            commit_run(&mut cur_run, &mut cur_para);
                        }
                    }
                    "fieldBegin" => {
                        let mut kind = String::new();
                        for attr in e.attributes().flatten() {
                            if local_name(attr.key.as_ref()) == "type" {
                                kind = attr
                                    .decode_and_unescape_value(&reader)
                                    .map(|c| c.into_owned())
                                    .unwrap_or_default();
                            }
                        }
                        field_stack.push(kind);
                    }
                    "fieldEnd" => {
                        field_stack.pop();
                    }
                    "tbl" => {
                        // `nesting_depth` mirrors upstream `isInTableInTable`:
                        // a fresh `<hp:tbl>` sitting inside another active
                        // `<hp:tbl>` gets depth ≥ 1.
                        let mut t = Table {
                            nesting_depth: table_depth,
                            ..Table::default()
                        };
                        for attr in e.attributes().flatten() {
                            let k = local_name(attr.key.as_ref());
                            let v = attr
                                .decode_and_unescape_value(&reader)
                                .map(|c| c.into_owned())
                                .unwrap_or_default();
                            match k.as_str() {
                                "id" => t.id = v.parse().unwrap_or(0),
                                "borderFillIDRef" => t.border_fill_id_ref = v.parse().unwrap_or(0),
                                "rowCnt" => t.row_cnt = v.parse().unwrap_or(0),
                                "colCnt" => t.col_cnt = v.parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                        section.tables.push(t);
                        if !is_self_closing {
                            table_depth = table_depth.saturating_add(1);
                        }
                    }
                    "t" => {
                        in_text = !is_self_closing;
                    }
                    "lineseg" => {
                        if let Some(p) = cur_para.as_mut() {
                            let mut seg = LineSeg::default();
                            for attr in e.attributes().flatten() {
                                let k = local_name(attr.key.as_ref());
                                let v = attr
                                    .decode_and_unescape_value(&reader)
                                    .map(|c| c.into_owned())
                                    .unwrap_or_default();
                                match k.as_str() {
                                    "textpos" => seg.text_pos = v.parse().unwrap_or(0),
                                    "vertpos" => seg.vert_pos = v.parse().unwrap_or(0),
                                    "vertsize" => seg.vert_size = v.parse().unwrap_or(0),
                                    "horzpos" => seg.horz_pos = v.parse().unwrap_or(0),
                                    "horzsize" => seg.horz_size = v.parse().unwrap_or(0),
                                    _ => {}
                                }
                            }
                            p.line_segs.push(seg);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) if in_text => {
                if let Some(r) = cur_run.as_mut() {
                    if let Ok(s) = t.unescape() {
                        r.text.push_str(&s);
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = local_name(e.name().as_ref());
                match name.as_str() {
                    "p" => {
                        if let Some(p) = cur_para.take() {
                            section.paragraphs.push(p);
                        }
                    }
                    "run" => {
                        commit_run(&mut cur_run, &mut cur_para);
                    }
                    "t" => {
                        in_text = false;
                    }
                    "tbl" => {
                        table_depth = table_depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok(section)
}

fn commit_run(cur_run: &mut Option<Run>, cur_para: &mut Option<Paragraph>) {
    if let (Some(r), Some(p)) = (cur_run.take(), cur_para.as_mut()) {
        p.runs.push(r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0"?>
<hs:sec xmlns:hs="s" xmlns:hp="p">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>Hello</hp:t>
    </hp:run>
    <hp:run charPrIDRef="1">
      <hp:t> world</hp:t>
    </hp:run>
  </hp:p>
  <hp:p id="1" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>second paragraph</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"#;

    #[test]
    fn parses_paragraphs_and_runs_with_text() {
        let s = parse_section(SAMPLE).unwrap();
        assert_eq!(s.paragraphs.len(), 2);
        let p0 = &s.paragraphs[0];
        assert_eq!(p0.runs.len(), 2);
        assert_eq!(p0.runs[0].char_pr_id_ref, 0);
        assert_eq!(p0.runs[0].text, "Hello");
        assert_eq!(p0.runs[1].char_pr_id_ref, 1);
        assert_eq!(p0.runs[1].text, " world");
        assert_eq!(s.paragraphs[1].runs[0].text, "second paragraph");
    }

    #[test]
    fn preserves_paragraph_ids() {
        let s = parse_section(SAMPLE).unwrap();
        assert_eq!(s.paragraphs[0].id, 0);
        assert_eq!(s.paragraphs[1].id, 1);
    }

    #[test]
    fn extracts_table_with_border_fill_id_ref() {
        let xml = r#"<?xml version="1.0"?>
<hs:sec xmlns:hs="s" xmlns:hp="p">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:tbl id="42" borderFillIDRef="3" rowCnt="2" colCnt="3">
        <hp:tr><hp:tc></hp:tc></hp:tr>
      </hp:tbl>
    </hp:run>
  </hp:p>
</hs:sec>"#;
        let s = parse_section(xml).unwrap();
        assert_eq!(s.tables.len(), 1);
        let t = &s.tables[0];
        assert_eq!(t.id, 42);
        assert_eq!(t.border_fill_id_ref, 3);
        assert_eq!(t.row_cnt, 2);
        assert_eq!(t.col_cnt, 3);
        assert_eq!(t.nesting_depth, 0);
    }

    #[test]
    fn flags_table_in_table_via_nesting_depth() {
        let xml = r#"<?xml version="1.0"?>
<hs:sec xmlns:hs="s" xmlns:hp="p">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:tbl id="1" borderFillIDRef="1">
        <hp:tr><hp:tc>
          <hp:subList><hp:p id="1" paraPrIDRef="0" styleIDRef="0">
            <hp:run charPrIDRef="0">
              <hp:tbl id="2" borderFillIDRef="1"/>
            </hp:run>
          </hp:p></hp:subList>
        </hp:tc></hp:tr>
      </hp:tbl>
    </hp:run>
  </hp:p>
</hs:sec>"#;
        let s = parse_section(xml).unwrap();
        assert_eq!(s.tables.len(), 2);
        // Outer table at depth 0, inner at depth 1.
        assert_eq!(s.tables[0].nesting_depth, 0);
        assert_eq!(s.tables[1].nesting_depth, 1);
    }
}
