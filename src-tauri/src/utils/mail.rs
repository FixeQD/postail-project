use mailparse::parse_header;

/// Decodes an RFC2047 MIME header value
/// Handles multi-token encoded-words, folded headers, and mixed ASCII/encoded content.
pub fn decode_mime_header(input: Option<&[u8]>) -> Option<String> {
    let bytes = input?;
    if bytes.is_empty() {
        return None;
    }

    // Wrap raw bytes as a synthetic header so mailparse can fully decode multi-token RFC2047
    let mut synthetic = b"Subject: ".to_vec();
    synthetic.extend_from_slice(bytes);
    synthetic.extend_from_slice(b"\r\n");

    match parse_header(&synthetic) {
        Ok((header, _)) => {
            let value = header.get_value();
            if value.is_empty() {
                // Fallback: best-effort UTF-8 of raw bytes
                Some(String::from_utf8_lossy(bytes).into_owned())
            } else {
                Some(value)
            }
        }
        Err(_) => Some(String::from_utf8_lossy(bytes).into_owned()),
    }
}
