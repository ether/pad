use pad::share::url_parse::parse_pad_url;

#[test]
fn http_url() {
    let r = parse_pad_url("http://example.com:9001/p/test").unwrap();
    assert_eq!(r.remote_base, "http://example.com:9001");
    assert_eq!(r.pad_id, "test");
}

#[test]
fn https_url_default_port() {
    let r = parse_pad_url("https://etherpad.org/p/my-pad").unwrap();
    assert_eq!(r.remote_base, "https://etherpad.org");
    assert_eq!(r.pad_id, "my-pad");
}

#[test]
fn rejects_non_pad_path() {
    assert!(parse_pad_url("http://example.com/foo").is_err());
}

#[test]
fn rejects_non_http_scheme() {
    assert!(parse_pad_url("ftp://example.com/p/x").is_err());
}
