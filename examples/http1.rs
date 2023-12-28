use webparse::Scheme;

fn main() {
    // let mut req = webparse::Request::new();
    // let ret = req.parse(b"GET /index.html HTTP/1.1\r\nHost");
    // assert!(ret.err().unwrap().is_partial());

    // let buf = b"GET /index.html HTTP/1.1\r\nHost: example.domain\r\n\r\n";
    // let ret = req.parse(buf).unwrap();
    
    // assert!(ret == buf.len());
    // assert!(req.is_complete());

    let u = webparse::Url::try_from("https://%4811:!%2011@www.baidu.com:88/path?aaa=222").unwrap();
    println!("url = {:?}", u);
    // assert_eq!(u.domain.unwrap(), "www.baidu.com");

    assert_eq!(u.scheme, Scheme::Https);
    assert_eq!(u.domain.unwrap(), "www.baidu.com");
    assert_eq!(u.username.unwrap(), "H11");
    assert_eq!(u.password.unwrap(), "! 11");
    assert_eq!(u.port.unwrap(), 88);
    assert_eq!(u.path, "/path");
    assert_eq!(u.query.unwrap(), "aaa=222");

}