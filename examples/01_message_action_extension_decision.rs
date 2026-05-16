use mailkit::{
    Message, MessageAction, MessageActionDecision, MessageActionExtension, MessageActionHandler,
    MessageActionKind, MessageEncryptionState, MessageInfo, MessageState,
};

struct ArchiveHandler;

impl MessageActionHandler for ArchiveHandler {
    fn decide_action_for_message(
        &self,
        message: &Message,
    ) -> Result<Option<MessageActionDecision>, mailkit::MailKitError> {
        assert_eq!(message.info().subject, "Quarterly report");
        Ok(Some(MessageActionDecision::action(
            &MessageAction::move_to_archive()?,
        )?))
    }
}

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
        date_sent: None,
        date_received: None,
        headers: None,
        raw_data: None,
    });
    let extension = MessageActionExtension::new(ArchiveHandler)?;
    let decision = extension
        .decide_action_for_message(&message)?
        .expect("decision");
    assert_eq!(
        decision.kind(),
        &mailkit::MessageActionDecisionKind::Action(MessageActionKind::MoveToArchive)
    );
    println!("message action extension returned an archive decision");
    Ok(())
}
