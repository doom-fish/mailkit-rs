use mailkit::AuthenticationResult;

#[test]
fn authentication_result_parses_header_methods() {
    let result = AuthenticationResult::parse(
        "mx.example.com; dkim=pass header.i=@example.com; spf=pass smtp.mailfrom=example.com",
    )
    .expect("authentication result");
    assert_eq!(result.authserv_id, "mx.example.com");
    assert_eq!(result.methods.len(), 2);
    assert_eq!(result.methods[1].properties["smtp.mailfrom"], "example.com");
}
