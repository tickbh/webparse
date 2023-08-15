extern crate webparse;

fn main() {
    let mut request = webparse::Request::new();
    let _result = request.parse(b"GET /index.html HTTP/1.1\r\nHost: example.domain\r\n\r\n").unwrap();
    println!("result = {:?}", request);
}
