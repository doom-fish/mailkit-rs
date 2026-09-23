mod support;

use std::sync::{Arc, Mutex};

use mailkit::{
    message_security_error_domain, ComposeContext, DecodedMessage, EncodedOutgoingMessage,
    MailKitError, Message, MessageEncoding, MessageEncodingResult, MessageSecurityErrorCode,
    MessageSecurityHandler, MessageSecurityHandlerDelegate, NSErrorInfo,
    OutgoingMessageEncodingStatus,
};

#[derive(Clone, Copy)]
enum Outcome {
    Fail,
    Panic,
    Nothing,
    SignedOnly,
    DelegateSigningError,
}

struct ScriptedHandler {
    outcome: Outcome,
}

impl MessageSecurityHandlerDelegate for ScriptedHandler {
    fn decoded_message_for_message_data(
        &self,
        _data: &[u8],
    ) -> Result<Option<DecodedMessage>, MailKitError> {
        match self.outcome {
            Outcome::Fail => Err(MailKitError::FrameworkError(
                "no private key for bob@example.com".into(),
            )),
            Outcome::Panic => panic!("decoder exploded"),
            _ => Ok(None),
        }
    }

    fn encoding_status_for_message(
        &self,
        _message: &Message,
        _compose_context: &ComposeContext,
    ) -> Result<OutgoingMessageEncodingStatus, MailKitError> {
        match self.outcome {
            Outcome::Panic => panic!("status exploded"),
            _ => Err(MailKitError::FrameworkError("keychain is locked".into())),
        }
    }

    fn encode_message(
        &self,
        _message: &Message,
        _compose_context: &ComposeContext,
    ) -> Result<MessageEncodingResult, MailKitError> {
        match self.outcome {
            Outcome::Fail => Err(MailKitError::FrameworkError(
                "no certificate for alice@example.com".into(),
            )),
            Outcome::Panic => panic!("encoder exploded"),
            Outcome::Nothing => MessageEncodingResult::new(None, None, None),
            Outcome::SignedOnly => {
                let encoded = EncodedOutgoingMessage::new(
                    b"signed",
                    MessageEncoding {
                        is_signed: true,
                        is_encrypted: false,
                    },
                )?;
                MessageEncodingResult::new(Some(&encoded), None, None)
            }
            Outcome::DelegateSigningError => MessageEncodingResult::new(
                None,
                Some(&NSErrorInfo::new(
                    "com.example.smime",
                    7,
                    "certificate expired",
                )),
                None,
            ),
        }
    }
}

fn context(should_sign: bool, should_encrypt: bool) -> ComposeContext {
    let mut info = support::sample_compose_context_info();
    info.should_sign = should_sign;
    info.should_encrypt = should_encrypt;
    ComposeContext::new(info)
}

fn encode(
    outcome: Outcome,
    should_sign: bool,
    should_encrypt: bool,
) -> mailkit::MessageEncodingResultInfo {
    let handler = MessageSecurityHandler::new(ScriptedHandler { outcome }).expect("handler");
    handler
        .encode_message(
            &support::sample_message(),
            &context(should_sign, should_encrypt),
        )
        .expect("Mail always receives an encoding result")
        .info()
        .clone()
}

fn assert_security_error(error: Option<&NSErrorInfo>, code: MessageSecurityErrorCode, text: &str) {
    let error = error.expect("security error");
    assert_eq!(error.domain, message_security_error_domain());
    assert_eq!(error.message_security_error_code(), Some(code));
    assert!(
        error.localized_description.contains(text),
        "{:?} does not mention {text:?}",
        error.localized_description
    );
}

#[test]
fn encode_error_fails_closed_for_both_requests() {
    let result = encode(Outcome::Fail, true, true);
    assert!(result.encoded_message.is_none());
    assert_security_error(
        result.signing_error.as_ref(),
        MessageSecurityErrorCode::EncodingError,
        "no certificate for alice@example.com",
    );
    assert_security_error(
        result.encryption_error.as_ref(),
        MessageSecurityErrorCode::EncodingError,
        "no certificate for alice@example.com",
    );
}

#[test]
fn encode_panic_fails_closed() {
    let result = encode(Outcome::Panic, true, false);
    assert!(result.encoded_message.is_none());
    assert_security_error(
        result.signing_error.as_ref(),
        MessageSecurityErrorCode::EncodingError,
        "panicked",
    );
}

#[test]
fn encode_error_reports_only_the_requested_operation() {
    let signing = encode(Outcome::Fail, true, false);
    assert!(signing.signing_error.is_some());
    assert!(signing.encryption_error.is_none());

    let encryption = encode(Outcome::Fail, false, true);
    assert!(encryption.signing_error.is_none());
    assert!(encryption.encryption_error.is_some());
}

#[test]
fn encode_error_fails_closed_even_without_a_request() {
    let result = encode(Outcome::Fail, false, false);
    assert!(result.encoded_message.is_none());
    assert!(result.signing_error.is_some());
    assert!(result.encryption_error.is_some());
}

#[test]
fn empty_result_for_a_signing_request_fails_closed() {
    let result = encode(Outcome::Nothing, true, false);
    assert!(result.encoded_message.is_none());
    assert_security_error(
        result.signing_error.as_ref(),
        MessageSecurityErrorCode::EncodingError,
        "did not sign",
    );
    assert!(result.encryption_error.is_none());
}

#[test]
fn unencrypted_result_for_an_encryption_request_fails_closed() {
    let result = encode(Outcome::SignedOnly, true, true);
    assert!(result.encoded_message.is_none());
    assert_security_error(
        result.encryption_error.as_ref(),
        MessageSecurityErrorCode::EncodingError,
        "did not encrypt",
    );
}

#[test]
fn empty_result_passes_when_nothing_was_requested() {
    let result = encode(Outcome::Nothing, false, false);
    assert!(result.encoded_message.is_none());
    assert!(result.signing_error.is_none());
    assert!(result.encryption_error.is_none());
}

#[test]
fn delegate_reported_errors_pass_through() {
    let result = encode(Outcome::DelegateSigningError, true, false);
    let error = result.signing_error.expect("delegate signing error");
    assert_eq!(error.domain, "com.example.smime");
    assert_eq!(error.code, 7);
    assert_eq!(error.localized_description, "certificate expired");
    assert!(result.encryption_error.is_none());
}

#[test]
fn status_error_surfaces_as_security_error() {
    for outcome in [Outcome::Fail, Outcome::Panic] {
        let handler = MessageSecurityHandler::new(ScriptedHandler { outcome }).expect("handler");
        let status = handler
            .encoding_status_for_message(&support::sample_message(), &context(true, true))
            .expect("Mail always receives a status");
        let info = status.info();
        assert!(!info.capabilities.can_sign);
        assert!(!info.capabilities.can_encrypt);
        assert!(info.addresses_failing_encryption.is_empty());
        let expected = match outcome {
            Outcome::Panic => "panicked",
            _ => "keychain is locked",
        };
        assert_security_error(
            info.security_error.as_ref(),
            MessageSecurityErrorCode::EncodingError,
            expected,
        );
    }
}

#[test]
fn decode_error_surfaces_as_decoding_error() {
    for (outcome, expected) in [
        (Outcome::Fail, "no private key for bob@example.com"),
        (Outcome::Panic, "panicked"),
    ] {
        let handler = MessageSecurityHandler::new(ScriptedHandler { outcome }).expect("handler");
        let decoded = handler
            .decoded_message_for_message_data(b"Content-Type: application/pkcs7-mime\r\n\r\n")
            .expect("decode result")
            .expect("Mail receives a failed decoded message");
        let info = decoded.info();
        assert!(info.raw_data.is_none());
        assert!(info.security_information.remote_content.should_block);
        assert_security_error(
            info.security_information.signing_error.as_ref(),
            MessageSecurityErrorCode::DecodingError,
            expected,
        );
        assert_security_error(
            info.security_information.encryption_error.as_ref(),
            MessageSecurityErrorCode::DecodingError,
            expected,
        );
    }
}

#[test]
fn decode_none_means_the_extension_is_not_needed() {
    let handler = MessageSecurityHandler::new(ScriptedHandler {
        outcome: Outcome::Nothing,
    })
    .expect("handler");
    assert!(handler
        .decoded_message_for_message_data(b"plain message")
        .expect("decode result")
        .is_none());
}

fn binary(len: usize, seed: u8) -> Vec<u8> {
    (0..=255u8)
        .cycle()
        .skip(usize::from(seed))
        .step_by(3)
        .take(len)
        .collect()
}

#[derive(Default)]
struct Seen {
    decode_input: Option<Vec<u8>>,
    message_raw: Option<Vec<u8>>,
    original_raw: Option<Vec<u8>>,
}

struct RecordingHandler {
    seen: Arc<Mutex<Seen>>,
    decoded: Vec<u8>,
    encoded: Vec<u8>,
}

impl MessageSecurityHandlerDelegate for RecordingHandler {
    fn decoded_message_for_message_data(
        &self,
        data: &[u8],
    ) -> Result<Option<DecodedMessage>, MailKitError> {
        self.seen.lock().unwrap().decode_input = Some(data.to_vec());
        let security = support::sample_security_information();
        Ok(Some(DecodedMessage::new(
            Some(self.decoded.as_slice()),
            &security,
            None,
            None,
        )?))
    }

    fn encoding_status_for_message(
        &self,
        _message: &Message,
        _compose_context: &ComposeContext,
    ) -> Result<OutgoingMessageEncodingStatus, MailKitError> {
        Ok(support::sample_encoding_status())
    }

    fn encode_message(
        &self,
        message: &Message,
        compose_context: &ComposeContext,
    ) -> Result<MessageEncodingResult, MailKitError> {
        let mut seen = self.seen.lock().unwrap();
        seen.message_raw.clone_from(&message.info().raw_data);
        seen.original_raw = compose_context
            .info()
            .original_message
            .as_ref()
            .and_then(|original| original.raw_data.clone());
        drop(seen);
        let encoded = EncodedOutgoingMessage::new(
            &self.encoded,
            MessageEncoding {
                is_signed: true,
                is_encrypted: true,
            },
        )?;
        MessageEncodingResult::new(Some(&encoded), None, None)
    }
}

#[test]
fn mime_bytes_cross_the_bridge_unchanged() {
    let seen = Arc::new(Mutex::new(Seen::default()));
    let decoded = binary(64 * 1024, 3);
    let encoded = binary(256 * 1024, 5);
    let handler = MessageSecurityHandler::new(RecordingHandler {
        seen: Arc::clone(&seen),
        decoded: decoded.clone(),
        encoded: encoded.clone(),
    })
    .expect("handler");

    let incoming = binary(1024 * 1024, 7);
    let result = handler
        .decoded_message_for_message_data(&incoming)
        .expect("decode result")
        .expect("decoded message");
    assert_eq!(
        seen.lock().unwrap().decode_input.as_deref(),
        Some(&incoming[..])
    );
    assert_eq!(result.info().raw_data.as_deref(), Some(&decoded[..]));

    handler
        .decoded_message_for_message_data(&[])
        .expect("empty decode result");
    assert_eq!(seen.lock().unwrap().decode_input.as_deref(), Some(&[][..]));

    let mut message_info = support::sample_message_info();
    let outgoing = binary(512 * 1024, 11);
    message_info.raw_data = Some(outgoing.clone());
    let mut context_info = support::sample_compose_context_info();
    let original = binary(4096, 13);
    context_info
        .original_message
        .as_mut()
        .expect("original message")
        .raw_data = Some(original.clone());

    let result = handler
        .encode_message(
            &Message::new(message_info),
            &ComposeContext::new(context_info),
        )
        .expect("encoding result");
    let seen = seen.lock().unwrap();
    assert_eq!(seen.message_raw.as_deref(), Some(&outgoing[..]));
    assert_eq!(seen.original_raw.as_deref(), Some(&original[..]));
    let encoded_message = result
        .info()
        .encoded_message
        .as_ref()
        .expect("encoded message");
    assert_eq!(encoded_message.raw_data, encoded);
    assert!(encoded_message.encoding.is_signed);
    assert!(encoded_message.encoding.is_encrypted);
    assert!(result.info().signing_error.is_none());
}

#[test]
fn missing_raw_data_stays_missing() {
    let seen = Arc::new(Mutex::new(Seen::default()));
    let handler = MessageSecurityHandler::new(RecordingHandler {
        seen: Arc::clone(&seen),
        decoded: Vec::new(),
        encoded: b"x".to_vec(),
    })
    .expect("handler");
    let mut message_info = support::sample_message_info();
    message_info.raw_data = None;
    let mut context_info = support::sample_compose_context_info();
    context_info.original_message = None;
    handler
        .encode_message(
            &Message::new(message_info),
            &ComposeContext::new(context_info),
        )
        .expect("encoding result");
    let seen = seen.lock().unwrap();
    assert!(seen.message_raw.is_none());
    assert!(seen.original_raw.is_none());
    drop(seen);
}
