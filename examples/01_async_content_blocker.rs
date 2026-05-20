#[cfg(feature = "async")]
use mailkit::prelude::*;

#[cfg(feature = "async")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MailKit Async API Example");
    println!();

    pollster::block_on(async {
        println!("This example demonstrates the async API for MailKit.");
        println!();
        println!("Note: MailKit is primarily designed for Mail extensions.");
        println!("Most async operations are only available in extension contexts.");
        println!();

        println!("Available async APIs:");
        println!("  - AsyncContentBlocker::content_rules_json()");
        println!("  - AsyncMessageActionDecision::decide()");
        println!("  - AsyncComposeSessionHandler::annotate_addresses_for_session()");
        println!("  - AsyncComposeSessionHandler::allow_message_send_for_session()");
        println!("  - AsyncMessageSecurityHandler::encoding_status_for_message()");
        println!("  - AsyncMessageSecurityHandler::encode_message()");
        println!("  - AsyncMessageSecurityHandler::primary_action_clicked_for_message_context()");
        println!("  - AsyncExtensionManager::reload_visible_messages()");
        println!("  - AsyncExtensionManager::reload_content_blocker()");
        println!();

        println!("Example: Creating a content blocker provider");
        let provider = SimpleContentBlockerProvider;
        let _blocker = ContentBlocker::new(provider)?;
        println!("Created content blocker successfully");

        println!();
        println!("For extension-specific async operations, use:");
        println!("  - async_api::AsyncContentBlocker");
        println!("  - async_api::AsyncMessageActionDecision");
        println!("  - async_api::AsyncComposeSessionHandler");
        println!("  - async_api::AsyncMessageSecurityHandler");
        println!("  - async_api::AsyncExtensionManager");

        Ok(())
    })
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature to be enabled.");
    println!("Run with: cargo run --example 01_async_content_blocker --features async");
}

#[cfg(feature = "async")]
struct SimpleContentBlockerProvider;

#[cfg(feature = "async")]
impl ContentBlockerProvider for SimpleContentBlockerProvider {
    fn content_rules_json(&self) -> Result<Vec<u8>, mailkit::MailKitError> {
        let rules: Vec<u8> = vec![];
        serde_json::to_vec(&rules).map_err(|e| mailkit::MailKitError::FrameworkError(e.to_string()))
    }
}
