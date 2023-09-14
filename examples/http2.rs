use webparse::{Helper, http2::{frame::Headers, Decoder, DEFAULT_SETTINGS_HEADER_TABLE_SIZE}, BinaryMut, Method, Scheme};

/// https://httpwg.org/specs/rfc7541.html#huffman.code, C.4
fn parse_header() {
    let mut decoder = Decoder::new();
    // C.4.1
    let buf = Helper::hex_to_vec("8286 8441 8cf1 e3c2 e5f2 3a6b a0ab 90f4 ff");
    let buf_len = buf.len();
    let mut header = Headers::empty();
    let size = header.parse(BinaryMut::from(buf), &mut decoder, DEFAULT_SETTINGS_HEADER_TABLE_SIZE).unwrap();
    assert!(size == buf_len);
    assert!(header.method() == &Some(Method::Get));
    assert!(header.path() == &Some("/".to_string()));
    assert!(header.scheme() == &Some(Scheme::Http));
    assert!(header.authority() == &Some("www.example.com".to_string()));

    // C.4.2
    let buf = Helper::hex_to_vec("8286 84be 5886 a8eb 1064 9cbf");
    let buf_len = buf.len();
    let mut header = Headers::empty();
    let size = header.parse(BinaryMut::from(buf), &mut decoder, DEFAULT_SETTINGS_HEADER_TABLE_SIZE).unwrap();
    assert!(size == buf_len);
    assert!(header.method() == &Some(Method::Get));
    assert!(header.path() == &Some("/".to_string()));
    assert!(header.scheme() == &Some(Scheme::Http));
    assert!(header.authority() == &Some("www.example.com".to_string()));
    assert!(header.fields()["cache-control"] == "no-cache");

    // C.4.3
    let buf = Helper::hex_to_vec("8287 85bf 4088 25a8 49e9 5ba9 7d7f 8925 a849 e95b b8e8 b4bf ");
    let buf_len = buf.len();
    let mut header = Headers::empty();
    let size = header.parse(BinaryMut::from(buf), &mut decoder, DEFAULT_SETTINGS_HEADER_TABLE_SIZE).unwrap();
    assert!(size == buf_len);
    assert!(header.method() == &Some(Method::Get));
    assert!(header.path() == &Some("/index.html".to_string()));
    assert!(header.scheme() == &Some(Scheme::Https));
    assert!(header.authority() == &Some("www.example.com".to_string()));
    assert!(header.fields()["custom-key"] == "custom-value");
}

fn main() {
    parse_header();
}