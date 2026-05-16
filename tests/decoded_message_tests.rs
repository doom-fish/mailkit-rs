mod support;

#[test]
fn decoded_message_preserves_security_and_banner_info() {
    let decoded = support::sample_decoded_message();
    assert_eq!(decoded.info().raw_data.as_deref(), Some(&b"hello"[..]));
    assert_eq!(
        decoded
            .info()
            .banner
            .as_ref()
            .map(|banner| banner.primary_action_title.as_str()),
        Some("Details")
    );
    assert!(decoded.info().security_information.is_encrypted);
}
