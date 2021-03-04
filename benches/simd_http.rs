#![allow(dead_code)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parsing::prelude::*;

struct RequestBuilder<'a> {
    method: Option<&'a str>,
    path: Option<&'a str>,
    http: Option<&'a str>,
    headers: Vec<(&'a str, &'a str)>,
}

struct Request<'a> {
    method: &'a str,
    path: &'a str,
    http: &'a str,
    headers: Vec<(&'a str, &'a str)>,
}

impl<'a> RequestBuilder<'a> {
    pub fn new() -> Self {
        Self {
            method: None,
            path: None,
            http: None,
            headers: Vec::new(),
        }
    }

    pub fn build(self) -> Request<'a> {
        Request {
            method: self.method.unwrap(),
            path: self.path.unwrap(),
            http: self.http.unwrap(),
            headers: self.headers,
        }
    }
}

fn http_parser<'a>() -> impl Parse<'a, Output = Request<'a>> {
    state(|| RequestBuilder::new())
        .then(parse_request())
        .map(|(mut builder, (method, path, version))| {
            builder.method = Some(method);
            builder.path = Some(path);
            builder.http = Some(version);
            builder
        })
        .then(parse_headers())
        .skip(slice(b"\r\n"))
        .map(|(mut builder, headers)| {
            builder.headers = headers;
            builder.build()
        })
}

fn parse_request<'a>() -> Cell<'a, impl Parse<'a, Output = (&'a str, &'a str, &'a str)>> {
    let method = slice(b"GET")
        | slice(b"HEAD")
        | slice(b"POST")
        | slice(b"PUT")
        | slice(b"DELETE")
        | slice(b"CONNECT")
        | slice(b"OPTIONS")
        | slice(b"TRACE")
        | slice(b"PATCH");

    let method = method.map(|bytes| to_str(bytes));
    let path = take_until_literal(b" ").map(|bytes| to_str(bytes));
    let version = take_until_literal(b"\r\n").map(|bytes| to_str(bytes));

    method
        .skip(chr(' '))
        .then(path)
        .skip(chr(' '))
        .then(version)
        .skip(slice(b"\r\n"))
        .map(|((a, b), c)| (a, b, c))
}

fn parse_headers<'a>() -> Cell<'a, impl Parse<'a, Output = Vec<(&'a str, &'a str)>>> {
    let header = take_until_literal(b":")
        .skip(slice(b": "))
        .then(take_until_literal(b"\r\n"))
        .map(|(key, value)| (to_str(key), to_str(value)))
        .skip(slice(b"\r\n"));

    many1(header)
}

fn to_str<'a>(bytes: &'a [u8]) -> &'a str {
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

pub fn http_bench(c: &mut Criterion) {
    let bytes = "GET /index.html HTTP/1.1\r\n\
        User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\n\
        Accept-Language: en-us\r\n\
        Accept-Encoding: gzip, deflate\r\n\
        Connection: Keep-Alive\r\n\r\n"
        .to_stream();
    let bytes = black_box(bytes);
    let parser = http_parser();

    c.bench_function("simd-http-parser", |b| {
        b.iter(|| {
            let res = parser.parse(bytes);
            assert!(res.is_ok());
        })
    });
}

criterion_group!(benches, http_bench);
criterion_main!(benches);
