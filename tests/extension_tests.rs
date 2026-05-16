use std::collections::BTreeMap;

use mailkit::{
    ComposeSession, ComposeSessionHandler, ComposeSessionHandlerDelegate, ContentBlocker,
    ContentBlockerProvider, DecodedMessage, DecodedMessageBanner, EmailAddress,
    ExtensionViewController, MailExtension, Message, MessageAction, MessageActionDecision,
    MessageActionExtension, MessageActionHandler, MessageEncoding, MessageEncodingResult,
    MessageSecurityHandler, MessageSecurityHandlerDelegate, MessageSecurityInformation,
    MessageSigner, OutgoingMessageCapabilities, OutgoingMessageEncodingStatus,
    RemoteContentBlocking,
};

struct StubComposeHandler;
impl ComposeSessionHandlerDelegate for StubComposeHandler {
    fn mail_compose_session_did_begin(
        &self,
        _session: &ComposeSession,
    ) -> Result<(), mailkit::MailKitError> {
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
    fn additional_headers_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<BTreeMap<String, Vec<String>>, mailkit::MailKitError> {
        Ok(BTreeMap::new())
    }
}

struct StubActionHandler;
impl MessageActionHandler for StubActionHandler {
    fn decide_action_for_message(
        &self,
        _message: &Message,
    ) -> Result<Option<MessageActionDecision>, mailkit::MailKitError> {
        Ok(Some(MessageActionDecision::action(
            &MessageAction::move_to_archive()?,
        )?))
    }
}

struct StubContentBlocker;
impl ContentBlockerProvider for StubContentBlocker {
    fn content_rules_json(&self) -> Result<Vec<u8>, mailkit::MailKitError> {
        Ok(b"[]".to_vec())
    }
}

struct StubSecurityHandler;
impl MessageSecurityHandlerDelegate for StubSecurityHandler {
    fn decoded_message_for_message_data(
        &self,
        _data: &[u8],
    ) -> Result<Option<DecodedMessage>, mailkit::MailKitError> {
        let email = EmailAddress::new("Alice Example <alice@example.com>")?;
        let signer = MessageSigner::new(&[&email], "Alice", Some(&[1]))?;
        let security = MessageSecurityInformation::new(
            &[&signer],
            true,
            None,
            None,
            RemoteContentBlocking {
                should_block: true,
                localized_reason: None,
            },
        )?;
        let banner = DecodedMessageBanner::new("Encrypted", "Details", true)?;
        Ok(Some(DecodedMessage::new(
            Some(b"hello"),
            &security,
            None,
            Some(&banner),
        )?))
    }
    fn encoding_status_for_message(
        &self,
        _message: &Message,
        _compose_context: &mailkit::ComposeContext,
    ) -> Result<OutgoingMessageEncodingStatus, mailkit::MailKitError> {
        let email = EmailAddress::new("Alice Example <alice@example.com>")?;
        OutgoingMessageEncodingStatus::new(
            OutgoingMessageCapabilities {
                can_sign: true,
                can_encrypt: true,
            },
            None,
            &[&email],
        )
    }
    fn encode_message(
        &self,
        _message: &Message,
        _compose_context: &mailkit::ComposeContext,
    ) -> Result<MessageEncodingResult, mailkit::MailKitError> {
        let encoded = mailkit::EncodedOutgoingMessage::new(
            b"raw",
            MessageEncoding {
                is_signed: true,
                is_encrypted: true,
            },
        )?;
        MessageEncodingResult::new(Some(&encoded), None, None)
    }
}

#[test]
fn mail_extension_reports_registered_handlers() {
    let extension = MailExtension::builder()
        .with_compose_session_handler(
            ComposeSessionHandler::new(StubComposeHandler).expect("compose"),
        )
        .with_message_action_handler(
            MessageActionExtension::new(StubActionHandler).expect("action"),
        )
        .with_content_blocker(ContentBlocker::new(StubContentBlocker).expect("blocker"))
        .with_message_security_handler(
            MessageSecurityHandler::new(StubSecurityHandler).expect("security"),
        )
        .build()
        .expect("extension");

    assert!(extension.has_compose_session_handler());
    assert!(extension.has_message_action_handler());
    assert!(extension.has_content_blocker());
    assert!(extension.has_message_security_handler());
}
