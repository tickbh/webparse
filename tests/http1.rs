use webparse::{Request, Version, Method};

macro_rules! req {

    ($name:ident, $buf:expr, |$arg:ident| $body:expr) => (
    #[test]
    fn $name() {

        let mut req = Request::new();
        let size = req.parse($buf.as_ref()).unwrap();
        assert_eq!(size, $buf.len());
        closure(req);

        fn closure($arg: Request<()>) {
            $body
        }
    }
    )
}


req! {
    urltest_001,
    b"GET /bar;par?b HTTP/1.1\r\nHost: foo\r\n\r\n",
    |req| {
        assert_eq!(req.method(), &Method::Get);
        assert_eq!(req.path(), "/bar;par");
        assert_eq!(req.version(), &Version::Http11);
        assert_eq!(req.headers().len(), 1);
        // assert_eq!(req.headers()["Host"], b"foo");
    }
}
    // let _result = request.parse(b"GET http://www.baidu.com/ HTTP/1.1\r\nHost: www.baidu.com\r\nUser-Agent: curl/7.74.0\r\nAccept: */*\r\nProxy-Connection: Keep-Alive\r\n\r\n");
    // println!("result = {:?}", request);
    // println!("is_partial = {}", request.is_partial());
    // println!("body len = {}", request.get_body_len());
    // println!("host len = {:?}", request.get_host());
    // println!("host len = {:?}", request.get_connect_url());
