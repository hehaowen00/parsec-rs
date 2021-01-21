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
    state(|| RequestBuilder::new())
        .then(cell(parse_request()))
        .map(|(mut builder, (method, path, version))| {
            builder.method = Some(method);
            builder.path = Some(path);
            builder.http = Some(version);
            builder
        })
        .then(cell(parse_headers()))
        .map(|(mut builder, headers)| {
            builder.headers = headers;
            builder
        })
        .skip(str_("\r\n"))
        .map(|builder| builder.build())
}

fn parse_request<'a>() -> impl Parse<'a, Output = (&'a str, &'a str, &'a str)> {
    let method = str_("GET")
        | str_("HEAD")
        | str_("POST")
        | str_("PUT")
        | str_("DELETE")
        | str_("CONNECT")
        | str_("OPTIONS")
        | str_("TRACE")
        | str_("PATCH");

    let path = take_until(char_(' ')).map(|bytes| to_str(bytes));

    let version = take_until(str_("\r\n")).map(|bytes| to_str(bytes));

    method
        .skip(char_(' '))
        .then(path)
        .skip(char_(' '))
        .then(version)
        .skip(str_("\r\n"))
        .map(|((a, b), c)| (a, b, c))
}

fn parse_headers<'a>() -> impl Parse<'a, Output = Vec<(&'a str, &'a str)>> {
    let header = take_until(char_(':'))
        .skip(str_(": "))
        .then(take_until(str_("\r\n")))
        .map(|(key, value)| (to_str(key), to_str(value)))
        .skip(str_("\r\n"));

    many1(header)
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
