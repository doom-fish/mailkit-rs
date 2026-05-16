use mailkit::{
    DecodedMessage, DecodedMessageBanner, EmailAddress, MessageSecurityInformation, MessageSigner,
    RemoteContentBlocking,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let decoded = DecodedMessage::new(Some(b"hello"), &security, Some(&[4, 5]), Some(&banner))?;
    assert!(decoded.info().security_information.is_encrypted);
    println!(
        "built MEDecodedMessage with {:?} bytes",
        decoded.info().raw_data.as_ref().map(Vec::len)
    );
    Ok(())
}
