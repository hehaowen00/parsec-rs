use parsing::prelude::*;

struct RequestBuilder<'a> {
    method: Option<&'a str>,
    path: Option<&'a str>,
    version: Option<&'a str>,
    headers: Vec<(&'a str, &'a str)>,
}

struct Request<'a> {
    method: &'a str,
    path: &'a str,
    version: &'a str,
    headers: Vec<(&'a str, &'a str)>,
}

impl<'a> RequestBuilder<'a> {
    pub fn new() -> Self {
        Self {
            method: None,
            path: None,
            version: None,
            headers: Vec::new(),
        }
    }

    pub fn build(self) -> Request<'a> {
        Request {
            method: self.method.unwrap(),
            path: self.path.unwrap(),
            version: self.version.unwrap(),
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
            builder.version = Some(version);
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

#[test]
fn http_test() {
    let bytes = "GET /index.html HTTP/1.1\r\nUser-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\nAccept-Language: en-us\r\nAccept-Encoding: gzip, deflate\r\nConnection: Keep-Alive\r\n\r\n"
        .to_stream();

    let http = http_parser();

    match http.parse(bytes) {
        Ok((xs, request)) => {
            assert_eq!(request.method, "GET");
            assert_eq!(request.path, "/index.html");
            assert_eq!(request.version, "HTTP/1.1");
            assert_eq!(
                request.headers,
                vec![
                    (
                        "User-Agent",
                        "Mozilla/4.0 (compatible; MSIE5.01; Windows NT)"
                    ),
                    ("Accept-Language", "en-us"),
                    ("Accept-Encoding", "gzip, deflate"),
                    ("Connection", "Keep-Alive")
                ]
            );
            assert_eq!(xs, &[]);
        }
        Err(xs) => {
            panic!("failed to parse bytes: {:?}", to_str(xs));
        }
    }
}
