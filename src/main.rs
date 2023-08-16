use webparse::Url;

extern crate webparse;


fn main() {
    let mut request = webparse::Request::new();
    // let _result = request.parse(b"GET /index.html HTTP/1.1\r\nHost: example.domain\r\n\r\n").unwrap();
    // println!("result = {:?}", request);
    // println!("is_partial = {}", request.is_partial());

    let _result = request.parse(b"GET /index.html HTTP/1.1\r\nHost: example.domain1\r\n");
    println!("result = {:?}", request);
    println!("is_partial = {}", request.is_partial());

    let url = Url::parse("https://%4811:!%2011@www.baidu.com:88/path?aaa=222");
    println!("value = {:?}", url);
    println!("value = {}", url.ok().unwrap());
    
    let url = Url::parse("/path?qqq=222");
    println!("value = {:?}", url);
    println!("value = {}", url.ok().unwrap());

    println!("decode = {:?}", Url::url_decode("%48%211111"));
    println!("decode = {:?}", Url::url_decode("%48%21111%1"));
    println!("decode = {:?}", Url::url_decode("%48%21111%4j"));
    // // let value = url::Url::parse("https://11:11@www.baidu.com/path");
    // let value = url::Url::parse("/path");
}
