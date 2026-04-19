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
use crate::types::{LineSeg, Paragraph, Run, Section};
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
            Ok(Event::Text(t)) => {
                if in_text {
                    if let Some(r) = cur_run.as_mut() {
                        if let Ok(s) = t.unescape() {
                            r.text.push_str(&s);
                        }
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
}
