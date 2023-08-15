extern crate webparse;

use url;

fn main() {
    let mut request = webparse::Request::new();
    // let _result = request.parse(b"GET /index.html HTTP/1.1\r\nHost: example.domain\r\n\r\n").unwrap();
    // println!("result = {:?}", request);
    // println!("is_partial = {}", request.is_partial());

    let _result = request.parse(b"GET /index.html HTTP/1.1\r\nHost: example.domai");
    println!("result = {:?}", request);
    println!("is_partial = {}", request.is_partial());

    // // let value = url::Url::parse("https://11:11@www.baidu.com/path");
    // let value = url::Url::parse("/path");
    // println!("value = {:?}", value);
}
