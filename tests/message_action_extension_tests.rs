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
