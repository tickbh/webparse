use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Range, RangeBounds};
use std::sync::{Arc, Mutex};

use webparse::http::http2::{Decoder, HeaderIndex};
use webparse::http::request;
use webparse::{
    url, Binary, BinaryMut, Buf, HeaderName, HeaderValue, Helper, Request, Response, Serialize,
    Url, Version, BufMut,
};

#[derive(Debug)]
pub enum Pay<T>
where T:Buf {
    Data(T)
}


pub trait Test {
    fn serialize1<B: Buf + BufMut>(&self, buf: &mut B);
}

impl Test for &'static str {
    fn serialize1<B: Buf + BufMut>(&self, buf: &mut B) {
        buf.put_slice(self.as_bytes());
    }
}

extern crate webparse;

fn hexstr_to_vec(s: &str) -> Vec<u8> {
    let mut result = vec![];
    let bytes = s.as_bytes();
    let mut val = 0;
    let mut is_first = true;
    for b in bytes {
        if b != &b' ' {
            if is_first {
                val = u8::from_str_radix(std::str::from_utf8(&[*b]).unwrap(), 16).unwrap();
                is_first = false
            } else {
                val =
                    val * 16 + u8::from_str_radix(std::str::from_utf8(&[*b]).unwrap(), 16).unwrap();
                result.push(val);
                val = 0;
                is_first = true;
            }
        }
    }
    result
}

// fn to_hex(v: u8) -> String {
//     if v < 10 {
//         return format!("{}", v);
//     } else {
//         return
//     }
// }

fn hex_debug_print(val: &[u8]) {
    for v in val {
        print!(
            "{}{}  ",
            String::from_utf8_lossy(&vec![Helper::to_hex(v / 16)]),
            String::from_utf8_lossy(&vec![Helper::to_hex(v % 16)])
        );
    }
    println!();
}


fn debug_request_parse_full_http2() {
    let http2: Vec<u8> = vec! [80, 82, 73, 32, 42, 32, 72, 84, 84, 80, 47, 50, 46, 48, 13, 10, 13, 10, 83, 77, 13, 10, 13, 10, 0, 0, 18, 4, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 100, 0, 4, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 4, 8, 0, 0, 0, 0, 0, 1, 255, 0, 1, 0, 0, 55, 1, 5, 0, 0, 0, 1, 131, 132, 134, 65, 143, 11, 226, 92, 46, 60, 184, 93, 125, 112, 178, 205, 199, 128, 240, 63, 122, 136, 37, 182, 80, 195, 171, 186, 210, 224, 83, 3, 42, 47, 42, 64, 136, 37, 168, 73, 233, 91, 169, 125, 127, 137, 37, 168, 73, 233, 91, 184, 232, 180, 191, 0, 0, 25, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 83, 69, 84, 84, 73, 78, 71, 83, 32, 101, 120, 112, 101, 99, 116, 101, 100];
    let mut req = Request::new();
    let size = req.parse(&http2).unwrap();
    println!("req = {:?}", req);
}

fn debug_request_parse_http2() {
    let mut decode = Decoder::new();
    let http2 = vec![
        0x82, 0x86, 0x84, 0x41, 0x8c, 0xf1, 0xe3, 0xc2, 0xe5, 0xf2, 0x3a, 0x6b, 0xa0, 0xab, 0x90,
        0xf4, 0xff,
    ];
    let http2 = hexstr_to_vec("8286 8441 0f77 7777 2e65 7861 6d70 6c65 2e63 6f6d ");

    let mut buf = BinaryMut::from(http2);

    let result = decode.decode_with_cb(&mut buf, |n, v| {
        println!("n = {:?}, v = {:?}", n, v);
    });
    // println!("result = {:?}", result);
    // let http2 = vec![
    //     0x82, 0x86, 0x84, 0xbe, 0x58, 0x08, 0x6e, 0x6f, 0x2d, 0x63, 0x61, 0x63, 0x68, 0x65,
    // ];
    // let http2 = hexstr_to_vec("8286 84be 5808 6e6f 2d63 6163 6865");

    // let mut buf = BinaryMut::from(http2);
    // let result = decode.decode_with_cb(&mut buf, |n, v| {
    //     println!("n = {:?}, v = {:?}", n, v);
    // });
    // println!("result = {:?}", result);

    // {
    //     //         8287 85bf 4088 25a8 49e9 5ba9 7d7f 8925 | ....@.%.I.[.}..%
    //     //          a849 e95b b8e8 b4bf
    //     let http2 = vec![
    //         0x82, 0x87, 0x85, 0xbf, 0x40, 0x88, 0x25, 0xa8, 0x49, 0xe9, 0x5b, 0xa9, 0x7d, 0x7f,
    //         0x89, 0x25, 0xa8, 0x49, 0xe9, 0x5b, 0xb8, 0xe8, 0xb4, 0xbf,
    //     ];
    //     let http2 = hexstr_to_vec(
    //         "8287 85bf 400a 6375 7374 6f6d 2d6b 6579 0c63 7573 746f 6d2d 7661 6c75 65",
    //     );

    //     let mut buf = BinaryMut::from(http2);

    //     let result = decode.decode_with_cb(&mut buf, |n, v| {
    //         println!("n = {:?}, v = {:?}", n, v);
    //     });
    //     println!("result = {:?}", result);
    // }
}

fn debug_request_parse() {
    let mut request = Request::new();
    request.headers_mut().insert("Connection", "ok");
    let xx = request.headers().is_keep_alive();
    return;
    // // let value = url::Url::parse("/path");
    let bytes = [80, 79, 83, 84, 32, 47, 112, 111, 115, 116, 32, 72, 84, 84, 80, 47, 49, 46, 49, 13, 10, 72, 111, 115, 116, 58, 32, 49, 57, 50, 46, 49, 54, 56, 46, 49, 55, 57, 46, 49, 51, 51, 58, 56, 48, 56, 48, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110, 116, 58, 32, 99, 117, 114, 108, 47, 55, 46, 55, 52, 46, 48, 13, 10, 65, 99, 99, 101, 112, 116, 58, 32, 42, 47, 42, 13, 10, 67, 111, 110, 110, 101, 99, 116, 105, 111, 110, 58, 32, 85, 112, 103, 114, 97, 100, 101, 44, 32, 72, 84, 84, 80, 50, 45, 83, 101, 116, 116, 105, 110, 103, 115, 13, 10, 85, 112, 103, 114, 97, 100, 101, 58, 32, 104, 50, 99, 13, 10, 72, 84, 84, 80, 50, 45, 83, 101, 116, 116, 105, 110, 103, 115, 58, 32, 65, 65, 77, 65, 65, 65, 66, 107, 65, 65, 81, 67, 65, 65, 65, 65, 65, 65, 73, 65, 65, 65, 65, 65, 13, 10, 99, 117, 115, 116, 111, 109, 45, 107, 101, 121, 58, 99, 117, 115, 116, 111, 109, 45, 118, 97, 108, 117, 101, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32, 50, 49, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 120, 45, 119, 119, 119, 45, 102, 111, 114, 109, 45, 117, 114, 108, 101, 110, 99, 111, 100, 101, 100, 13, 10, 13, 10, 97, 97, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48];
    let _result = request.parse(b"GET //:///// HTTP/1.1\r\nHost: Upgrade, HTTP2-Settings \r\n\r\naaa");
    // let _result = request.parse(&bytes);

    println!("result = {:?}", request);
    println!("is_partial = {}", request.is_partial());
    println!("body len = {}", request.get_body_len());
    println!("host len = {:?}", request.get_host());
    println!("host len = {:?}", request.get_connect_url());
    println!(
        "http data = {}",
        String::from_utf8_lossy(&request.http1_data().unwrap())
    );
    assert_eq!(
        String::from_utf8_lossy(&request.http1_data().unwrap()).as_bytes(),
        b"GET //:///// HTTP/1.1\r\nHost: \r\n\r\n"
    );
    let x = &request.headers()["Host"];
    if x == &"foo" {
        println!("111");
    }
    if &"foo" == x {
        println!("111");
    }
}

fn main() {

    let mut req = crate::Request::new();
    let ret = req.parse(b"GET /index.html HTTP/1.1\r\nHost");
    assert!(ret.err().unwrap().is_partial());

    let buf = b"GET /index.html HTTP/1.1\r\nHost: example.domain\r\n\r\n";
    let ret = req.parse(buf).unwrap();
    
    assert!(ret == buf.len());
    assert!(req.is_complete());

    debug_request_parse();
    // let mut binmut = BinaryMut::new();
    // "aaa".serialize1(&mut binmut);

    // let bin = Binary::new();
    // let p = Pay::Data(bin);
    // println!("bbb = {:?}", p);
    // debug_request_parse_full_http2();
    // debug_request_parse_http2();

    //  let req = request::Builder::new()
    //      .method("POST")
    //      .body(())
    //      .unwrap();

    // debug_request_parse();
    // let v = vec![1u8, 2, 3, 5, 7, 9, 10].into_boxed_slice();
    // let mut b = Binary::from(v);
    // {
    //     b.get_next();
    //     let c = b.clone_slice();
    //     drop(c);
    //     b.get_next();
    //     let d = b.clone_slice();
    //     drop(d);
    // }
    // drop(b);
    println!("finish");
    // let len = v.len();
    // let raw = Box::into_raw(v) as *mut u8;

    // // Layout::from_size_align(cap, 1);

    // let value = unsafe { Vec::from_raw_parts(raw, len, len) };
    // println!("value = {:?}", value);

    if true {
        return;
    }

    // let mut request = webparse::Request::builder().body("What is this".to_string()).unwrap();
    // // let _result = request.parse(b"GET /index.html HTTP/1.1\r\nHost: example.domain\r\n\r\n").unwrap();
    // // println!("result = {:?}", request);
    // // println!("is_partial = {}", request.is_partial());

    // let _result = request.parse(b"GET /index.html HTTP/1.1\r\nHost: example.domain1\r\ncontent-length: 1111\r\n");
    // println!("result = {:?}", request);
    // println!("is_partial = {}", request.is_partial());
    // println!("body len = {}", request.get_body_len());

    let url: Result<Url, webparse::WebError> =
        Url::try_from("https://%4811:!%2011@www.baidu.com:88/path?aaa=222");
    println!("value = {:?}", url);
    println!("value = {}", url.ok().unwrap());

    let url = Url::try_from("/path?qqq=222");
    println!("value = {:?}", url);
    println!("value = {}", url.ok().unwrap());

    println!("decode = {:?}", Url::url_decode("%48%211111"));
    println!("decode = {:?}", Url::url_decode("%48%21111%1"));
    println!("decode = {:?}", Url::url_decode("%48%21111%4j"));
    let value = Url::try_from("https://11:11@www.baidu.com/path").unwrap();
    println!("value = {}", value);

    // // let value = url::Url::parse("/path");
    // let _result = request.parse(b"GET http://www.baidu.com/ HTTP/1.1\r\nHost: www.baidu.com\r\nUser-Agent: curl/7.74.0\r\nAccept: */*\r\nProxy-Connection: Keep-Alive\r\n\r\n");
    // println!("result = {:?}", request);
    // println!("is_partial = {}", request.is_partial());
    // println!("body len = {}", request.get_body_len());
    // println!("host len = {:?}", request.get_host());
    // println!("host len = {:?}", request.get_connect_url());

    // let mut buffer = Buffer::new();
    // request.serialize(&mut buffer).expect("ok");
    // println!("aaaaaaaaaaaaaaa {}", String::from_utf8_lossy(buffer.get_write_data()));

    // println!("aaaaaaaaaaaaaaa11111 {}", String::from_utf8_lossy(&request.httpdata().unwrap()));

    // let mut req = Request::builder();
    // assert_eq!(req.url_ref().unwrap(), "/" );
    // req = req.url("https://www.rust-lang.org/");
    // assert_eq!(req.url_ref().unwrap(), "https://www.rust-lang.org/" );

    // let mut req = Request::builder().version(Version::Http2).method("GET")
    //     .header("Accept", "text/html")
    //     .header("X-Custom-Foo", "bar");
    // {
    //     let headers = req.headers_mut().unwrap();
    //     headers["AAAA"] = HeaderValue::Stand("ok");
    //     let xx = &headers["Accept"];
    //     let aaa = &headers["AAAA"];
    //     println!("xxx = {:?}", xx);
    //     println!("aaa = {:?}", aaa);

    //     for value in headers.iter() {
    //         println!("____={:?}", value.0);
    //         println!("____={:?}", value.1);
    //     }
    //     assert_eq!( &headers["Accept"], "text/html" );
    //     assert_eq!( &headers["X-Custom-Foo"], "bar" );
    // }

    let mut req = Request::builder()
        .version(Version::Http2)
        .method("GET")
        .url("/")
        .header(":scheme", "http")
        .header(":authority", "www.example.com");
    {
        // let headers = req.headers_mut().unwrap();
        // headers["AAAA"] = HeaderValue::Stand("ok");
        // let xx = &headers["Accept"];
        // let aaa = &headers["AAAA"];
        // println!("xxx = {:?}", xx);
        // println!("aaa = {:?}", aaa);

        // for value in headers.iter() {
        //     println!("____={:?}", value.0);
        //     println!("____={:?}", value.1);
        // }
        // assert_eq!( &headers["Accept"], "text/html" );
        // assert_eq!( &headers["X-Custom-Foo"], "bar" );
    }

    let mut rrr = req.body(()).unwrap();

    // let data = rrr.http2data().unwrap();

    // println!("req.httpdata() = {:?}", hex_debug_print(&data));

    // let mut decode = Decoder::new();
    // let mut buf = BinaryMut::from(data);
    // let result = decode.decode_with_cb(&mut buf, |n, v| {
    //     println!("n = {:?}, v = {}", n, v);
    // });

    // let mut index = Arc::new(HeaderIndex::new());
    // Arc::get_mut(&mut index).map(|v| {
    //     v.add_header(
    //         HeaderName::from_static("aaa"),
    //         HeaderValue::from_static("aa"),
    //     );
    // });

    // let xx = Arc::get_mut(&mut index);
    // println!("========={:?}", xx);
    // let xx111 = Arc::get_mut(&mut index);
    // println!("========={:?}", xx111);
    // // rrr.extensions_mut().insert(index);
    // // let new = rrr.extensions_mut().get_mut::<Arc<HeaderIndex>>();
    // // println!("========={:?}", new);

    // // if true {
    // //     return;
    // // }

    // let u = url::Builder::new()
    //     .scheme("https")
    //     .domain("www.baidu.com")
    //     .build()
    //     .unwrap();
    // println!("u = {}", u);

    // let response = Response::builder()
    //     .header("Accept", "text/html")
    //     .header("X-Custom-Foo", "bar")
    //     .body("my is web")
    //     .unwrap();

    // println!(
    //     "ssssssssssss {}",
    //     String::from_utf8_lossy(&response.httpdata().unwrap())
    // );

    let mut xx = HashMap::<(u32, u8), u8>::new();
    xx.insert((48, 5), 6);

    // xx.get(&b"xx");

    println!("aaa {:?}", xx.get(&(48, 5)));
    // let response = url::Builder::
}
