mod support;

use mailkit::{
    MessageAction, MessageActionDecision, MessageActionDecisionKind, MessageActionExtension,
    MessageActionHandler, MessageActionKind,
};

struct ArchiveHandler;

impl MessageActionHandler for ArchiveHandler {
    fn decide_action_for_message(
        &self,
        message: &mailkit::Message,
    ) -> Result<Option<MessageActionDecision>, mailkit::MailKitError> {
        assert_eq!(message.info().subject, "Quarterly report");
        Ok(Some(MessageActionDecision::action(
            &MessageAction::move_to_archive()?,
        )?))
    }

    fn required_headers(&self) -> Result<Vec<String>, mailkit::MailKitError> {
        Ok(vec!["Authentication-Results".into()])
    }
}

#[test]
fn message_action_extension_round_trips_decision_and_required_headers() {
    let extension = MessageActionExtension::new(ArchiveHandler).expect("message action extension");
    let decision = extension
        .decide_action_for_message(&support::sample_message())
        .expect("decision")
        .expect("action decision");
    assert_eq!(
        decision.kind(),
        &MessageActionDecisionKind::Action(MessageActionKind::MoveToArchive)
    );
    assert_eq!(
        extension.required_headers().expect("required headers"),
        vec!["Authentication-Results"]
    );
}

struct RawDataHandler {
    seen: std::sync::Arc<std::sync::Mutex<Vec<Option<Vec<u8>>>>>,
}

impl MessageActionHandler for RawDataHandler {
    fn decide_action_for_message(
        &self,
        message: &mailkit::Message,
    ) -> Result<Option<MessageActionDecision>, mailkit::MailKitError> {
        self.seen
            .lock()
            .expect("seen")
            .push(message.info().raw_data.clone());
        Ok(None)
    }
}

#[test]
fn message_action_handler_receives_raw_data_unchanged() {
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let extension = MessageActionExtension::new(RawDataHandler {
        seen: std::sync::Arc::clone(&seen),
    })
    .expect("message action extension");
    let mut info = support::sample_message_info();
    let raw: Vec<u8> = (0..=255u8).cycle().take(200_000).collect();
    info.raw_data = Some(raw.clone());
    assert!(extension
        .decide_action_for_message(&mailkit::Message::new(info))
        .expect("decision")
        .is_none());
    assert_eq!(*seen.lock().expect("seen"), vec![Some(raw)]);
}
