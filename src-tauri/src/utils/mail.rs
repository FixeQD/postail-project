use mailparse::parse_header;

pub fn decode_mime_header(header: Option<&[u8]>) -> Option<String> {
    header.map(|bytes| {
        let mut dummy = b"X: ".to_vec();
        dummy.extend_from_slice(bytes);

        match parse_header(&dummy) {
            Ok((parsed, _)) => parsed.get_value(),
            Err(_) => String::from_utf8_lossy(bytes).to_string(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_mime_header() {
        let input = b"=?UTF-8?Q?Alert_bezpiecze=C5=84stwa?=";
        assert_eq!(
            decode_mime_header(Some(input)),
            Some("Alert bezpieczeństwa".to_string())
        );

        // Base64 UTF-8
        let input = b"=?UTF-8?B?QWxlcnQgYmV6cGllY3plxYRzdHdh?=";
        assert_eq!(
            decode_mime_header(Some(input)),
            Some("Alert bezpieczeństwa".to_string())
        );

        // Plain text
        let input = b"Just a normal subject";
        assert_eq!(
            decode_mime_header(Some(input)),
            Some("Just a normal subject".to_string())
        );

        // Multiple parts
        let input = b"=?UTF-8?Q?Part1?= =?UTF-8?Q?Part2?=";
        assert_eq!(decode_mime_header(Some(input)), Some("Part1Part2".to_string()));

        // Mixed plain and encoded
        let input = b"Hello =?UTF-8?Q?World?=";
        assert_eq!(decode_mime_header(Some(input)), Some("Hello World".to_string()));
    }
}
