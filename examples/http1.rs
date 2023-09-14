

fn main() {
    let mut req = webparse::Request::new();
    let ret = req.parse(b"GET /index.html HTTP/1.1\r\nHost");
    assert!(ret.err().unwrap().is_partial());

    let buf = b"GET /index.html HTTP/1.1\r\nHost: example.domain\r\n\r\n";
    let ret = req.parse(buf).unwrap();
    
    assert!(ret == buf.len());
    assert!(req.is_complete());
}