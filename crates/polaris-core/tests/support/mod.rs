//! Shared fixture builders for integration + golden tests.
//!
//! Produces realistic HWPX byte streams from declarative inputs. The XML
//! uses the real OWPML namespace URIs so future DVC.exe parity checks
//! against the same `doc.hwpx` can succeed without modification.

use std::io::{Cursor, Write};

use zip::write::FileOptions;
use zip::ZipWriter;

/// High-level fixture description — enough knobs to vary what the
/// resulting HWPX reports for the rule categories the engine covers today.
#[derive(Debug, Clone)]
pub struct Fixture {
    pub hangul_face: String,
    pub char_prs: Vec<FixCharPr>,
    pub para_prs: Vec<FixParaPr>,
    pub paragraphs: Vec<FixParagraph>,
}

#[derive(Debug, Clone)]
pub struct FixCharPr {
    pub id: u32,
    pub height: u32, // 1/100 pt (10pt → 1000)
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone)]
pub struct FixParaPr {
    pub id: u32,
    pub align: String, // e.g., "JUSTIFY"
    pub line_spacing_value: f64,
}

#[derive(Debug, Clone)]
pub struct FixParagraph {
    pub para_pr_id_ref: u32,
    pub runs: Vec<FixRun>,
}

#[derive(Debug, Clone)]
pub struct FixRun {
    pub char_pr_id_ref: u32,
    pub text: String,
}

impl Fixture {
    /// A one-paragraph, 바탕-10pt, non-bold document with single run "안녕".
    pub fn baseline() -> Self {
        Self {
            hangul_face: "바탕".into(),
            char_prs: vec![FixCharPr {
                id: 0,
                height: 1000,
                bold: false,
                italic: false,
            }],
            para_prs: vec![FixParaPr {
                id: 0,
                align: "JUSTIFY".into(),
                line_spacing_value: 160.0,
            }],
            paragraphs: vec![FixParagraph {
                para_pr_id_ref: 0,
                runs: vec![FixRun {
                    char_pr_id_ref: 0,
                    text: "안녕".into(),
                }],
            }],
        }
    }

    pub fn to_hwpx_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zip = ZipWriter::new(cursor);
            let opts = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

            zip.start_file("mimetype", opts).unwrap();
            zip.write_all(b"application/hwp+zip").unwrap();

            zip.start_file("META-INF/container.xml", opts).unwrap();
            zip.write_all(CONTAINER_XML.as_bytes()).unwrap();

            zip.start_file("Contents/content.hpf", opts).unwrap();
            zip.write_all(CONTENT_HPF.as_bytes()).unwrap();

            zip.start_file("Contents/header.xml", opts).unwrap();
            zip.write_all(self.header_xml().as_bytes()).unwrap();

            zip.start_file("Contents/section0.xml", opts).unwrap();
            zip.write_all(self.section_xml().as_bytes()).unwrap();

            zip.finish().unwrap();
        }
        buf
    }

    fn header_xml(&self) -> String {
        let mut s = String::new();
        s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n");
        s.push_str(
            "<hh:head xmlns:hh=\"http://www.hancom.co.kr/hwpml/2011/head\" \
             xmlns:hc=\"http://www.hancom.co.kr/hwpml/2011/core\" \
             xmlns:hp=\"http://www.hancom.co.kr/hwpml/2011/paragraph\" \
             version=\"1.31\" secCnt=\"1\">\n",
        );
        s.push_str("  <hh:refList>\n");
        s.push_str("    <hh:fontfaces itemCnt=\"1\">\n");
        s.push_str("      <hh:fontface lang=\"HANGUL\" itemCnt=\"1\">\n");
        s.push_str(&format!(
            "        <hh:font id=\"0\" face=\"{}\" type=\"TTF\"/>\n",
            self.hangul_face
        ));
        s.push_str("      </hh:fontface>\n");
        s.push_str("    </hh:fontfaces>\n");
        s.push_str(&format!(
            "    <hh:charProperties itemCnt=\"{}\">\n",
            self.char_prs.len()
        ));
        for c in &self.char_prs {
            s.push_str(&format!(
                "      <hh:charPr id=\"{}\" height=\"{}\" textColor=\"#000000\" shadeColor=\"none\">\n",
                c.id, c.height
            ));
            s.push_str("        <hh:fontRef hangul=\"0\" latin=\"0\" hanja=\"0\" japanese=\"0\" other=\"0\" symbol=\"0\" user=\"0\"/>\n");
            s.push_str("        <hh:ratio hangul=\"100\" latin=\"100\" hanja=\"100\" japanese=\"100\" other=\"100\" symbol=\"100\" user=\"100\"/>\n");
            if c.bold {
                s.push_str("        <hh:bold/>\n");
            }
            if c.italic {
                s.push_str("        <hh:italic/>\n");
            }
            s.push_str("      </hh:charPr>\n");
        }
        s.push_str("    </hh:charProperties>\n");
        s.push_str(&format!(
            "    <hh:paraProperties itemCnt=\"{}\">\n",
            self.para_prs.len()
        ));
        for p in &self.para_prs {
            s.push_str(&format!("      <hh:paraPr id=\"{}\">\n", p.id));
            s.push_str(&format!(
                "        <hh:align horizontal=\"{}\" vertical=\"BASELINE\"/>\n",
                p.align
            ));
            s.push_str(&format!(
                "        <hh:lineSpacing type=\"PERCENT\" value=\"{}\" unit=\"HWPUNIT\"/>\n",
                p.line_spacing_value
            ));
            s.push_str("      </hh:paraPr>\n");
        }
        s.push_str("    </hh:paraProperties>\n");
        s.push_str("  </hh:refList>\n");
        s.push_str("</hh:head>\n");
        s
    }

    fn section_xml(&self) -> String {
        let mut s = String::new();
        s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n");
        s.push_str(
            "<hs:sec xmlns:hs=\"http://www.hancom.co.kr/hwpml/2011/section\" \
             xmlns:hp=\"http://www.hancom.co.kr/hwpml/2011/paragraph\" \
             xmlns:hc=\"http://www.hancom.co.kr/hwpml/2011/core\">\n",
        );
        for (pi, p) in self.paragraphs.iter().enumerate() {
            s.push_str(&format!(
                "  <hp:p id=\"{}\" paraPrIDRef=\"{}\" styleIDRef=\"0\" pageBreak=\"0\" columnBreak=\"0\" merged=\"0\">\n",
                pi, p.para_pr_id_ref
            ));
            for r in &p.runs {
                s.push_str(&format!(
                    "    <hp:run charPrIDRef=\"{}\"><hp:t>{}</hp:t></hp:run>\n",
                    r.char_pr_id_ref,
                    xml_escape(&r.text)
                ));
            }
            s.push_str("  </hp:p>\n");
        }
        s.push_str("</hs:sec>\n");
        s
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<ocf:container xmlns:ocf="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
  <ocf:rootfiles>
    <ocf:rootfile full-path="Contents/content.hpf" media-type="application/hwpml-package+xml"/>
  </ocf:rootfiles>
</ocf:container>
"#;

const CONTENT_HPF: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf/" version="1.0" unique-identifier="hwpxUUID">
  <opf:manifest>
    <opf:item id="header" href="Contents/header.xml" media-type="application/xml"/>
    <opf:item id="sec0" href="Contents/section0.xml" media-type="application/xml"/>
  </opf:manifest>
  <opf:spine>
    <opf:itemref idref="sec0" linear="yes"/>
  </opf:spine>
</opf:package>
"#;
