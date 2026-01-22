use encoded_words::decode;

pub fn decode_mime_header(input: Option<&[u8]>) -> Option<String> {
    let bytes = input?;
    let s = std::str::from_utf8(bytes).ok()?;

    match decode(s) {
        Ok(res) => Some(res.decoded),
        Err(_) => Some(s.to_string()),
    }
}
