//! polaris-hwpx: pure-Rust HWPX (OWPML) parser.
//!
//! Byte-input API. No filesystem dependence, so the same code path works on
//! native and WASM targets. Upstream reference: `Source/OWPMLReader.*`.

use std::io::{Cursor, Read};

use thiserror::Error;

mod container;
mod header;
mod section;
mod types;

pub use types::{
    CharPr, FaceName, FontRef, Header, HwpxDocument, LineSeg, ParaPr, Paragraph, Run, Section,
    Shadow, Strikeout, Style, Underline,
};

#[derive(Debug, Error)]
pub enum HwpxError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP container error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("XML parse error: {0}")]
    Xml(String),
    #[error("HWPX structural error: {0}")]
    Structure(&'static str),
}

/// Open an HWPX document from a byte slice.
pub fn open_bytes(input: &[u8]) -> Result<HwpxDocument, HwpxError> {
    let reader = Cursor::new(input);
    let mut zip = zip::ZipArchive::new(reader)?;

    let mimetype = read_entry_as_string(&mut zip, "mimetype").unwrap_or_default();

    let content_hpf = read_entry_as_string(&mut zip, "Contents/content.hpf")
        .ok_or(HwpxError::Structure("missing Contents/content.hpf"))?;
    let manifest = container::parse_content_hpf(&content_hpf)?;

    let header_path = manifest
        .header_path
        .as_deref()
        .unwrap_or("Contents/header.xml");
    let header_xml = read_entry_as_string(&mut zip, header_path)
        .ok_or(HwpxError::Structure("missing header.xml"))?;
    let mut header = header::parse_header(&header_xml)?;
    header.has_macro = manifest.has_macro;

    let mut sections = Vec::with_capacity(manifest.section_paths.len());
    for path in &manifest.section_paths {
        if let Some(xml) = read_entry_as_string(&mut zip, path) {
            sections.push(section::parse_section(&xml)?);
        }
    }

    Ok(HwpxDocument {
        mimetype,
        header,
        sections,
    })
}

fn read_entry_as_string<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> Option<String> {
    let mut entry = zip.by_name(name).ok()?;
    let mut s = String::new();
    entry.read_to_string(&mut s).ok()?;
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_zip_bytes() {
        let err = open_bytes(b"not a zip file").unwrap_err();
        assert!(matches!(err, HwpxError::Zip(_)));
    }

    /// Build a minimal synthetic HWPX in memory and verify end-to-end parsing.
    #[test]
    fn opens_synthetic_hwpx() {
        let bytes = build_synthetic_hwpx();
        let doc = open_bytes(&bytes).expect("parse synthetic HWPX");
        assert_eq!(doc.mimetype, "application/hwp+zip");
        assert_eq!(doc.header.char_shapes.len(), 2);
        assert!(doc.header.char_shapes[0].bold);
        assert_eq!(doc.header.para_shapes.len(), 1);
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].paragraphs.len(), 1);
        let p = &doc.sections[0].paragraphs[0];
        assert_eq!(p.runs.len(), 1);
        assert_eq!(p.runs[0].text, "안녕");
        assert_eq!(p.runs[0].char_pr_id_ref, 1);
    }

    fn build_synthetic_hwpx() -> Vec<u8> {
        use std::io::Write;
        use zip::write::FileOptions;
        use zip::ZipWriter;

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
                br##"<?xml version="1.0" encoding="UTF-8"?>
<hh:head xmlns:hh="h">
  <hh:refList>
    <hh:charProperties itemCnt="2">
      <hh:charPr id="0" height="1000" textColor="#000000">
        <hh:fontRef hangul="0" latin="0"/>
        <hh:bold/>
      </hh:charPr>
      <hh:charPr id="1" height="1200" textColor="#000000">
        <hh:fontRef hangul="0" latin="0"/>
      </hh:charPr>
    </hh:charProperties>
    <hh:paraProperties itemCnt="1">
      <hh:paraPr id="0">
        <hh:align horizontal="JUSTIFY" vertical="BASELINE"/>
        <hh:lineSpacing type="PERCENT" value="160"/>
      </hh:paraPr>
    </hh:paraProperties>
  </hh:refList>
</hh:head>"##,
            )
            .unwrap();

            zip.start_file("Contents/section0.xml", opts).unwrap();
            zip.write_all(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<hs:sec xmlns:hs=\"s\" xmlns:hp=\"p\">\n\
  <hp:p id=\"0\" paraPrIDRef=\"0\" styleIDRef=\"0\">\n\
    <hp:run charPrIDRef=\"1\"><hp:t>안녕</hp:t></hp:run>\n\
  </hp:p>\n\
</hs:sec>"
                    .as_bytes(),
            )
            .unwrap();

            zip.finish().unwrap();
        }
        buf
    }
}
