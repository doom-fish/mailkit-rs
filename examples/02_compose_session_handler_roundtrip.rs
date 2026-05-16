use std::collections::BTreeMap;

use mailkit::{
    compose_session_error_domain, AddressAnnotation, ComposeContextInfo, ComposeSession,
    ComposeSessionErrorCode, ComposeSessionHandler, ComposeSessionHandlerDelegate,
    ComposeSessionInfo, ComposeUserAction, ExtensionViewController, MessageEncryptionState,
    MessageInfo, MessageState, NSErrorInfo,
};

struct ExampleHandler;

impl ComposeSessionHandlerDelegate for ExampleHandler {
    fn mail_compose_session_did_begin(
        &self,
        session: &ComposeSession,
    ) -> Result<(), mailkit::MailKitError> {
        session.reload_session()?;
        Ok(())
    }

    fn mail_compose_session_did_end(
        &self,
        _session: &ComposeSession,
    ) -> Result<(), mailkit::MailKitError> {
        Ok(())
    }

    fn view_controller_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<ExtensionViewController, mailkit::MailKitError> {
        ExtensionViewController::new()
    }

    fn annotate_addresses_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<Vec<mailkit::AddressAnnotationMatch>, mailkit::MailKitError> {
        let annotation = AddressAnnotation::warning("Recipient key missing")?;
        Ok(vec![mailkit::AddressAnnotationMatch::new(
            "Bob Example <bob@example.com>",
            &annotation,
        )])
    }

    fn allow_message_send_for_session(&self, _session: &ComposeSession) -> Result<(), NSErrorInfo> {
        Err(NSErrorInfo::new(
            compose_session_error_domain(),
            ComposeSessionErrorCode::InvalidRecipients.as_code(),
            "Recipient key missing",
        ))
    }

    fn additional_headers_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<BTreeMap<String, Vec<String>>, mailkit::MailKitError> {
        Ok(BTreeMap::from([(
            "X-MailKit-Test".into(),
            vec!["compose".into()],
        )]))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let message = MessageInfo {
        state: MessageState::Draft,
        encryption_state: MessageEncryptionState::Unknown,
        subject: "Reply message".into(),
        from_address: mailkit::EmailAddressInfo {
            raw_string: "Alice Example <alice@example.com>".into(),
            address_string: Some("alice@example.com".into()),
        },
        to_addresses: vec![mailkit::EmailAddressInfo {
            raw_string: "Bob Example <bob@example.com>".into(),
            address_string: Some("bob@example.com".into()),
        }],
        cc_addresses: Vec::new(),
        bcc_addresses: Vec::new(),
        reply_to_addresses: Vec::new(),
        all_recipient_addresses: vec![mailkit::EmailAddressInfo {
            raw_string: "Bob Example <bob@example.com>".into(),
            address_string: Some("bob@example.com".into()),
        }],
        date_sent: None,
        date_received: None,
        headers: None,
        raw_data: None,
    };
    let context = ComposeContextInfo {
        context_id: "00000000-0000-0000-0000-000000000010".into(),
        original_message: None,
        action: ComposeUserAction::Reply,
        is_encrypted: false,
        should_encrypt: true,
        is_signed: false,
        should_sign: true,
    };
    let session = ComposeSession::new(ComposeSessionInfo {
        session_id: "00000000-0000-0000-0000-000000000011".into(),
        mail_message: message,
        compose_context: context,
    })?;
    let handler = ComposeSessionHandler::new(ExampleHandler)?;
    handler.mail_compose_session_did_begin(&session)?;
    let _controller = handler.view_controller_for_session(&session)?;
    let annotations = handler.annotate_addresses_for_session(&session)?;
    assert_eq!(annotations.len(), 1);
    assert!(handler.allow_message_send_for_session(&session).is_err());
    println!("compose-session handler validated a draft and produced annotations");
    Ok(())
}
