#![allow(dead_code, clippy::missing_panics_doc, clippy::must_use_candidate)]

use std::collections::BTreeMap;

use mailkit::{
    ComposeContextInfo, ComposeSessionInfo, ComposeUserAction, DecodedMessage,
    DecodedMessageBanner, EmailAddress, EmailAddressInfo, EncodedOutgoingMessage, Message,
    MessageEncoding, MessageEncryptionState, MessageInfo, MessageSecurityInformation,
    MessageSigner, MessageState, OutgoingMessageCapabilities, OutgoingMessageEncodingStatus,
    RemoteContentBlocking,
};

pub fn sample_email() -> EmailAddress {
    EmailAddress::new("Alice Example <alice@example.com>").expect("sample email")
}

pub fn sample_email_info() -> EmailAddressInfo {
    EmailAddressInfo {
        raw_string: "Alice Example <alice@example.com>".into(),
        address_string: Some("alice@example.com".into()),
    }
}

pub fn sample_message_info() -> MessageInfo {
    MessageInfo {
        state: MessageState::Received,
        encryption_state: MessageEncryptionState::Encrypted,
        subject: "Quarterly report".into(),
        from_address: sample_email_info(),
        to_addresses: vec![EmailAddressInfo {
            raw_string: "Bob Example <bob@example.com>".into(),
            address_string: Some("bob@example.com".into()),
        }],
        cc_addresses: Vec::new(),
        bcc_addresses: Vec::new(),
        reply_to_addresses: Vec::new(),
        all_recipient_addresses: vec![EmailAddressInfo {
            raw_string: "Bob Example <bob@example.com>".into(),
            address_string: Some("bob@example.com".into()),
        }],
        date_sent: Some("2026-05-16T00:00:00Z".into()),
        date_received: Some("2026-05-16T00:00:01Z".into()),
        headers: Some(BTreeMap::from([(
            "Authentication-Results".into(),
            vec![
                "mx.example.com; dkim=pass header.i=@example.com; spf=pass smtp.mailfrom=example.com"
                    .into(),
            ],
        )])),
        raw_data: Some(
            concat!(
                "From: Alice Example <alice@example.com>\r\n",
                "To: Bob Example <bob@example.com>\r\n",
                "Subject: Quarterly report\r\n",
                "MIME-Version: 1.0\r\n",
                "Content-Type: multipart/mixed; boundary=frontier\r\n",
                "\r\n",
                "--frontier\r\n",
                "Content-Type: text/plain\r\n\r\nHello\r\n",
                "--frontier\r\n",
                "Content-Type: text/plain; name=report.txt\r\n",
                "Content-Disposition: attachment; filename=report.txt\r\n",
                "Content-Transfer-Encoding: 7bit\r\n\r\nreport body\r\n",
                "--frontier--\r\n"
            )
            .as_bytes()
            .to_vec(),
        ),
    }
}

pub fn sample_message() -> Message {
    Message::new(sample_message_info())
}

pub fn sample_compose_context_info() -> ComposeContextInfo {
    ComposeContextInfo {
        context_id: "00000000-0000-0000-0000-000000000001".into(),
        original_message: Some(Box::new(sample_message_info())),
        action: ComposeUserAction::Reply,
        is_encrypted: true,
        should_encrypt: true,
        is_signed: true,
        should_sign: true,
    }
}

pub fn sample_compose_session_info() -> ComposeSessionInfo {
    ComposeSessionInfo {
        session_id: "00000000-0000-0000-0000-000000000002".into(),
        mail_message: sample_message_info(),
        compose_context: sample_compose_context_info(),
    }
}

pub fn sample_signer() -> MessageSigner {
    let email = sample_email();
    MessageSigner::new(&[&email], "Alice Example", Some(&[1, 2, 3])).expect("sample signer")
}

pub fn sample_security_information() -> MessageSecurityInformation {
    let signer = sample_signer();
    MessageSecurityInformation::new(
        &[&signer],
        true,
        None,
        None,
        RemoteContentBlocking {
            should_block: true,
            localized_reason: Some("Remote content blocked".into()),
        },
    )
    .expect("sample security information")
}

pub fn sample_decoded_message() -> DecodedMessage {
    let security = sample_security_information();
    let banner = DecodedMessageBanner::new("Encrypted", "Details", true).expect("banner");
    DecodedMessage::new(Some(b"hello"), &security, Some(&[4, 5, 6]), Some(&banner))
        .expect("decoded message")
}

pub fn sample_encoded_message() -> EncodedOutgoingMessage {
    EncodedOutgoingMessage::new(
        b"raw message",
        MessageEncoding {
            is_signed: true,
            is_encrypted: true,
        },
    )
    .expect("encoded message")
}

pub fn sample_encoding_status() -> OutgoingMessageEncodingStatus {
    let email = sample_email();
    OutgoingMessageEncodingStatus::new(
        OutgoingMessageCapabilities {
            can_sign: true,
            can_encrypt: true,
        },
        None,
        &[&email],
    )
    .expect("encoding status")
}
