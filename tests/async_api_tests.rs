#![cfg(feature = "async")]

use mailkit::prelude::*;

struct SimpleContentBlockerProvider;

impl ContentBlockerProvider for SimpleContentBlockerProvider {
    fn content_rules_json(&self) -> Result<Vec<u8>, mailkit::MailKitError> {
        let rules: Vec<u8> = vec![];
        serde_json::to_vec(&rules).map_err(|e| mailkit::MailKitError::FrameworkError(e.to_string()))
    }
}

#[test]
fn test_async_content_blocker_creation() {
    let provider = SimpleContentBlockerProvider;
    let blocker = ContentBlocker::new(provider);
    assert!(blocker.is_ok());
}

#[test]
fn test_async_content_blocker_sync_access() {
    let provider = SimpleContentBlockerProvider;
    let blocker = ContentBlocker::new(provider).expect("Failed to create blocker");
    let rules = blocker.content_rules_json();
    assert!(rules.is_ok());
}

#[test]
fn test_simple_message_creation() {
    let from_info = mailkit::EmailAddressInfo {
        raw_string: "Test User <test@example.com>".to_string(),
        address_string: Some("test@example.com".to_string()),
    };

    let info = mailkit::MessageInfo {
        state: mailkit::MessageState::Received,
        encryption_state: mailkit::MessageEncryptionState::NotEncrypted,
        subject: "Test Subject".to_string(),
        from_address: from_info,
        to_addresses: vec![],
        cc_addresses: vec![],
        bcc_addresses: vec![],
        reply_to_addresses: vec![],
        all_recipient_addresses: vec![],
        date_sent: None,
        date_received: None,
        headers: None,
        raw_data: None,
    };
    let message = mailkit::Message::new(info);
    assert_eq!(message.info().subject, "Test Subject");
}

