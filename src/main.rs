use webparse::{url, Url, Request, HeaderName, HeaderValue, Serialize, Buffer, Response};

extern crate webparse;


fn main() {
    let mut request = webparse::Request::builder().body("What is this".to_string()).unwrap();
    // // let _result = request.parse(b"GET /index.html HTTP/1.1\r\nHost: example.domain\r\n\r\n").unwrap();
    // // println!("result = {:?}", request);
    // // println!("is_partial = {}", request.is_partial());

    // let _result = request.parse(b"GET /index.html HTTP/1.1\r\nHost: example.domain1\r\ncontent-length: 1111\r\n");
    // println!("result = {:?}", request);
    // println!("is_partial = {}", request.is_partial());
    // println!("body len = {}", request.get_body_len());

    // let url = Url::parse("https://%4811:!%2011@www.baidu.com:88/path?aaa=222");
    // println!("value = {:?}", url);
    // println!("value = {}", url.ok().unwrap());
    
    // let url = Url::parse("/path?qqq=222");
    // println!("value = {:?}", url);
    // println!("value = {}", url.ok().unwrap());

    // println!("decode = {:?}", Url::url_decode("%48%211111"));
    // println!("decode = {:?}", Url::url_decode("%48%21111%1"));
    // println!("decode = {:?}", Url::url_decode("%48%21111%4j"));
    let value = Url::parse("https://11:11@www.baidu.com/path").unwrap();
    println!("value = {}", value);

    // let value = url::Url::parse("/path");
    let _result = request.parse(b"GET http://www.baidu.com/ HTTP/1.1\r\nHost: www.baidu.com\r\nUser-Agent: curl/7.74.0\r\nAccept: */*\r\nProxy-Connection: Keep-Alive\r\n\r\n");
    println!("result = {:?}", request);
    println!("is_partial = {}", request.is_partial());
    println!("body len = {}", request.get_body_len());
    println!("host len = {:?}", request.get_host());
    println!("host len = {:?}", request.get_connect_url());

    let mut buffer = Buffer::new();
    request.serialize(&mut buffer).expect("ok");
    println!("aaaaaaaaaaaaaaa {}", String::from_utf8_lossy(buffer.get_write_data()));

    println!("aaaaaaaaaaaaaaa11111 {}", String::from_utf8_lossy(&request.httpdata().unwrap()));

    let mut req = Request::builder();
    assert_eq!(req.url_ref().unwrap(), "/" );
    req = req.url("https://www.rust-lang.org/");
    assert_eq!(req.url_ref().unwrap(), "https://www.rust-lang.org/" );


    let mut req = Request::builder()
        .header("Accept", "text/html")
        .header("X-Custom-Foo", "bar");
    let headers = req.headers_mut().unwrap();
    headers["AAAA"] = HeaderValue::Stand("ok");
    let xx = &headers["Accept"];
    let aaa = &headers["AAAA"];
    println!("xxx = {:?}", xx);
    println!("aaa = {:?}", aaa);


    for value in headers.iter() {
        println!("____={:?}", value.0);
        println!("____={:?}", value.1);
    }
    assert_eq!( &headers["Accept"], "text/html" );
    assert_eq!( &headers["X-Custom-Foo"], "bar" );

    let u = url::Builder::new().scheme("https").domain("www.baidu.com").build().unwrap();
    println!("u = {}", u);

    let response = Response::builder()
    .header("Accept", "text/html")
    .header("X-Custom-Foo", "bar").body("my is web").unwrap();
    
    println!("ssssssssssss {}", String::from_utf8_lossy(&response.httpdata().unwrap()));
    // let response = url::Builder::
    
}
