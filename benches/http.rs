#![allow(dead_code)]

use criterion::{criterion_group, criterion_main, Criterion};
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
    State::new(|| RequestBuilder::new())
        .then(parse_request())
        .map(|(mut builder, (method, path, version))| {
            builder.method = Some(method);
            builder.path = Some(path);
            builder.http = Some(version);
            builder
        })
        .then(parse_headers())
        .map(|(mut builder, headers)| {
            builder.headers = headers;
            builder
        })
        .skip(Str::new("\r\n"))
        .map(|builder| builder.build())
}

fn parse_request<'a>() -> impl Parse<'a, Output = (&'a str, &'a str, &'a str)> {
    let method = Str::new("GET")
        | Str::new("HEAD")
        | Str::new("POST")
        | Str::new("PUT")
        | Str::new("DELETE")
        | Str::new("CONNECT")
        | Str::new("OPTIONS")
        | Str::new("TRACE")
        | Str::new("PATCH");

    let path = TakeUntil::new(Char::new(' ')).map(|bytes| to_str(bytes));

    let version = TakeUntil::new(Str::new("\r\n")).map(|bytes| to_str(bytes));

    method
        .skip(Char::new(' '))
        .then(path)
        .skip(Char::new(' '))
        .then(version)
        .skip(Str::new("\r\n"))
        .map(|((a, b), c)| (a, b, c))
}

fn parse_headers<'a>() -> impl Parse<'a, Output = Vec<(&'a str, &'a str)>> {
    let header = TakeUntil::new(Char::new(':'))
        .skip(Str::new(": "))
        .then(TakeUntil::new(Str::new("\r\n")))
        .map(|(key, value)| (to_str(key), to_str(value)))
        .skip(Str::new("\r\n"));

    Many1::new(header)
}

fn to_str<'a>(bytes: &'a [u8]) -> &'a str {
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

fn http_bench(c: &mut Criterion) {
    let bytes = "GET /index.html HTTP/1.1\r\nUser-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\nAccept-Language: en-us\r\nAccept-Encoding: gzip, deflate\r\nConnection: Keep-Alive\r\n\r\n"
        .to_stream();

    let http = http_parser();

    c.bench_function("http-parser", |b| {
        b.iter(|| {
            let res = http.parse(bytes);
            assert!(res.is_ok());
        })
    });
}

criterion_group!(benches, http_bench);
criterion_main!(benches);
