use googletest::prelude::*;
use rust_petitparser::assert_success;
use rust_petitparser::prelude::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Authority {
    pub username: Option<String>,
    pub password: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UriParts {
    pub scheme: Option<String>,
    pub authority: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<String>,
    pub path: String,
    pub query: Option<String>,
    pub params: Vec<(String, Option<String>)>,
    pub fragment: Option<String>,
}

pub fn uri() -> impl Parser<UriParts> {
    seq!(
        seq!(scheme(), char(':')).opt(),
        seq!(string("//"), authority_span()).opt(),
        path_span(),
        seq!(char('?'), query_span()).opt(),
        seq!(char('#'), fragment_span()).opt(),
        => |scheme, authority, path, query, fragment| {
        let scheme = scheme.map(|(s, _)| s);
        let authority_str = authority.map(|(_, a)| a);
        let auth = parse_authority(authority_str.as_deref().unwrap_or(""));
        let query_str = query.map(|(_, q)| q);
        let params = parse_query(query_str.as_deref().unwrap_or(""));
        let fragment = fragment.map(|(_, f)| f);
        UriParts {
            scheme,
            authority: authority_str,
            username: auth.username,
            password: auth.password,
            hostname: auth.hostname,
            port: auth.port,
            path,
            query: query_str,
            params,
            fragment,
        }
    })
}

fn scheme() -> impl Parser<String> {
    pattern("^:/?#")
        .plus()
        .input_with_message("scheme".to_string())
}

fn authority_span() -> impl Parser<String> {
    pattern("^/?#")
        .star()
        .input_with_message("authority".to_string())
}

fn path_span() -> impl Parser<String> {
    pattern("^?#").star().input_with_message("path".to_string())
}

fn query_span() -> impl Parser<String> {
    pattern("^#").star().input_with_message("query".to_string())
}

fn fragment_span() -> impl Parser<String> {
    any().star().input_with_message("fragment".to_string())
}

pub fn authority() -> impl Parser<Authority> {
    seq!(credentials().opt(), hostname().opt(), port().opt()).map(
        |(credentials, hostname, port)| Authority {
            username: credentials.clone().map(|(u, _)| u),
            password: credentials.and_then(|(_, p)| p.map(|(_, pw)| pw)),
            hostname,
            port: port.map(|(_, p)| p),
        },
    )
}

pub fn parse_authority(input: &str) -> Authority {
    authority()
        .parse(input)
        .map(|s| s.value)
        .unwrap_or_default()
}

fn credentials() -> impl Parser<(String, Option<(char, String)>)> {
    seq!(
        username(),
        seq!(char(':'), password()).opt(),
        char('@') =>
        |u, p, _| (u, p)
    )
}

fn username() -> impl Parser<String> {
    pattern("^:@")
        .plus()
        .input_with_message("username".to_string())
}

fn password() -> impl Parser<String> {
    pattern("^@")
        .plus()
        .input_with_message("password".to_string())
}

fn hostname() -> impl Parser<String> {
    pattern("^:")
        .plus()
        .input_with_message("hostname".to_string())
}

fn port() -> impl Parser<(char, String)> {
    seq!(
        char(':'),
        digit().plus().input_with_message("port".to_string()),
    )
}

pub fn query() -> impl Parser<Vec<(String, Option<String>)>> {
    param()
        .plus_sep(char('&'), Trailing::Disallowed)
        .map(|params| {
            params
                .into_iter()
                .filter(|(key, value)| !key.is_empty() || value.is_some())
                .collect()
        })
}

pub fn parse_query(input: &str) -> Vec<(String, Option<String>)> {
    query().parse(input).map(|s| s.value).unwrap_or_default()
}

fn param() -> impl Parser<(String, Option<String>)> {
    seq!(param_key(), seq!(char('='), param_value()).opt(),
        => |key, value| (key, value.map(|(_, v)| v)),
    )
}

fn param_key() -> impl Parser<String> {
    pattern("^=&")
        .star()
        .input_with_message("param key".to_string())
}

fn param_value() -> impl Parser<String> {
    pattern("^&")
        .star()
        .input_with_message("param value".to_string())
}

#[gtest]
fn uri_with_fragment() {
    let p = uri().end();
    assert_success!(
        p,
        "http://www.ics.uci.edu/pub/ietf/uri/#Related",
        &UriParts {
            scheme: Some("http".to_string()),
            authority: Some("www.ics.uci.edu".to_string()),
            username: None,
            password: None,
            hostname: Some("www.ics.uci.edu".to_string()),
            port: None,
            path: "/pub/ietf/uri/".to_string(),
            query: None,
            params: vec![],
            fragment: Some("Related".to_string()),
        }
    );
}

#[gtest]
fn uri_with_query_params() {
    let p = uri().end();
    assert_success!(
        p,
        "http://a/b/c/d;e?f&g=h",
        &UriParts {
            scheme: Some("http".to_string()),
            authority: Some("a".to_string()),
            username: None,
            password: None,
            hostname: Some("a".to_string()),
            port: None,
            path: "/b/c/d;e".to_string(),
            query: Some("f&g=h".to_string()),
            params: vec![
                ("f".to_string(), None),
                ("g".to_string(), Some("h".to_string())),
            ],
            fragment: None,
        }
    );
}

#[gtest]
fn uri_with_port_and_odd_query() {
    let p = uri().end();
    assert_success!(
        p,
        "ftp://www.example.org:22/foo bar/zork<>?\\^`{|}",
        &UriParts {
            scheme: Some("ftp".to_string()),
            authority: Some("www.example.org:22".to_string()),
            username: None,
            password: None,
            hostname: Some("www.example.org".to_string()),
            port: Some("22".to_string()),
            path: "/foo bar/zork<>".to_string(),
            query: Some("\\^`{|}".to_string()),
            params: vec![("\\^`{|}".to_string(), None)],
            fragment: None,
        }
    );
}

#[gtest]
fn uri_without_authority() {
    let p = uri().end();
    assert_success!(
        p,
        "data:text/plain;charset=iso-8859-7,hallo",
        &UriParts {
            scheme: Some("data".to_string()),
            authority: None,
            username: None,
            password: None,
            hostname: None,
            port: None,
            path: "text/plain;charset=iso-8859-7,hallo".to_string(),
            query: None,
            params: vec![],
            fragment: None,
        }
    );
}

#[gtest]
fn uri_with_unicode_hostname() {
    let p = uri().end();
    assert_success!(
        p,
        "https://www.übermäßig.de/müßiggänger",
        &UriParts {
            scheme: Some("https".to_string()),
            authority: Some("www.übermäßig.de".to_string()),
            username: None,
            password: None,
            hostname: Some("www.übermäßig.de".to_string()),
            port: None,
            path: "/müßiggänger".to_string(),
            query: None,
            params: vec![],
            fragment: None,
        },
        36
    );
}

#[gtest]
fn uri_without_slashes() {
    let p = uri().end();
    assert_success!(
        p,
        "http:test",
        &UriParts {
            scheme: Some("http".to_string()),
            authority: None,
            username: None,
            password: None,
            hostname: None,
            port: None,
            path: "test".to_string(),
            query: None,
            params: vec![],
            fragment: None,
        }
    );
}

#[gtest]
fn uri_with_backslashes_in_path() {
    let p = uri().end();
    assert_success!(
        p,
        "file:c:\\\\foo\\\\bar.html",
        &UriParts {
            scheme: Some("file".to_string()),
            authority: None,
            username: None,
            password: None,
            hostname: None,
            port: None,
            path: "c:\\\\foo\\\\bar.html".to_string(),
            query: None,
            params: vec![],
            fragment: None,
        }
    );
}

#[gtest]
fn uri_with_credentials() {
    let p = uri().end();
    assert_success!(
        p,
        "file://foo:bar@localhost/test",
        &UriParts {
            scheme: Some("file".to_string()),
            authority: Some("foo:bar@localhost".to_string()),
            username: Some("foo".to_string()),
            password: Some("bar".to_string()),
            hostname: Some("localhost".to_string()),
            port: None,
            path: "/test".to_string(),
            query: None,
            params: vec![],
            fragment: None,
        }
    );
}

#[gtest]
fn uri_regex_smoke_test() {
    let p = uri().end();
    let inputs = [
        "http://foo.com/blah_blah",
        "http://foo.com/blah_blah/",
        "http://foo.com/blah_blah_(wikipedia)",
        "http://foo.com/blah_blah_(wikipedia)_(again)",
        "http://www.example.com/wpstyle/?p=364",
        "https://www.example.com/foo/?bar=baz&inga=42&quux",
        "http://✪df.ws/123",
        "http://userid:password@example.com:8080",
        "http://userid:password@example.com:8080/",
        "http://userid@example.com",
        "http://userid@example.com/",
        "http://userid@example.com:8080",
        "http://userid@example.com:8080/",
        "http://userid:password@example.com",
        "http://userid:password@example.com/",
        "http://142.42.1.1/",
        "http://142.42.1.1:8080/",
        "http://➡.ws/䨹",
        "http://⌘.ws",
        "http://⌘.ws/",
        "http://foo.com/blah_(wikipedia)#cite-1",
        "http://foo.com/blah_(wikipedia)_blah#cite-1",
        "http://foo.com/unicode_(✪)_in_parens",
        "http://foo.com/(something)?after=parens",
        "http://☺.damowmow.com/",
        "http://code.google.com/events/#&product=browser",
        "http://j.mp",
        "ftp://foo.bar/baz",
        "http://foo.bar/?q=Test%20URL-encoded%20stuff",
        "http://مثال.إختبار",
        "http://例子.测试",
        "http://उदाहरण.परीक्षा",
        "http://-.~_!$&'()*+,;=:%40:80%2f::::::@example.com",
        "http://1337.net",
        "http://a.b-c.de",
        "http://223.255.255.254",
    ];
    for input in inputs {
        let result = p.parse(input);
        assert_that!(result.is_ok(), eq(true), "failed to parse: {input}");
    }
}
