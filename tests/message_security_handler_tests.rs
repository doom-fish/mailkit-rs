mod support;

use mailkit::{
    message_security_error_domain, ComposeContext, ExtensionViewController, MessageSecurityHandler,
    MessageSecurityHandlerDelegate, MessageSignerInfo,
};

struct StaticSecurityHandler;

impl MessageSecurityHandlerDelegate for StaticSecurityHandler {
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
        message_signers: &[MessageSignerInfo],
    ) -> Result<Option<ExtensionViewController>, mailkit::MailKitError> {
        assert_eq!(message_signers.len(), 1);
        Ok(Some(ExtensionViewController::new()?))
    }

    fn extension_view_controller_for_message_context(
        &self,
        context: &[u8],
    ) -> Result<Option<ExtensionViewController>, mailkit::MailKitError> {
        assert_eq!(context, b"context");
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
fn message_security_handler_round_trips_decoder_encoder_and_ui_hooks() {
    let handler =
        MessageSecurityHandler::new(StaticSecurityHandler).expect("message security handler");
    let decoded = handler
        .decoded_message_for_message_data(b"raw")
        .expect("decode result")
        .expect("decoded message");
    assert_eq!(decoded.info().raw_data.as_deref(), Some(&b"hello"[..]));

    let message = support::sample_message();
    let context = mailkit::ComposeContext::new(support::sample_compose_context_info());
    let status = handler
        .encoding_status_for_message(&message, &context)
        .expect("encoding status");
    assert!(status.info().capabilities.can_sign);

    let result = handler
        .encode_message(&message, &context)
        .expect("encode message");
    assert!(result.info().encoded_message.is_some());

    let signer_info = support::sample_signer().info().clone();
    let _signers_controller = handler
        .extension_view_controller_for_signers(&[signer_info])
        .expect("signers controller")
        .expect("signers view controller");
    let _context_controller = handler
        .extension_view_controller_for_message_context(b"context")
        .expect("context controller")
        .expect("context view controller");
    let _primary_action_controller = handler
        .primary_action_clicked_for_message_context(b"context")
        .expect("primary action controller")
        .expect("primary action view controller");
    assert!(!message_security_error_domain().is_empty());
}
