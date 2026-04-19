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
    pub border_fills: Vec<FixBorderFill>,
    pub paragraphs: Vec<FixParagraph>,
    /// Add a `<opf:item href="Scripts/macros.js" …>` entry so upstream
    /// macro detection (manifest-scan for `.js`) fires on this document.
    pub has_macro: bool,
}

/// Declarative `<hh:borderFill>`. Only the four cardinal sides are
/// configurable; slash / diagonal / backSlash are emitted as NONE.
#[derive(Debug, Clone)]
pub struct FixBorderFill {
    pub id: u32,
    pub left_kind: String,
    pub left_width_mm: f64,
    pub left_color: String,
    pub right_kind: String,
    pub right_width_mm: f64,
    pub right_color: String,
    pub top_kind: String,
    pub top_width_mm: f64,
    pub top_color: String,
    pub bottom_kind: String,
    pub bottom_width_mm: f64,
    pub bottom_color: String,
}

impl FixBorderFill {
    /// Solid 0.12 mm black border on all four sides — matches the
    /// single borderFill the baseline fixture already emits (id 1).
    pub fn solid_default(id: u32) -> Self {
        Self {
            id,
            left_kind: "SOLID".into(),
            left_width_mm: 0.12,
            left_color: "#000000".into(),
            right_kind: "SOLID".into(),
            right_width_mm: 0.12,
            right_color: "#000000".into(),
            top_kind: "SOLID".into(),
            top_width_mm: 0.12,
            top_color: "#000000".into(),
            bottom_kind: "SOLID".into(),
            bottom_width_mm: 0.12,
            bottom_color: "#000000".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FixCharPr {
    pub id: u32,
    pub height: u32, // 1/100 pt (10pt → 1000)
    pub bold: bool,
    pub italic: bool,
    /// `<hh:ratio hangul="…">` (percentage). Default: 100.
    pub ratio: f64,
    /// `<hh:spacing hangul="…">` (HWPUNIT). Default: 0.
    pub spacing: f64,
}

impl Default for FixCharPr {
    fn default() -> Self {
        Self {
            id: 0,
            height: 1000,
            bold: false,
            italic: false,
            ratio: 100.0,
            spacing: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FixParaPr {
    pub id: u32,
    pub align: String, // e.g., "JUSTIFY"
    pub line_spacing_value: f64,
    pub margin_prev: f64,
    pub margin_next: f64,
    pub margin_intent: f64,
    pub margin_left: f64,
}

impl Default for FixParaPr {
    fn default() -> Self {
        Self {
            id: 0,
            align: "JUSTIFY".into(),
            line_spacing_value: 160.0,
            margin_prev: 0.0,
            margin_next: 0.0,
            margin_intent: 0.0,
            margin_left: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FixParagraph {
    pub para_pr_id_ref: u32,
    /// Non-zero value sets `styleIDRef` on the `<hp:p>` — drives the
    /// `style.permission` rule.
    pub style_id_ref: u32,
    pub runs: Vec<FixRun>,
    /// Optional table rendered after the final run of this paragraph.
    /// The fixture wraps it in an extra `<hp:run>` so real HWPX parsers
    /// accept it.
    pub table: Option<FixTable>,
}

#[derive(Debug, Clone)]
pub struct FixTable {
    pub id: u32,
    pub border_fill_id_ref: u32,
    pub row_cnt: u32,
    pub col_cnt: u32,
}

#[derive(Debug, Clone)]
pub struct FixRun {
    pub char_pr_id_ref: u32,
    pub text: String,
    /// When true, the emitted run is wrapped in
    /// `<hp:fieldBegin type="HYPERLINK"> … <hp:fieldEnd/>` so the parser
    /// surfaces it as a hyperlinked run for the permission rule.
    pub hyperlink: bool,
}

impl Fixture {
    /// Baseline: one paragraph, 바탕 Hangul face at 10pt, non-bold, "안녕".
    pub fn baseline() -> Self {
        Self {
            hangul_face: "바탕".into(),
            char_prs: vec![FixCharPr::default()],
            para_prs: vec![FixParaPr::default()],
            border_fills: vec![FixBorderFill::solid_default(1)],
            paragraphs: vec![FixParagraph {
                para_pr_id_ref: 0,
                style_id_ref: 0,
                runs: vec![FixRun {
                    char_pr_id_ref: 0,
                    text: "안녕".into(),
                    hyperlink: false,
                }],
                table: None,
            }],
            has_macro: false,
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
            zip.write_all(self.content_hpf().as_bytes()).unwrap();

            if self.has_macro {
                // Minimal JS asset — presence in the manifest is what
                // upstream macro detection checks for, content doesn't
                // need to be valid JavaScript.
                zip.start_file("Scripts/macros.js", stored).unwrap();
                zip.write_all(b"// polaris fixture macro stub\n").unwrap();
            }

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

    fn content_hpf(&self) -> String {
        let mut s = String::with_capacity(1024);
        s.push_str(CONTENT_HPF_PREFIX);
        if self.has_macro {
            s.push_str(
                "<opf:item id=\"macros\" href=\"Scripts/macros.js\" \
                 media-type=\"application/javascript\"/>",
            );
        }
        s.push_str(CONTENT_HPF_SUFFIX);
        s
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
        // borderFills: declared up front so tables and charPr can reference
        // them by id. The baseline fixture defines one (id=1, solid black);
        // a golden case can swap/extend this to drive the table-border rule.
        s.push_str(&format!(
            "<hh:borderFills itemCnt=\"{}\">",
            self.border_fills.len()
        ));
        for bf in &self.border_fills {
            s.push_str(&format!(
                "<hh:borderFill id=\"{}\" threeD=\"0\" shadow=\"0\" \
                 slash=\"NONE\" backSlash=\"NONE\" crookedSlash=\"0\" \
                 isCounterSlash=\"0\" isCounterBackSlash=\"0\">\
                 <hh:slash type=\"NONE\" Crooked=\"0\" isCounter=\"0\"/>\
                 <hh:backSlash type=\"NONE\" Crooked=\"0\" isCounter=\"0\"/>\
                 <hh:leftBorder type=\"{}\" width=\"{:.2} mm\" color=\"{}\"/>\
                 <hh:rightBorder type=\"{}\" width=\"{:.2} mm\" color=\"{}\"/>\
                 <hh:topBorder type=\"{}\" width=\"{:.2} mm\" color=\"{}\"/>\
                 <hh:bottomBorder type=\"{}\" width=\"{:.2} mm\" color=\"{}\"/>\
                 <hh:diagonal type=\"SOLID\" width=\"0.12 mm\" color=\"#000000\"/>\
                 </hh:borderFill>",
                bf.id,
                bf.left_kind,
                bf.left_width_mm,
                bf.left_color,
                bf.right_kind,
                bf.right_width_mm,
                bf.right_color,
                bf.top_kind,
                bf.top_width_mm,
                bf.top_color,
                bf.bottom_kind,
                bf.bottom_width_mm,
                bf.bottom_color,
            ));
        }
        s.push_str("</hh:borderFills>");
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
                 <hh:ratio hangul=\"{r}\" latin=\"{r}\" hanja=\"{r}\" \
                 japanese=\"{r}\" other=\"{r}\" symbol=\"{r}\" user=\"{r}\"/>\
                 <hh:spacing hangul=\"{sp}\" latin=\"{sp}\" hanja=\"{sp}\" \
                 japanese=\"{sp}\" other=\"{sp}\" symbol=\"{sp}\" user=\"{sp}\"/>\
                 <hh:relSz hangul=\"100\" latin=\"100\" hanja=\"100\" \
                 japanese=\"100\" other=\"100\" symbol=\"100\" user=\"100\"/>\
                 <hh:offset hangul=\"0\" latin=\"0\" hanja=\"0\" japanese=\"0\" \
                 other=\"0\" symbol=\"0\" user=\"0\"/>",
                c.id,
                c.height,
                r = c.ratio,
                sp = c.spacing,
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
                 <hc:intent value=\"{intent}\" unit=\"HWPUNIT\"/>\
                 <hc:left value=\"{left}\" unit=\"HWPUNIT\"/>\
                 <hc:right value=\"0\" unit=\"HWPUNIT\"/>\
                 <hc:prev value=\"{prev}\" unit=\"HWPUNIT\"/>\
                 <hc:next value=\"{next}\" unit=\"HWPUNIT\"/>\
                 </hh:margin>\
                 <hh:lineSpacing type=\"PERCENT\" value=\"{ls}\" unit=\"HWPUNIT\"/>\
                 <hh:border borderFillIDRef=\"1\" offsetLeft=\"0\" offsetRight=\"0\" \
                 offsetTop=\"0\" offsetBottom=\"0\" connect=\"0\" ignoreMargin=\"0\"/>\
                 </hh:paraPr>",
                p.id,
                p.align,
                ls = p.line_spacing_value,
                intent = p.margin_intent,
                left = p.margin_left,
                prev = p.margin_prev,
                next = p.margin_next,
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
                "<hp:p id=\"{}\" paraPrIDRef=\"{}\" styleIDRef=\"{}\" \
                 pageBreak=\"0\" columnBreak=\"0\" merged=\"0\">",
                pi, p.para_pr_id_ref, p.style_id_ref
            ));
            for (ri, r) in p.runs.iter().enumerate() {
                // <hp:fieldBegin> opens a hyperlink scope around the run
                // when requested. The matching fieldEnd closes it right
                // after the text so the scope covers exactly this run.
                if r.hyperlink {
                    s.push_str(
                        "<hp:fieldBegin type=\"HYPERLINK\" name=\"hyp\" \
                         editable=\"1\" dirty=\"0\"><hp:parameters count=\"0\"/>\
                         </hp:fieldBegin>",
                    );
                }
                s.push_str(&format!("<hp:run charPrIDRef=\"{}\">", r.char_pr_id_ref));
                // <hp:secPr> belongs in the very first run of the first paragraph.
                if pi == 0 && ri == 0 {
                    s.push_str(SEC_PR);
                }
                s.push_str(&format!("<hp:t>{}</hp:t>", xml_escape(&r.text)));
                s.push_str("</hp:run>");
                if r.hyperlink {
                    s.push_str("<hp:fieldEnd fieldType=\"HYPERLINK\" fieldName=\"hyp\"/>");
                }
            }
            // Optional table — emitted after the last regular run so the
            // <hp:tbl> still sits inside the paragraph, matching how real
            // HWPX stores inline tables.
            if let Some(t) = p.table.as_ref() {
                s.push_str(&format!(
                    "<hp:run charPrIDRef=\"{}\">",
                    p.runs.first().map(|r| r.char_pr_id_ref).unwrap_or(0)
                ));
                s.push_str(&format!(
                    "<hp:tbl id=\"{}\" zOrder=\"0\" numberingType=\"TABLE\" \
                     textWrap=\"TOP_AND_BOTTOM\" textFlow=\"BOTH_SIDES\" \
                     borderFillIDRef=\"{}\" noAdjust=\"0\" rowCnt=\"{}\" \
                     colCnt=\"{}\" cellSpacing=\"0\">\
                     <hp:sz width=\"42520\" widthRelTo=\"ABSOLUTE\" \
                     height=\"2000\" heightRelTo=\"ABSOLUTE\" protect=\"0\"/>\
                     <hp:pos treatAsChar=\"0\" affectLSpacing=\"0\" \
                     flowWithText=\"0\" allowOverlap=\"0\" holdAnchorAndSO=\"0\" \
                     vertRelTo=\"PARA\" horzRelTo=\"COLUMN\" vertAlign=\"TOP\" \
                     horzAlign=\"LEFT\" vertOffset=\"0\" horzOffset=\"0\"/>\
                     <hp:outside left=\"0\" right=\"0\" top=\"0\" bottom=\"0\"/>\
                     <hp:inMargin left=\"141\" right=\"141\" top=\"141\" bottom=\"141\"/>\
                     <hp:tr><hp:tc name=\"\" header=\"0\" hasMargin=\"0\" \
                     protect=\"0\" editable=\"1\" dirty=\"0\" borderFillIDRef=\"{}\">\
                     <hp:subList id=\"\" textDirection=\"HORIZONTAL\" \
                     lineWrap=\"BREAK\" vertAlign=\"TOP\" linkListIDRef=\"0\" \
                     linkListNextIDRef=\"0\" textWidth=\"0\" padding=\"0\" \
                     lang=\"KOREAN\"/>\
                     <hp:cellAddr colAddr=\"0\" rowAddr=\"0\"/>\
                     <hp:cellSpan colSpan=\"1\" rowSpan=\"1\"/>\
                     <hp:cellSz width=\"42520\" height=\"2000\"/>\
                     <hp:cellMargin left=\"141\" right=\"141\" top=\"141\" bottom=\"141\"/>\
                     </hp:tc></hp:tr>\
                     </hp:tbl>",
                    t.id, t.border_fill_id_ref, t.row_cnt, t.col_cnt, t.border_fill_id_ref
                ));
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

const CONTENT_HPF_PREFIX: &str = concat!(
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
);

const CONTENT_HPF_SUFFIX: &str = concat!(
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
