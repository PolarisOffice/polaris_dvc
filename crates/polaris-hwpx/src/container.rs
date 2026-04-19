//! HWPX container handling: mimetype + OPF manifest (`content.hpf`).

use crate::HwpxError;
use quick_xml::events::Event;
use quick_xml::Reader;

/// List of section XML paths extracted from `Contents/content.hpf`.
///
/// HWPX manifests sections as `<opf:item media-type="application/xml" …>`
/// entries whose `href` starts with `Contents/section`. We preserve the
/// spine order (by idref) when possible, falling back to manifest order.
#[derive(Debug, Default, Clone)]
pub struct Manifest {
    pub section_paths: Vec<String>,
    pub header_path: Option<String>,
}

pub fn parse_content_hpf(xml: &str) -> Result<Manifest, HwpxError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut manifest = Manifest::default();
    // id -> href
    let mut items: Vec<(String, String, String)> = Vec::new();
    let mut spine: Vec<String> = Vec::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = local_name(e.name().as_ref());
                if name == "item" {
                    let mut id = String::new();
                    let mut href = String::new();
                    let mut media = String::new();
                    for attr in e.attributes().flatten() {
                        let key = local_name(attr.key.as_ref());
                        let value = attr
                            .decode_and_unescape_value(&reader)
                            .map_err(|err| HwpxError::Xml(err.to_string()))?
                            .into_owned();
                        match key.as_str() {
                            "id" => id = value,
                            "href" => href = value,
                            "media-type" => media = value,
                            _ => {}
                        }
                    }
                    if !id.is_empty() && !href.is_empty() {
                        items.push((id, href, media));
                    }
                }
                if name == "itemref" {
                    for attr in e.attributes().flatten() {
                        if local_name(attr.key.as_ref()) == "idref" {
                            let value = attr
                                .decode_and_unescape_value(&reader)
                                .map_err(|err| HwpxError::Xml(err.to_string()))?
                                .into_owned();
                            spine.push(value);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    // Resolve header
    manifest.header_path = items
        .iter()
        .find(|(_, href, _)| href.ends_with("header.xml"))
        .map(|(_, href, _)| href.clone());

    // Resolve sections, preferring spine order.
    let mut ordered: Vec<String> = Vec::new();
    for id in &spine {
        if let Some((_, href, _)) = items.iter().find(|(i, _, _)| i == id) {
            if href.contains("/section") {
                ordered.push(href.clone());
            }
        }
    }
    if ordered.is_empty() {
        for (_, href, _) in &items {
            if href.contains("/section") && href.ends_with(".xml") {
                ordered.push(href.clone());
            }
        }
    }
    manifest.section_paths = ordered;
    Ok(manifest)
}

/// Returns the local name (stripped of any `ns:` prefix) as a `String`.
pub fn local_name(full: &[u8]) -> String {
    let s = std::str::from_utf8(full).unwrap_or("");
    match s.rsplit_once(':') {
        Some((_, local)) => local.to_string(),
        None => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_HPF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf/" version="1.0">
  <opf:manifest>
    <opf:item id="header" href="Contents/header.xml" media-type="application/xml"/>
    <opf:item id="sec0" href="Contents/section0.xml" media-type="application/xml"/>
    <opf:item id="sec1" href="Contents/section1.xml" media-type="application/xml"/>
    <opf:item id="settings" href="Contents/settings.xml" media-type="application/xml"/>
  </opf:manifest>
  <opf:spine>
    <opf:itemref idref="sec0"/>
    <opf:itemref idref="sec1"/>
  </opf:spine>
</opf:package>"#;

    #[test]
    fn parses_manifest_and_spine_order() {
        let m = parse_content_hpf(SAMPLE_HPF).unwrap();
        assert_eq!(m.header_path.as_deref(), Some("Contents/header.xml"));
        assert_eq!(
            m.section_paths,
            vec![
                "Contents/section0.xml".to_string(),
                "Contents/section1.xml".to_string()
            ]
        );
    }

    #[test]
    fn falls_back_to_manifest_order_without_spine() {
        let xml = r#"<?xml version="1.0"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf/">
  <opf:manifest>
    <opf:item id="b" href="Contents/section1.xml" media-type="application/xml"/>
    <opf:item id="a" href="Contents/section0.xml" media-type="application/xml"/>
  </opf:manifest>
</opf:package>"#;
        let m = parse_content_hpf(xml).unwrap();
        // Manifest order is preserved when spine is absent.
        assert_eq!(
            m.section_paths,
            vec![
                "Contents/section1.xml".to_string(),
                "Contents/section0.xml".to_string()
            ]
        );
    }
}
