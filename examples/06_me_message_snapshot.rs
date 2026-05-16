use std::collections::BTreeMap;

use mailkit::{AuthenticationResult, Message, MessageEncryptionState, MessageInfo, MessageState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let message = Message::new(MessageInfo {
        state: MessageState::Received,
        encryption_state: MessageEncryptionState::Encrypted,
        subject: "Quarterly report".into(),
        from_address: mailkit::EmailAddressInfo {
            raw_string: "Alice Example <alice@example.com>".into(),
            address_string: Some("alice@example.com".into()),
        },
        to_addresses: Vec::new(),
        cc_addresses: Vec::new(),
        bcc_addresses: Vec::new(),
        reply_to_addresses: Vec::new(),
        all_recipient_addresses: Vec::new(),
        date_sent: Some("2026-05-16T00:00:00Z".into()),
        date_received: Some("2026-05-16T00:00:01Z".into()),
        headers: Some(BTreeMap::from([(
            "Authentication-Results".into(),
            vec![
                "mx.example.com; dkim=pass header.i=@example.com; spf=pass smtp.mailfrom=example.com"
                    .into(),
            ],
        )])),
        raw_data: None,
    });
    let auth = message.authentication_results()?;
    assert_eq!(auth[0].authserv_id, "mx.example.com");
    assert_eq!(
        AuthenticationResult::parse_headers(message.info().headers.as_ref())?.len(),
        1
    );
    println!(
        "MEMessage helper exposed {} authentication result(s)",
        auth.len()
    );
    Ok(())
}
