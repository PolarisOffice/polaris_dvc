//! Format detection and a future-friendly parser dispatch layer.
//!
//! HWPX is implemented in `polaris-rhwpdvc-hwpx`. The HWP 5.0 legacy binary format
//! (OLE2/CFB) is out of the initial scope but reserved behind the `hwp5`
//! feature so dispatch code can be wired in later without API churn.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("HWPX parse error: {0}")]
    Hwpx(#[from] polaris_rhwpdvc_hwpx::HwpxError),
    #[error("HWP 5.0 (OLE2/CFB) parsing is not implemented in this build")]
    Hwp5NotImplemented,
    #[error("unknown document format")]
    UnknownFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Hwpx,
    Hwp5,
    Unknown,
}

/// Identify a document by magic bytes. HWPX is a ZIP with a `mimetype`
/// entry; HWP 5.0 is an OLE2 compound file (`D0 CF 11 E0 A1 B1 1A E1`).
pub fn sniff(bytes: &[u8]) -> DocumentFormat {
    if bytes.len() >= 4 && &bytes[..2] == b"PK" {
        DocumentFormat::Hwpx
    } else if bytes.len() >= 8 && bytes[..8] == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1] {
        DocumentFormat::Hwp5
    } else {
        DocumentFormat::Unknown
    }
}

#[derive(Debug)]
pub enum Document {
    Hwpx(polaris_rhwpdvc_hwpx::HwpxDocument),
}

pub fn parse(bytes: &[u8]) -> Result<Document, ParseError> {
    match sniff(bytes) {
        DocumentFormat::Hwpx => Ok(Document::Hwpx(polaris_rhwpdvc_hwpx::open_bytes(bytes)?)),
        DocumentFormat::Hwp5 => Err(ParseError::Hwp5NotImplemented),
        DocumentFormat::Unknown => Err(ParseError::UnknownFormat),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_zip_as_hwpx() {
        assert_eq!(sniff(b"PK\x03\x04rest"), DocumentFormat::Hwpx);
    }

    #[test]
    fn sniffs_ole2_as_hwp5() {
        let ole2 = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00];
        assert_eq!(sniff(&ole2), DocumentFormat::Hwp5);
    }

    #[test]
    fn sniffs_other_as_unknown() {
        assert_eq!(sniff(b"plain text"), DocumentFormat::Unknown);
    }

    #[test]
    fn hwp5_parsing_returns_not_implemented() {
        let ole2 = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        let err = parse(&ole2).unwrap_err();
        assert!(matches!(err, ParseError::Hwp5NotImplemented));
    }
}
