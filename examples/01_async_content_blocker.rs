#[cfg(feature = "async")]
use mailkit::async_api::{AsyncContentBlocker, AsyncMessageActionDecision};
#[cfg(feature = "async")]
use mailkit::prelude::*;

#[cfg(feature = "async")]
struct TrackerBlocker;

#[cfg(feature = "async")]
impl ContentBlockerProvider for TrackerBlocker {
    fn content_rules_json(&self) -> Result<Vec<u8>, MailKitError> {
        Ok(
            br#"[{"trigger":{"url-filter":"tracker\\.example"},"action":{"type":"block"}}]"#
                .to_vec(),
        )
    }
}

#[cfg(feature = "async")]
struct ArchiveNewsletters;

#[cfg(feature = "async")]
impl MessageActionHandler for ArchiveNewsletters {
    fn decide_action_for_message(
        &self,
        message: &Message,
    ) -> Result<Option<MessageActionDecision>, MailKitError> {
        if message.info().subject.starts_with("Newsletter") {
            Ok(Some(MessageActionDecision::action(
                &MessageAction::move_to_archive()?,
            )?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(feature = "async")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let blocker = ContentBlocker::new(TrackerBlocker)?;
    let actions = MessageActionExtension::new(ArchiveNewsletters)?;
    let message = Message::new(MessageInfo {
        state: MessageState::Received,
        encryption_state: MessageEncryptionState::NotEncrypted,
        subject: "Newsletter: May".into(),
        from_address: EmailAddressInfo {
            raw_string: "News <news@example.com>".into(),
            address_string: Some("news@example.com".into()),
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

    pollster::block_on(async {
        let rules = AsyncContentBlocker::content_rules_json(&blocker).await?;
        println!("content rules: {}", String::from_utf8_lossy(&rules));

        match AsyncMessageActionDecision::decide(&actions, &message).await? {
            Some(decision) => println!("decision: {:?}", decision.kind()),
            None => println!("no decision"),
        }
        Ok(())
    })
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature to be enabled.");
    println!("Run with: cargo run --example 01_async_content_blocker --features async");
}
