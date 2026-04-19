//! Parse `Contents/header.xml` into the CharPr / ParaPr / FaceName / Style
//! tables. Uses quick-xml's streaming API so large headers don't spike RAM.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::container::local_name;
use crate::types::{
    Border, BorderFill, Bullet, CharPr, FaceName, Fill, FillBrush, FillGradation, FontRef, Header,
    Numbering, ParaHead, ParaPr, Shadow, Strikeout, Style, Underline,
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
    let mut cur_border_fill: Option<BorderFill> = None;
    let mut cur_numbering: Option<Numbering> = None;

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
                    "ratio" if cur_char.is_some() => {
                        if let Some(c) = cur_char.as_mut() {
                            c.ratio_hangul = attr_f64(&attrs, "hangul").unwrap_or(0.0);
                        }
                    }
                    "spacing" if cur_char.is_some() => {
                        if let Some(c) = cur_char.as_mut() {
                            c.spacing_hangul = attr_f64(&attrs, "hangul").unwrap_or(0.0);
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
                    // `<hh:margin>` children live under paraPr. `intent` is
                    // the first-line indent (positive=indent, negative=outdent);
                    // `prev`/`next` are the before/after spacing; `left` is
                    // the body indent. All expressed in HWPUNIT.
                    "intent" if cur_para.is_some() => {
                        if let Some(p) = cur_para.as_mut() {
                            p.margin_intent = attr_f64(&attrs, "value").unwrap_or(0.0);
                        }
                    }
                    "left" if cur_para.is_some() => {
                        if let Some(p) = cur_para.as_mut() {
                            p.margin_left = attr_f64(&attrs, "value").unwrap_or(0.0);
                        }
                    }
                    "prev" if cur_para.is_some() => {
                        if let Some(p) = cur_para.as_mut() {
                            p.margin_prev = attr_f64(&attrs, "value").unwrap_or(0.0);
                        }
                    }
                    "next" if cur_para.is_some() => {
                        if let Some(p) = cur_para.as_mut() {
                            p.margin_next = attr_f64(&attrs, "value").unwrap_or(0.0);
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
                    "borderFill" => {
                        cur_border_fill = Some(BorderFill {
                            id: attr_u32(&attrs, "id").unwrap_or(0),
                            ..BorderFill::default()
                        });
                    }
                    "numbering" => {
                        cur_numbering = Some(Numbering {
                            id: attr_u32(&attrs, "id").unwrap_or(0),
                            start: attr_u32(&attrs, "start").unwrap_or(0),
                            ..Numbering::default()
                        });
                    }
                    "paraHead" if cur_numbering.is_some() => {
                        // `<hh:paraHead level numFormat start>` — one row of
                        // the numbering level table. `numberShape` is a
                        // separate attribute mapped to upstream's `GetNumShape`.
                        let head = ParaHead {
                            level: attr_u32(&attrs, "level").unwrap_or(0),
                            start: attr_u32(&attrs, "start").unwrap_or(0),
                            num_format: attr(&attrs, "numFormat").unwrap_or_default(),
                            number_shape: attr_u32(&attrs, "numberShape").unwrap_or(0),
                        };
                        if let Some(n) = cur_numbering.as_mut() {
                            n.para_heads.push(head);
                        }
                    }
                    "bullet" => {
                        header.bullets.push(Bullet {
                            id: attr_u32(&attrs, "id").unwrap_or(0),
                            char_: attr(&attrs, "char").unwrap_or_default(),
                        });
                    }
                    "leftBorder" | "rightBorder" | "topBorder" | "bottomBorder"
                        if cur_border_fill.is_some() =>
                    {
                        let border = Border {
                            kind: attr(&attrs, "type").unwrap_or_default(),
                            width_mm: attr(&attrs, "width")
                                .map(|s| parse_width_mm(&s))
                                .unwrap_or(0.0),
                            color: attr(&attrs, "color").unwrap_or_default(),
                        };
                        if let Some(bf) = cur_border_fill.as_mut() {
                            match name.as_str() {
                                "leftBorder" => bf.left = border,
                                "rightBorder" => bf.right = border,
                                "topBorder" => bf.top = border,
                                "bottomBorder" => bf.bottom = border,
                                _ => unreachable!(),
                            }
                        }
                    }
                    // Fill sub-elements under <hh:borderFill>. Upstream writes
                    // at most one of winBrush / gradation / imgBrush per
                    // borderFill, so we take the first we see (Fill::None
                    // default → set once, subsequent ones ignored).
                    "winBrush" if cur_border_fill.is_some() => {
                        let brush = FillBrush {
                            face_color: attr(&attrs, "faceColor").unwrap_or_default(),
                            hatch_color: attr(&attrs, "hatchColor").unwrap_or_default(),
                            hatch_style: attr(&attrs, "hatchStyle").unwrap_or_default(),
                            alpha: attr_u32(&attrs, "alpha").unwrap_or(0),
                        };
                        if let Some(bf) = cur_border_fill.as_mut() {
                            if matches!(bf.fill, Fill::None) {
                                bf.fill = Fill::Brush(brush);
                            }
                        }
                    }
                    "gradation" if cur_border_fill.is_some() => {
                        let grad = FillGradation {
                            kind: attr(&attrs, "type").unwrap_or_default(),
                            angle: attr_i32(&attrs, "angle").unwrap_or(0),
                            center_x: attr_i32(&attrs, "centerX").unwrap_or(0),
                            center_y: attr_i32(&attrs, "centerY").unwrap_or(0),
                            colors: Vec::new(),
                        };
                        if let Some(bf) = cur_border_fill.as_mut() {
                            if matches!(bf.fill, Fill::None) {
                                bf.fill = Fill::Gradation(grad);
                            }
                        }
                    }
                    "imgBrush" if cur_border_fill.is_some() => {
                        if let Some(bf) = cur_border_fill.as_mut() {
                            if matches!(bf.fill, Fill::None) {
                                bf.fill = Fill::Image;
                            }
                        }
                    }
                    _ => {}
                }

                // Self-closing charPr/paraPr/borderFill: commit immediately
                // since no End event will be emitted for them.
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
                        "borderFill" => {
                            if let Some(bf) = cur_border_fill.take() {
                                header.border_fills.push(bf);
                            }
                        }
                        "numbering" => {
                            if let Some(n) = cur_numbering.take() {
                                header.numberings.push(n);
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
                    "borderFill" => {
                        if let Some(bf) = cur_border_fill.take() {
                            header.border_fills.push(bf);
                        }
                    }
                    "numbering" => {
                        if let Some(n) = cur_numbering.take() {
                            header.numberings.push(n);
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
    if let Some(bf) = cur_border_fill.take() {
        header.border_fills.push(bf);
    }
    if let Some(n) = cur_numbering.take() {
        header.numberings.push(n);
    }

    Ok(header)
}

fn attr(attrs: &[(String, String)], key: &str) -> Option<String> {
    attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

/// Parse HWPX width attributes like "0.12 mm" into a plain float in
/// millimeters. Tolerates missing unit suffix and extra whitespace.
fn parse_width_mm(s: &str) -> f64 {
    let trimmed = s.trim().trim_end_matches("mm").trim();
    trimmed.parse().unwrap_or(0.0)
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

    #[test]
    fn parses_numberings_and_bullets() {
        let xml = r##"<?xml version="1.0"?>
<hh:head xmlns:hh="h">
  <hh:refList>
    <hh:numberings itemCnt="1">
      <hh:numbering id="3" start="1">
        <hh:paraHead level="1" start="1" numFormat="^1." numberShape="0"/>
        <hh:paraHead level="2" start="1" numFormat="^2." numberShape="8"/>
      </hh:numbering>
    </hh:numberings>
    <hh:bullets itemCnt="2">
      <hh:bullet id="1" char="□"/>
      <hh:bullet id="2" char="★"/>
    </hh:bullets>
  </hh:refList>
</hh:head>"##;
        let h = parse_header(xml).unwrap();
        assert_eq!(h.numberings.len(), 1);
        let n = &h.numberings[0];
        assert_eq!(n.id, 3);
        assert_eq!(n.para_heads.len(), 2);
        assert_eq!(n.para_heads[0].level, 1);
        assert_eq!(n.para_heads[0].num_format, "^1.");
        assert_eq!(n.para_heads[1].num_format, "^2.");
        assert_eq!(n.para_heads[1].number_shape, 8);
        assert_eq!(h.bullets.len(), 2);
        assert_eq!(h.bullets[0].char_, "□");
        assert_eq!(h.bullets[1].char_, "★");
    }

    #[test]
    fn parses_border_fills() {
        let xml = r##"<?xml version="1.0"?>
<hh:head xmlns:hh="h">
  <hh:refList>
    <hh:borderFills itemCnt="1">
      <hh:borderFill id="1">
        <hh:leftBorder type="SOLID" width="0.12 mm" color="#000000"/>
        <hh:rightBorder type="SOLID" width="0.12 mm" color="#000000"/>
        <hh:topBorder type="DASH" width="0.40 mm" color="#FF0000"/>
        <hh:bottomBorder type="SOLID" width="0.12 mm" color="#000000"/>
      </hh:borderFill>
    </hh:borderFills>
  </hh:refList>
</hh:head>"##;
        let h = parse_header(xml).unwrap();
        assert_eq!(h.border_fills.len(), 1);
        let bf = &h.border_fills[0];
        assert_eq!(bf.id, 1);
        assert_eq!(bf.top.kind, "DASH");
        assert_eq!(bf.top.width_mm, 0.40);
        assert_eq!(bf.top.color, "#FF0000");
        assert_eq!(bf.bottom.kind, "SOLID");
        assert_eq!(bf.bottom.width_mm, 0.12);
    }
}
