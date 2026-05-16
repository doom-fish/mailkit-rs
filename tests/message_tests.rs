mod support;

#[test]
fn message_exposes_authentication_results_and_attachments() {
    let message = support::sample_message();
    let auth = message
        .authentication_results()
        .expect("authentication results");
    assert_eq!(auth.len(), 1);
    assert_eq!(auth[0].methods[0].method, "dkim");

    let attachments = message.attachments().expect("attachments");
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].file_name.as_deref(), Some("report.txt"));
}
