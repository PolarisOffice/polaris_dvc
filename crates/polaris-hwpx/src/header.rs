//! Parse `Contents/header.xml` into the CharPr / ParaPr / FaceName / Style
//! tables. Uses quick-xml's streaming API so large headers don't spike RAM.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::container::local_name;
use crate::types::{
    CharPr, FaceName, FontRef, Header, ParaPr, Shadow, Strikeout, Style, Underline,
};
use crate::HwpxError;

pub fn parse_header(xml: &str) -> Result<Header, HwpxError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut header = Header::default();
    let mut buf = Vec::new();

    // Context trackers
    let mut cur_face_lang: Option<String> = None;
    let mut cur_char: Option<CharPr> = None;
    let mut cur_para: Option<ParaPr> = None;

    loop {
        let event = reader.read_event_into(&mut buf);
        let is_self_closing = matches!(event, Ok(Event::Empty(_)));
        match event {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = local_name(e.name().as_ref());
                let attrs: Vec<(String, String)> = e
                    .attributes()
                    .flatten()
                    .map(|a| {
                        let k = local_name(a.key.as_ref());
                        let v = a
                            .decode_and_unescape_value(&reader)
                            .map(|c| c.into_owned())
                            .unwrap_or_default();
                        (k, v)
                    })
                    .collect();

                match name.as_str() {
                    "fontface" => {
                        // Attribute `lang` scopes nested <font> entries.
                        cur_face_lang = attr(&attrs, "lang");
                    }
                    "font" => {
                        let id = attr_u32(&attrs, "id").unwrap_or(0);
                        let face = attr(&attrs, "face").unwrap_or_default();
                        let lang = cur_face_lang.clone().unwrap_or_default();
                        header.face_names.push(FaceName { id, lang, face });
                    }
                    "charPr" => {
                        cur_char = Some(CharPr {
                            id: attr_u32(&attrs, "id").unwrap_or(0),
                            height: attr_u32(&attrs, "height").unwrap_or(0),
                            text_color: attr(&attrs, "textColor").unwrap_or_default(),
                            ..CharPr::default()
                        });
                    }
                    "fontRef" if cur_char.is_some() => {
                        let fr = FontRef {
                            hangul: attr_u32(&attrs, "hangul").unwrap_or(0),
                            latin: attr_u32(&attrs, "latin").unwrap_or(0),
                            hanja: attr_u32(&attrs, "hanja").unwrap_or(0),
                            japanese: attr_u32(&attrs, "japanese").unwrap_or(0),
                            other: attr_u32(&attrs, "other").unwrap_or(0),
                            symbol: attr_u32(&attrs, "symbol").unwrap_or(0),
                            user: attr_u32(&attrs, "user").unwrap_or(0),
                        };
                        if let Some(c) = cur_char.as_mut() {
                            c.font_ref = fr;
                        }
                    }
                    "bold" => {
                        if let Some(c) = cur_char.as_mut() {
                            c.bold = true;
                        }
                    }
                    "italic" => {
                        if let Some(c) = cur_char.as_mut() {
                            c.italic = true;
                        }
                    }
                    "outline" => {
                        if let Some(c) = cur_char.as_mut() {
                            c.outline = true;
                        }
                    }
                    "emboss" => {
                        if let Some(c) = cur_char.as_mut() {
                            c.emboss = true;
                        }
                    }
                    "engrave" => {
                        if let Some(c) = cur_char.as_mut() {
                            c.engrave = true;
                        }
                    }
                    "supscript" => {
                        if let Some(c) = cur_char.as_mut() {
                            c.supscript = true;
                        }
                    }
                    "subscript" => {
                        if let Some(c) = cur_char.as_mut() {
                            c.subscript = true;
                        }
                    }
                    "underline" if cur_char.is_some() => {
                        if let Some(c) = cur_char.as_mut() {
                            c.underline = Some(Underline {
                                kind: attr(&attrs, "type").unwrap_or_default(),
                                shape: attr(&attrs, "shape").unwrap_or_default(),
                                color: attr(&attrs, "color").unwrap_or_default(),
                            });
                        }
                    }
                    "strikeout" if cur_char.is_some() => {
                        if let Some(c) = cur_char.as_mut() {
                            c.strikeout = Some(Strikeout {
                                shape: attr(&attrs, "shape").unwrap_or_default(),
                                color: attr(&attrs, "color").unwrap_or_default(),
                            });
                        }
                    }
                    "shadow" if cur_char.is_some() => {
                        if let Some(c) = cur_char.as_mut() {
                            c.shadow = Some(Shadow {
                                kind: attr(&attrs, "type").unwrap_or_default(),
                                color: attr(&attrs, "color").unwrap_or_default(),
                                offset_x: attr_i32(&attrs, "offsetX").unwrap_or(0),
                                offset_y: attr_i32(&attrs, "offsetY").unwrap_or(0),
                            });
                        }
                    }
                    "paraPr" => {
                        cur_para = Some(ParaPr {
                            id: attr_u32(&attrs, "id").unwrap_or(0),
                            ..ParaPr::default()
                        });
                    }
                    "align" if cur_para.is_some() => {
                        if let Some(p) = cur_para.as_mut() {
                            p.align_horizontal = attr(&attrs, "horizontal").unwrap_or_default();
                        }
                    }
                    "lineSpacing" if cur_para.is_some() => {
                        if let Some(p) = cur_para.as_mut() {
                            p.line_spacing_type = attr(&attrs, "type").unwrap_or_default();
                            p.line_spacing_value = attr_f64(&attrs, "value").unwrap_or(0.0);
                        }
                    }
                    "style" => {
                        header.styles.push(Style {
                            id: attr_u32(&attrs, "id").unwrap_or(0),
                            name: attr(&attrs, "name").unwrap_or_default(),
                            kind: attr(&attrs, "type").unwrap_or_default(),
                            para_pr_id_ref: attr_u32(&attrs, "paraPrIDRef").unwrap_or(0),
                            char_pr_id_ref: attr_u32(&attrs, "charPrIDRef").unwrap_or(0),
                        });
                    }
                    _ => {}
                }

                // Self-closing charPr/paraPr: commit immediately since no End
                // event will be emitted for them.
                if is_self_closing {
                    match name.as_str() {
                        "charPr" => {
                            if let Some(c) = cur_char.take() {
                                header.char_shapes.push(c);
                            }
                        }
                        "paraPr" => {
                            if let Some(p) = cur_para.take() {
                                header.para_shapes.push(p);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = local_name(e.name().as_ref());
                match name.as_str() {
                    "fontface" => cur_face_lang = None,
                    "charPr" => {
                        if let Some(c) = cur_char.take() {
                            header.char_shapes.push(c);
                        }
                    }
                    "paraPr" => {
                        if let Some(p) = cur_para.take() {
                            header.para_shapes.push(p);
                        }
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

    // Flush anything still open (defensive; malformed input).
    if let Some(c) = cur_char.take() {
        header.char_shapes.push(c);
    }
    if let Some(p) = cur_para.take() {
        header.para_shapes.push(p);
    }

    Ok(header)
}

fn attr(attrs: &[(String, String)], key: &str) -> Option<String> {
    attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}
fn attr_u32(attrs: &[(String, String)], key: &str) -> Option<u32> {
    attr(attrs, key).and_then(|s| s.trim().parse().ok())
}
fn attr_i32(attrs: &[(String, String)], key: &str) -> Option<i32> {
    attr(attrs, key).and_then(|s| s.trim().parse().ok())
}
fn attr_f64(attrs: &[(String, String)], key: &str) -> Option<f64> {
    attr(attrs, key).and_then(|s| s.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r##"<?xml version="1.0"?>
<hh:head xmlns:hh="h" xmlns:hc="c">
  <hh:refList>
    <hh:fontfaces itemCnt="1">
      <hh:fontface lang="HANGUL" itemCnt="1">
        <hh:font id="0" face="함초롬바탕" type="TTF"/>
      </hh:fontface>
      <hh:fontface lang="LATIN" itemCnt="1">
        <hh:font id="0" face="Times New Roman" type="TTF"/>
      </hh:fontface>
    </hh:fontfaces>
    <hh:charProperties itemCnt="2">
      <hh:charPr id="0" height="1000" textColor="#000000">
        <hh:fontRef hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/>
        <hh:bold/>
      </hh:charPr>
      <hh:charPr id="1" height="1200" textColor="#FF0000">
        <hh:fontRef hangul="0" latin="0"/>
        <hh:italic/>
        <hh:underline type="BOTTOM" shape="SOLID" color="#000000"/>
      </hh:charPr>
    </hh:charProperties>
    <hh:paraProperties itemCnt="1">
      <hh:paraPr id="0">
        <hh:align horizontal="JUSTIFY" vertical="BASELINE"/>
        <hh:lineSpacing type="PERCENT" value="160"/>
      </hh:paraPr>
    </hh:paraProperties>
    <hh:styles itemCnt="1">
      <hh:style id="0" type="PARA" name="바탕글" paraPrIDRef="0" charPrIDRef="0"/>
    </hh:styles>
  </hh:refList>
</hh:head>"##;

    #[test]
    fn parses_char_shapes_with_booleans_and_underline() {
        let h = parse_header(SAMPLE).unwrap();
        assert_eq!(h.char_shapes.len(), 2);
        let c0 = &h.char_shapes[0];
        assert_eq!(c0.id, 0);
        assert_eq!(c0.height, 1000);
        assert!(c0.bold);
        assert!(!c0.italic);
        let c1 = &h.char_shapes[1];
        assert_eq!(c1.id, 1);
        assert!(c1.italic);
        assert_eq!(c1.underline.as_ref().unwrap().kind, "BOTTOM");
    }

    #[test]
    fn parses_para_shapes() {
        let h = parse_header(SAMPLE).unwrap();
        assert_eq!(h.para_shapes.len(), 1);
        let p = &h.para_shapes[0];
        assert_eq!(p.align_horizontal, "JUSTIFY");
        assert_eq!(p.line_spacing_type, "PERCENT");
        assert_eq!(p.line_spacing_value, 160.0);
    }

    #[test]
    fn parses_face_names_per_language() {
        let h = parse_header(SAMPLE).unwrap();
        assert_eq!(h.face_names.len(), 2);
        let hangul = h.face_name(0, "HANGUL").unwrap();
        assert_eq!(hangul.face, "함초롬바탕");
        let latin = h.face_name(0, "LATIN").unwrap();
        assert_eq!(latin.face, "Times New Roman");
    }

    #[test]
    fn parses_styles() {
        let h = parse_header(SAMPLE).unwrap();
        assert_eq!(h.styles.len(), 1);
        assert_eq!(h.styles[0].name, "바탕글");
    }
}
