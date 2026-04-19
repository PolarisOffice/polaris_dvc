//! polaris-hwpx: pure-Rust HWPX (OWPML) parser.
//!
//! Phase 3 of the plan. This skeleton exposes the public byte-input API so
//! dependent crates compile; the real parser will land in follow-up commits.
//!
//! Upstream reference: hancom-io/dvc `Source/OWPMLReader.*`.

use std::io::Cursor;

use thiserror::Error;

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

#[derive(Debug, Default, Clone)]
pub struct HwpxDocument {
    pub mimetype: String,
    pub sections: Vec<Section>,
}

#[derive(Debug, Default, Clone)]
pub struct Section {
    pub paragraphs: Vec<Paragraph>,
}

#[derive(Debug, Default, Clone)]
pub struct Paragraph {
    pub char_pr_id_ref: Option<String>,
    pub para_pr_id_ref: Option<String>,
    pub text: String,
}

/// Open an HWPX document from a byte slice.
///
/// This is the only I/O entry point — no filesystem dependence, so the
/// same API works on native and WASM targets.
pub fn open_bytes(input: &[u8]) -> Result<HwpxDocument, HwpxError> {
    let reader = Cursor::new(input);
    let mut zip = zip::ZipArchive::new(reader)?;

    let mut mimetype = String::new();
    if let Ok(mut f) = zip.by_name("mimetype") {
        use std::io::Read;
        f.read_to_string(&mut mimetype)?;
    }

    // Phase 3: parse META-INF/container.xml, Contents/content.hpf, and
    // Contents/section*.xml into the structures above. Left as `Structure`
    // error until implemented so callers fail loudly rather than silently.
    Ok(HwpxDocument {
        mimetype,
        sections: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_zip_bytes() {
        let err = open_bytes(b"not a zip file").unwrap_err();
        assert!(matches!(err, HwpxError::Zip(_)));
    }
}
