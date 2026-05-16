use mailkit::{
    ComposeContext, ComposeContextInfo, ComposeUserAction, DecodedMessageBanner, EmailAddress,
    ExtensionViewController, Message, MessageEncoding, MessageEncodingResult,
    MessageEncryptionState, MessageInfo, MessageSecurityHandler, MessageSecurityHandlerDelegate,
    MessageSecurityInformation, MessageSigner, MessageState, OutgoingMessageCapabilities,
    OutgoingMessageEncodingStatus, RemoteContentBlocking,
};

struct ExampleSecurityHandler;

impl MessageSecurityHandlerDelegate for ExampleSecurityHandler {
    fn decoded_message_for_message_data(
        &self,
        _data: &[u8],
    ) -> Result<Option<mailkit::DecodedMessage>, mailkit::MailKitError> {
        let email = EmailAddress::new("Alice Example <alice@example.com>")?;
        let signer = MessageSigner::new(&[&email], "Alice Example", Some(&[1, 2, 3]))?;
        let security = MessageSecurityInformation::new(
            &[&signer],
            true,
            None,
            None,
            RemoteContentBlocking {
                should_block: true,
                localized_reason: Some("Remote content blocked".into()),
            },
        )?;
        let banner = DecodedMessageBanner::new("Encrypted", "Details", true)?;
        Ok(Some(mailkit::DecodedMessage::new(
            Some(b"hello"),
            &security,
            Some(&[4, 5]),
            Some(&banner),
        )?))
    }

    fn encoding_status_for_message(
        &self,
        _message: &Message,
        _compose_context: &ComposeContext,
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
        _compose_context: &ComposeContext,
    ) -> Result<MessageEncodingResult, mailkit::MailKitError> {
        let encoded = mailkit::EncodedOutgoingMessage::new(
            b"raw message",
            MessageEncoding {
                is_signed: true,
                is_encrypted: true,
            },
        )?;
        MessageEncodingResult::new(Some(&encoded), None, None)
    }

    fn extension_view_controller_for_signers(
        &self,
        _message_signers: &[mailkit::MessageSignerInfo],
    ) -> Result<Option<ExtensionViewController>, mailkit::MailKitError> {
        Ok(Some(ExtensionViewController::new()?))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = MessageSecurityHandler::new(ExampleSecurityHandler)?;
    let decoded = handler
        .decoded_message_for_message_data(b"raw")?
        .expect("decoded message");
    assert_eq!(decoded.info().raw_data.as_deref(), Some(&b"hello"[..]));

    let message = Message::new(MessageInfo {
        state: MessageState::Draft,
        encryption_state: MessageEncryptionState::Unknown,
        subject: "Draft".into(),
        from_address: mailkit::EmailAddressInfo {
            raw_string: "Alice Example <alice@example.com>".into(),
            address_string: Some("alice@example.com".into()),
        },
        to_addresses: Vec::new(),
        cc_addresses: Vec::new(),
        bcc_addresses: Vec::new(),
        reply_to_addresses: Vec::new(),
        all_recipient_addresses: Vec::new(),
        date_sent: None,
        date_received: None,
        headers: None,
        raw_data: Some(b"raw message".to_vec()),
    });
    let context = ComposeContext::new(ComposeContextInfo {
        context_id: "00000000-0000-0000-0000-000000000100".into(),
        original_message: None,
        action: ComposeUserAction::NewMessage,
        is_encrypted: false,
        should_encrypt: true,
        is_signed: false,
        should_sign: true,
    });
    let status = handler.encoding_status_for_message(&message, &context)?;
    assert!(status.info().capabilities.can_sign);
    let result = handler.encode_message(&message, &context)?;
    assert!(result.info().encoded_message.is_some());
    println!("message-security handler decoded and encoded a MailKit message");
    Ok(())
}
