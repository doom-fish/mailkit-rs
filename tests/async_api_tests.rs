#![cfg(feature = "async")]

mod support;

use std::collections::BTreeMap;

use mailkit::async_api::{
    AsyncComposeSessionHandler, AsyncExtensionManager, AsyncMessageSecurityHandler,
};
use mailkit::prelude::*;
use mailkit::{
    compose_session_error_domain, message_security_error_domain, ComposeSession,
    ComposeSessionHandler, ComposeSessionHandlerDelegate, ExtensionViewController,
    MessageSecurityHandler, MessageSecurityHandlerDelegate, MessageSignerInfo, NSErrorInfo,
};

struct SimpleContentBlockerProvider;

impl ContentBlockerProvider for SimpleContentBlockerProvider {
    fn content_rules_json(&self) -> Result<Vec<u8>, mailkit::MailKitError> {
        let rules: Vec<u8> = vec![];
        serde_json::to_vec(&rules).map_err(|e| mailkit::MailKitError::FrameworkError(e.to_string()))
    }
}

struct AsyncComposeHandler;

impl ComposeSessionHandlerDelegate for AsyncComposeHandler {
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

    fn annotate_addresses_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<Vec<mailkit::AddressAnnotationMatch>, mailkit::MailKitError> {
        let annotation = AddressAnnotation::warning("Missing recipient certificate")?;
        Ok(vec![mailkit::AddressAnnotationMatch::new(
            "Bob Example <bob@example.com>",
            &annotation,
        )])
    }

    fn allow_message_send_for_session(&self, _session: &ComposeSession) -> Result<(), NSErrorInfo> {
        Err(NSErrorInfo::new(
            compose_session_error_domain(),
            mailkit::ComposeSessionErrorCode::InvalidRecipients.as_code(),
            "Missing recipient certificate",
        ))
    }

    fn additional_headers_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<BTreeMap<String, Vec<String>>, mailkit::MailKitError> {
        Ok(BTreeMap::new())
    }
}

struct AsyncSecurityHandler;

impl MessageSecurityHandlerDelegate for AsyncSecurityHandler {
    fn decoded_message_for_message_data(
        &self,
        _data: &[u8],
    ) -> Result<Option<mailkit::DecodedMessage>, mailkit::MailKitError> {
        Ok(Some(support::sample_decoded_message()))
    }

    fn encoding_status_for_message(
        &self,
        message: &mailkit::Message,
        compose_context: &ComposeContext,
    ) -> Result<mailkit::OutgoingMessageEncodingStatus, mailkit::MailKitError> {
        assert_eq!(message.info().subject, "Quarterly report");
        assert!(compose_context.info().should_encrypt);
        Ok(support::sample_encoding_status())
    }

    fn encode_message(
        &self,
        _message: &mailkit::Message,
        _compose_context: &ComposeContext,
    ) -> Result<mailkit::MessageEncodingResult, mailkit::MailKitError> {
        let encoded = support::sample_encoded_message();
        mailkit::MessageEncodingResult::new(Some(&encoded), None, None)
    }

    fn extension_view_controller_for_signers(
        &self,
        _message_signers: &[MessageSignerInfo],
    ) -> Result<Option<ExtensionViewController>, mailkit::MailKitError> {
        Ok(Some(ExtensionViewController::new()?))
    }

    fn primary_action_clicked_for_message_context(
        &self,
        context: &[u8],
    ) -> Result<Option<ExtensionViewController>, mailkit::MailKitError> {
        assert_eq!(context, b"context");
        Ok(Some(ExtensionViewController::new()?))
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

#[test]
fn test_async_compose_session_handler_roundtrip() {
    let handler = ComposeSessionHandler::new(AsyncComposeHandler).expect("compose handler");
    let session =
        ComposeSession::new(support::sample_compose_session_info()).expect("compose session");

    let annotations = pollster::block_on(
        AsyncComposeSessionHandler::annotate_addresses_for_session(&handler, &session),
    )
    .expect("annotations");
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].raw_string, "Bob Example <bob@example.com>");

    let send_error = pollster::block_on(
        AsyncComposeSessionHandler::allow_message_send_for_session(&handler, &session),
    )
    .expect_err("send blocked");
    assert!(send_error
        .message()
        .contains("Missing recipient certificate"));
}

#[test]
fn test_async_message_security_handler_roundtrip() {
    let handler =
        MessageSecurityHandler::new(AsyncSecurityHandler).expect("message security handler");
    let message = support::sample_message();
    let context = ComposeContext::new(support::sample_compose_context_info());

    let status = pollster::block_on(AsyncMessageSecurityHandler::encoding_status_for_message(
        &handler, &message, &context,
    ))
    .expect("encoding status");
    assert!(status.info().capabilities.can_sign);

    let result = pollster::block_on(AsyncMessageSecurityHandler::encode_message(
        &handler, &message, &context,
    ))
    .expect("encode message");
    assert!(result.info().encoded_message.is_some());

    let controller = pollster::block_on(
        AsyncMessageSecurityHandler::primary_action_clicked_for_message_context(
            &handler, b"context",
        ),
    )
    .expect("primary action controller");
    assert!(controller.is_some());
    assert!(!message_security_error_domain().is_empty());
}

#[test]
fn test_async_extension_manager_api_surface_is_available() {
    let _: fn() -> mailkit::async_api::ReloadVisibleMessagesFuture =
        AsyncExtensionManager::reload_visible_messages;
    let _: fn(&str) -> mailkit::async_api::ReloadContentBlockerFuture =
        AsyncExtensionManager::reload_content_blocker;
}

struct FailingSecurityHandler;

impl MessageSecurityHandlerDelegate for FailingSecurityHandler {
    fn decoded_message_for_message_data(
        &self,
        _data: &[u8],
    ) -> Result<Option<mailkit::DecodedMessage>, mailkit::MailKitError> {
        Ok(None)
    }

    fn encoding_status_for_message(
        &self,
        _message: &mailkit::Message,
        _compose_context: &ComposeContext,
    ) -> Result<mailkit::OutgoingMessageEncodingStatus, mailkit::MailKitError> {
        Err(mailkit::MailKitError::FrameworkError(
            "token removed".into(),
        ))
    }

    fn encode_message(
        &self,
        _message: &mailkit::Message,
        _compose_context: &ComposeContext,
    ) -> Result<mailkit::MessageEncodingResult, mailkit::MailKitError> {
        Err(mailkit::MailKitError::FrameworkError(
            "token removed".into(),
        ))
    }
}

#[test]
fn test_async_message_security_handler_fails_closed() {
    let handler = MessageSecurityHandler::new(FailingSecurityHandler).expect("handler");
    let message = support::sample_message();
    let context = ComposeContext::new(support::sample_compose_context_info());

    let result = pollster::block_on(AsyncMessageSecurityHandler::encode_message(
        &handler, &message, &context,
    ))
    .expect("Mail always receives an encoding result");
    let info = result.info();
    assert!(info.encoded_message.is_none());
    for error in [&info.signing_error, &info.encryption_error] {
        let error = error.as_ref().expect("encoding error");
        assert_eq!(
            error.message_security_error_code(),
            Some(mailkit::MessageSecurityErrorCode::EncodingError)
        );
        assert!(error.localized_description.contains("token removed"));
    }

    let status = pollster::block_on(AsyncMessageSecurityHandler::encoding_status_for_message(
        &handler, &message, &context,
    ))
    .expect("Mail always receives a status");
    assert!(!status.info().capabilities.can_sign);
    assert!(status
        .info()
        .security_error
        .as_ref()
        .is_some_and(|error| error.localized_description.contains("token removed")));
}
