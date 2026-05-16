use mailkit::{
    EmailAddress, EncodedOutgoingMessage, MessageEncoding, MessageEncodingResult,
    OutgoingMessageCapabilities, OutgoingMessageEncodingStatus,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let encoded = EncodedOutgoingMessage::new(
        b"raw message",
        MessageEncoding {
            is_signed: true,
            is_encrypted: true,
        },
    )?;
    let email = EmailAddress::new("Alice Example <alice@example.com>")?;
    let status = OutgoingMessageEncodingStatus::new(
        OutgoingMessageCapabilities {
            can_sign: true,
            can_encrypt: true,
        },
        None,
        &[&email],
    )?;
    let result = MessageEncodingResult::new(Some(&encoded), None, None)?;
    assert!(status.info().capabilities.can_encrypt);
    assert!(result.info().encoded_message.is_some());
    println!("built MEEncodedOutgoingMessage and its encoding result");
    Ok(())
}
