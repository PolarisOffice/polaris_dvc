//! Shared fixture builders for integration + golden tests.
//!
//! Produces viewer-openable HWPX byte streams from declarative inputs. The
//! layout, file set, and XML namespace declarations mirror what a real
//! HWPX (e.g., a file produced by Hancom Office) contains, cross-checked
//! against the Apache-2.0 samples in neolord0/hwpxlib `testFile/
//! reader_writer/`.
//!
//! Minimal but spec-compliant file set:
//!
//! - `mimetype` (first, STORED, "application/hwp+zip")
//! - `version.xml` (HCFVersion)
//! - `settings.xml` (HWPApplicationSetting)
//! - `META-INF/container.xml` (OCF, points to content.hpf + PrvText)
//! - `META-INF/manifest.xml` (ODF manifest)
//! - `Preview/PrvText.txt`
//! - `Contents/content.hpf` (OPF with metadata + manifest + spine)
//! - `Contents/header.xml` (full namespace + beginNum + refList)
//! - `Contents/section0.xml` (full namespace + secPr in first run)

use std::io::{Cursor, Write};

use zip::write::FileOptions;
use zip::ZipWriter;

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
    /// Baseline: one paragraph, 바탕 Hangul face at 10pt, non-bold, "안녕".
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
            let stored = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

            // mimetype must be first and uncompressed.
            zip.start_file("mimetype", stored).unwrap();
            zip.write_all(MIMETYPE.as_bytes()).unwrap();

            zip.start_file("version.xml", stored).unwrap();
            zip.write_all(VERSION_XML.as_bytes()).unwrap();

            zip.start_file("settings.xml", stored).unwrap();
            zip.write_all(SETTINGS_XML.as_bytes()).unwrap();

            zip.start_file("META-INF/container.xml", stored).unwrap();
            zip.write_all(CONTAINER_XML.as_bytes()).unwrap();

            zip.start_file("META-INF/manifest.xml", stored).unwrap();
            zip.write_all(MANIFEST_XML.as_bytes()).unwrap();

            zip.start_file("Preview/PrvText.txt", stored).unwrap();
            zip.write_all(self.preview_text().as_bytes()).unwrap();

            zip.start_file("Contents/content.hpf", stored).unwrap();
            zip.write_all(CONTENT_HPF.as_bytes()).unwrap();

            zip.start_file("Contents/header.xml", stored).unwrap();
            zip.write_all(self.header_xml().as_bytes()).unwrap();

            zip.start_file("Contents/section0.xml", stored).unwrap();
            zip.write_all(self.section_xml().as_bytes()).unwrap();

            zip.finish().unwrap();
        }
        buf
    }

    fn preview_text(&self) -> String {
        self.paragraphs
            .iter()
            .flat_map(|p| p.runs.iter().map(|r| r.text.as_str()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn header_xml(&self) -> String {
        let mut s = String::new();
        s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        s.push_str("<hh:head ");
        s.push_str(HWPX_NAMESPACES);
        s.push_str(" version=\"1.4\" secCnt=\"1\">");
        s.push_str(
            "<hh:beginNum page=\"1\" footnote=\"1\" endnote=\"1\" pic=\"1\" \
             tbl=\"1\" equation=\"1\"/>",
        );
        s.push_str("<hh:refList>");
        // fontfaces covering the 7 language categories referenced by <hh:fontRef>.
        s.push_str("<hh:fontfaces itemCnt=\"7\">");
        for lang in FONT_LANGS {
            s.push_str(&format!(
                "<hh:fontface lang=\"{}\" fontCnt=\"1\">\
                 <hh:font id=\"0\" face=\"{}\" type=\"TTF\" isEmbedded=\"0\"/>\
                 </hh:fontface>",
                lang,
                xml_escape(&self.hangul_face)
            ));
        }
        s.push_str("</hh:fontfaces>");
        // Minimal borderFills — real documents have one; a viewer expects the
        // itemCnt to match the children count.
        s.push_str(
            "<hh:borderFills itemCnt=\"1\">\
             <hh:borderFill id=\"1\" threeD=\"0\" shadow=\"0\" slash=\"NONE\" \
             backSlash=\"NONE\" crookedSlash=\"0\" isCounterSlash=\"0\" \
             isCounterBackSlash=\"0\">\
             <hh:slash type=\"NONE\" Crooked=\"0\" isCounter=\"0\"/>\
             <hh:backSlash type=\"NONE\" Crooked=\"0\" isCounter=\"0\"/>\
             <hh:leftBorder type=\"SOLID\" width=\"0.12 mm\" color=\"#000000\"/>\
             <hh:rightBorder type=\"SOLID\" width=\"0.12 mm\" color=\"#000000\"/>\
             <hh:topBorder type=\"SOLID\" width=\"0.12 mm\" color=\"#000000\"/>\
             <hh:bottomBorder type=\"SOLID\" width=\"0.12 mm\" color=\"#000000\"/>\
             <hh:diagonal type=\"SOLID\" width=\"0.12 mm\" color=\"#000000\"/>\
             </hh:borderFill>\
             </hh:borderFills>",
        );
        // charProperties
        s.push_str(&format!(
            "<hh:charProperties itemCnt=\"{}\">",
            self.char_prs.len()
        ));
        for c in &self.char_prs {
            s.push_str(&format!(
                "<hh:charPr id=\"{}\" height=\"{}\" textColor=\"#000000\" \
                 shadeColor=\"none\" useFontSpace=\"0\" useKerning=\"0\" \
                 symMark=\"NONE\" borderFillIDRef=\"1\">\
                 <hh:fontRef hangul=\"0\" latin=\"0\" hanja=\"0\" japanese=\"0\" \
                 other=\"0\" symbol=\"0\" user=\"0\"/>\
                 <hh:ratio hangul=\"100\" latin=\"100\" hanja=\"100\" \
                 japanese=\"100\" other=\"100\" symbol=\"100\" user=\"100\"/>\
                 <hh:spacing hangul=\"0\" latin=\"0\" hanja=\"0\" japanese=\"0\" \
                 other=\"0\" symbol=\"0\" user=\"0\"/>\
                 <hh:relSz hangul=\"100\" latin=\"100\" hanja=\"100\" \
                 japanese=\"100\" other=\"100\" symbol=\"100\" user=\"100\"/>\
                 <hh:offset hangul=\"0\" latin=\"0\" hanja=\"0\" japanese=\"0\" \
                 other=\"0\" symbol=\"0\" user=\"0\"/>",
                c.id, c.height
            ));
            if c.bold {
                s.push_str("<hh:bold/>");
            }
            if c.italic {
                s.push_str("<hh:italic/>");
            }
            s.push_str(
                "<hh:underline type=\"NONE\" shape=\"SOLID\" color=\"#000000\"/>\
                 <hh:strikeout shape=\"NONE\" color=\"#000000\"/>\
                 <hh:outline type=\"NONE\"/>\
                 <hh:shadow type=\"NONE\" color=\"#C0C0C0\" offsetX=\"10\" \
                 offsetY=\"10\"/>\
                 </hh:charPr>",
            );
        }
        s.push_str("</hh:charProperties>");
        // paraProperties
        s.push_str(&format!(
            "<hh:paraProperties itemCnt=\"{}\">",
            self.para_prs.len()
        ));
        for p in &self.para_prs {
            s.push_str(&format!(
                "<hh:paraPr id=\"{}\" tabPrIDRef=\"0\" condense=\"0\" \
                 fontLineHeight=\"0\" snapToGrid=\"1\" suppressLineNumbers=\"0\" \
                 checked=\"0\">\
                 <hh:align horizontal=\"{}\" vertical=\"BASELINE\"/>\
                 <hh:heading type=\"NONE\" idRef=\"0\" level=\"0\"/>\
                 <hh:breakSetting breakLatinWord=\"KEEP_WORD\" \
                 breakNonLatinWord=\"KEEP_WORD\" widowOrphan=\"0\" \
                 keepWithNext=\"0\" keepLines=\"0\" pageBreakBefore=\"0\" \
                 lineWrap=\"BREAK\"/>\
                 <hh:margin>\
                 <hc:intent value=\"0\" unit=\"HWPUNIT\"/>\
                 <hc:left value=\"0\" unit=\"HWPUNIT\"/>\
                 <hc:right value=\"0\" unit=\"HWPUNIT\"/>\
                 <hc:prev value=\"0\" unit=\"HWPUNIT\"/>\
                 <hc:next value=\"0\" unit=\"HWPUNIT\"/>\
                 </hh:margin>\
                 <hh:lineSpacing type=\"PERCENT\" value=\"{}\" unit=\"HWPUNIT\"/>\
                 <hh:border borderFillIDRef=\"1\" offsetLeft=\"0\" offsetRight=\"0\" \
                 offsetTop=\"0\" offsetBottom=\"0\" connect=\"0\" ignoreMargin=\"0\"/>\
                 </hh:paraPr>",
                p.id, p.align, p.line_spacing_value
            ));
        }
        s.push_str("</hh:paraProperties>");
        // Minimal styles + tabPrList + numbering placeholders the viewer expects.
        s.push_str(
            "<hh:styles itemCnt=\"1\">\
             <hh:style id=\"0\" type=\"PARA\" name=\"바탕글\" engName=\"Normal\" \
             paraPrIDRef=\"0\" charPrIDRef=\"0\" nextStyleIDRef=\"0\" \
             langID=\"1042\" lockForm=\"0\"/>\
             </hh:styles>",
        );
        s.push_str(
            "<hh:numberings itemCnt=\"0\"/>\
             <hh:bullets itemCnt=\"0\"/>\
             <hh:memoProperties itemCnt=\"0\"/>\
             <hh:trackChanges itemCnt=\"0\"/>\
             <hh:trackChangeAuthors itemCnt=\"0\"/>",
        );
        s.push_str("</hh:refList>");
        s.push_str("<hh:forbiddenWordList/>");
        s.push_str(
            "<hh:compatibleDocument targetProgram=\"HWP2018\">\
             <hh:layoutCompatibility textWrap=\"0\" tableCellApply=\"0\" \
             imageFormat=\"0\" hangulEnglishSpacing=\"0\" blockOverlap=\"0\" \
             legacyLineSpacing=\"0\" legacyAnchorPos=\"0\" \
             legacyPageBreakInTableCell=\"0\" adjustTabStopsAtParagraphEnd=\"0\" \
             treatQuotationAsLatin=\"0\"/>\
             </hh:compatibleDocument>",
        );
        s.push_str(
            "<hh:docOption>\
             <hh:linkinfo path=\"\" pageInherit=\"0\" footnoteInherit=\"0\"/>\
             </hh:docOption>",
        );
        s.push_str("</hh:head>");
        s
    }

    fn section_xml(&self) -> String {
        // Per-line vertical advance in HWPUNIT. Matches our fixture's
        // vertsize=1000 + spacing=600. Real documents compute this from
        // paraPr line spacing; for the validator harness, a fixed advance
        // is enough as long as upstream's FindPageInfo sees paragraphs
        // stacking downward (so no spurious page breaks).
        const LINE_ADVANCE: i64 = 1_600;

        let mut s = String::new();
        s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        s.push_str("<hs:sec ");
        s.push_str(HWPX_NAMESPACES);
        s.push('>');
        let mut cumulative_vert: i64 = 0;
        for (pi, p) in self.paragraphs.iter().enumerate() {
            s.push_str(&format!(
                "<hp:p id=\"{}\" paraPrIDRef=\"{}\" styleIDRef=\"0\" \
                 pageBreak=\"0\" columnBreak=\"0\" merged=\"0\">",
                pi, p.para_pr_id_ref
            ));
            for (ri, r) in p.runs.iter().enumerate() {
                s.push_str(&format!("<hp:run charPrIDRef=\"{}\">", r.char_pr_id_ref));
                // <hp:secPr> belongs in the very first run of the first paragraph.
                if pi == 0 && ri == 0 {
                    s.push_str(SEC_PR);
                }
                s.push_str(&format!("<hp:t>{}</hp:t>", xml_escape(&r.text)));
                s.push_str("</hp:run>");
            }
            // Single lineseg per paragraph, vertpos accumulating across
            // paragraphs so the 2nd+ paragraph isn't interpreted as a new
            // page by the engine's FindPageInfo port.
            s.push_str(&format!(
                "<hp:linesegarray>\
                 <hp:lineseg textpos=\"0\" vertpos=\"{}\" vertsize=\"1000\" \
                 textheight=\"1000\" baseline=\"850\" spacing=\"600\" \
                 horzpos=\"0\" horzsize=\"42520\" flags=\"393216\"/>\
                 </hp:linesegarray>",
                cumulative_vert
            ));
            s.push_str("</hp:p>");
            cumulative_vert += LINE_ADVANCE;
        }
        s.push_str("</hs:sec>");
        s
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const MIMETYPE: &str = "application/hwp+zip";

const HWPX_NAMESPACES: &str = concat!(
    "xmlns:ha=\"http://www.hancom.co.kr/hwpml/2011/app\" ",
    "xmlns:hp=\"http://www.hancom.co.kr/hwpml/2011/paragraph\" ",
    "xmlns:hp10=\"http://www.hancom.co.kr/hwpml/2016/paragraph\" ",
    "xmlns:hs=\"http://www.hancom.co.kr/hwpml/2011/section\" ",
    "xmlns:hc=\"http://www.hancom.co.kr/hwpml/2011/core\" ",
    "xmlns:hh=\"http://www.hancom.co.kr/hwpml/2011/head\" ",
    "xmlns:hhs=\"http://www.hancom.co.kr/hwpml/2011/history\" ",
    "xmlns:hm=\"http://www.hancom.co.kr/hwpml/2011/master-page\" ",
    "xmlns:hpf=\"http://www.hancom.co.kr/schema/2011/hpf\" ",
    "xmlns:dc=\"http://purl.org/dc/elements/1.1/\" ",
    "xmlns:opf=\"http://www.idpf.org/2007/opf/\" ",
    "xmlns:ooxmlchart=\"http://www.hancom.co.kr/hwpml/2016/ooxmlchart\" ",
    "xmlns:hwpunitchar=\"http://www.hancom.co.kr/hwpml/2016/HwpUnitChar\" ",
    "xmlns:epub=\"http://www.idpf.org/2007/ops\" ",
    "xmlns:config=\"urn:oasis:names:tc:opendocument:xmlns:config:1.0\""
);

const FONT_LANGS: &[&str] = &[
    "HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER",
];

const VERSION_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
    "<hv:HCFVersion xmlns:hv=\"http://www.hancom.co.kr/hwpml/2011/version\" ",
    "tagetApplication=\"WORDPROCESSOR\" major=\"5\" minor=\"0\" micro=\"5\" ",
    "buildNumber=\"0\" os=\"1\" xmlVersion=\"1.4\" application=\"polaris-rhwpdvc\" ",
    "appVersion=\"0.1.0\"/>"
);

const SETTINGS_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
    "<ha:HWPApplicationSetting ",
    "xmlns:ha=\"http://www.hancom.co.kr/hwpml/2011/app\" ",
    "xmlns:config=\"urn:oasis:names:tc:opendocument:xmlns:config:1.0\">",
    "<ha:CaretPosition listIDRef=\"0\" paraIDRef=\"0\" pos=\"0\"/>",
    "</ha:HWPApplicationSetting>"
);

const CONTAINER_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
    "<ocf:container xmlns:ocf=\"urn:oasis:names:tc:opendocument:xmlns:container\" ",
    "xmlns:hpf=\"http://www.hancom.co.kr/schema/2011/hpf\">",
    "<ocf:rootfiles>",
    "<ocf:rootfile full-path=\"Contents/content.hpf\" media-type=\"application/hwpml-package+xml\"/>",
    "<ocf:rootfile full-path=\"Preview/PrvText.txt\" media-type=\"text/plain\"/>",
    "</ocf:rootfiles>",
    "</ocf:container>"
);

const MANIFEST_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
    "<odf:manifest xmlns:odf=\"urn:oasis:names:tc:opendocument:xmlns:manifest:1.0\"/>"
);

const CONTENT_HPF: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>",
    "<opf:package ",
    "xmlns:ha=\"http://www.hancom.co.kr/hwpml/2011/app\" ",
    "xmlns:hp=\"http://www.hancom.co.kr/hwpml/2011/paragraph\" ",
    "xmlns:hs=\"http://www.hancom.co.kr/hwpml/2011/section\" ",
    "xmlns:hc=\"http://www.hancom.co.kr/hwpml/2011/core\" ",
    "xmlns:hh=\"http://www.hancom.co.kr/hwpml/2011/head\" ",
    "xmlns:hpf=\"http://www.hancom.co.kr/schema/2011/hpf\" ",
    "xmlns:dc=\"http://purl.org/dc/elements/1.1/\" ",
    "xmlns:opf=\"http://www.idpf.org/2007/opf/\" ",
    "version=\"\" unique-identifier=\"\" id=\"\">",
    "<opf:metadata>",
    "<opf:title/>",
    "<opf:language>ko</opf:language>",
    "<opf:meta name=\"creator\" content=\"text\">polaris-rhwpdvc</opf:meta>",
    "<opf:meta name=\"subject\" content=\"text\"/>",
    "<opf:meta name=\"description\" content=\"text\"/>",
    "<opf:meta name=\"CreatedDate\" content=\"text\">2026-04-19T00:00:00Z</opf:meta>",
    "<opf:meta name=\"ModifiedDate\" content=\"text\">2026-04-19T00:00:00Z</opf:meta>",
    "</opf:metadata>",
    "<opf:manifest>",
    "<opf:item id=\"header\" href=\"Contents/header.xml\" media-type=\"application/xml\"/>",
    "<opf:item id=\"section0\" href=\"Contents/section0.xml\" media-type=\"application/xml\"/>",
    "<opf:item id=\"settings\" href=\"settings.xml\" media-type=\"application/xml\"/>",
    "</opf:manifest>",
    "<opf:spine>",
    "<opf:itemref idref=\"header\"/>",
    "<opf:itemref idref=\"section0\" linear=\"no\"/>",
    "</opf:spine>",
    "</opf:package>"
);

const SEC_PR: &str = concat!(
    "<hp:secPr id=\"\" textDirection=\"HORIZONTAL\" spaceColumns=\"1134\" ",
    "tabStop=\"8000\" tabStopVal=\"4000\" tabStopUnit=\"HWPUNIT\" ",
    "outlineShapeIDRef=\"0\" memoShapeIDRef=\"0\" textVerticalWidthHead=\"0\" ",
    "masterPageCnt=\"0\">",
    "<hp:grid lineGrid=\"0\" charGrid=\"0\" wonggojiFormat=\"0\"/>",
    "<hp:startNum pageStartsOn=\"BOTH\" page=\"0\" pic=\"0\" tbl=\"0\" equation=\"0\"/>",
    "<hp:visibility hideFirstHeader=\"0\" hideFirstFooter=\"0\" ",
    "hideFirstMasterPage=\"0\" border=\"SHOW_ALL\" fill=\"SHOW_ALL\" ",
    "hideFirstPageNum=\"0\" hideFirstEmptyLine=\"0\" showLineNumber=\"0\"/>",
    "<hp:lineNumberShape restartType=\"0\" countBy=\"0\" distance=\"0\" ",
    "startNumber=\"0\"/>",
    "<hp:pagePr landscape=\"WIDELY\" width=\"59528\" height=\"84188\" ",
    "gutterType=\"LEFT_ONLY\">",
    "<hp:margin header=\"4252\" footer=\"4252\" gutter=\"0\" left=\"8504\" ",
    "right=\"8504\" top=\"5668\" bottom=\"4252\"/>",
    "</hp:pagePr>",
    "<hp:footNotePr>",
    "<hp:autoNumFormat type=\"DIGIT\" userChar=\"\" prefixChar=\"\" ",
    "suffixChar=\")\" supscript=\"0\"/>",
    "<hp:noteLine length=\"-1\" type=\"SOLID\" width=\"0.12 mm\" color=\"#000000\"/>",
    "<hp:noteSpacing betweenNotes=\"283\" belowLine=\"567\" aboveLine=\"850\"/>",
    "<hp:numbering type=\"CONTINUOUS\" newNum=\"1\"/>",
    "<hp:placement place=\"EACH_COLUMN\" beneathText=\"0\"/>",
    "</hp:footNotePr>",
    "<hp:endNotePr>",
    "<hp:autoNumFormat type=\"DIGIT\" userChar=\"\" prefixChar=\"\" ",
    "suffixChar=\")\" supscript=\"0\"/>",
    "<hp:noteLine length=\"0\" type=\"SOLID\" width=\"0.12 mm\" color=\"#000000\"/>",
    "<hp:noteSpacing betweenNotes=\"0\" belowLine=\"567\" aboveLine=\"850\"/>",
    "<hp:numbering type=\"CONTINUOUS\" newNum=\"1\"/>",
    "<hp:placement place=\"DOC_END\" beneathText=\"0\"/>",
    "</hp:endNotePr>",
    "</hp:secPr>"
);
