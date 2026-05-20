# mailkit-rs

Safe Rust bindings for Apple’s [MailKit](https://developer.apple.com/documentation/mailkit) framework on macOS 13+.

> **Status:** `v0.3.6` expands the async API (Tier 1) with executor-agnostic futures for compose-session validation, message-security callbacks, and extension-manager reloads. The crate covers the public MailKit headers audited against the macOS 26.5 SDK and adds headless test harnesses for the Mail-hosted protocols. The crate also ships helper layers for parsing `Authentication-Results` headers and RFC822/EML attachments from `MEMessage.rawData`, because MailKit doesn’t expose public `MEAuthenticationResult` or `EMLAttachment` types.

## Highlights

- Value wrappers for `MEEmailAddress`, `MEAddressAnnotation`, `MEMessageAction`, `MEMessageActionDecision`, `MEDecodedMessageBanner`, `MEMessageSigner`, `MEMessageSecurityInformation`, `MEDecodedMessage`, `MEEncodedOutgoingMessage`, `MEOutgoingMessageEncodingStatus`, and `MEMessageEncodingResult`
- Snapshot wrappers for `MEMessage`, `MEComposeContext`, and `MEComposeSession`
- Rust-driven adapters for `MEComposeSessionHandler`, `MEMessageActionHandler`, `MEContentBlocker`, `MEMessageSecurityHandler`, and `MEExtension`
- Helper parsers for `Authentication-Results` headers and MIME attachments embedded in raw RFC822 message data
- `ExtensionManager` helpers for `reloadVisibleMessages` and `reloadContentBlocker(withIdentifier:)`

- **Async API** (Tier 1, opt-in `async` feature) with `AsyncMessageActionDecision`, `AsyncContentBlocker`, `AsyncComposeSessionHandler`, `AsyncMessageSecurityHandler`, and `AsyncExtensionManager`

## Quick start

```rust,no_run
use std::collections::BTreeMap;

use mailkit::{
    AuthenticationResult, Message, MessageEncryptionState, MessageInfo, MessageState,
};

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
        date_sent: Some("2026-05-16T00:00:00Z".into()),
        date_received: Some("2026-05-16T00:00:01Z".into()),
        headers: Some(BTreeMap::from([(
            "Authentication-Results".into(),
            vec![
                "mx.example.com; dkim=pass header.i=@example.com; spf=pass smtp.mailfrom=example.com"
                    .into(),
            ],
        )])),
        raw_data: None,
    });

    let results = message.authentication_results()?;
    assert_eq!(results[0].authserv_id, "mx.example.com");
    assert_eq!(AuthenticationResult::parse_headers(message.info().headers.as_ref())?.len(), 1);
    Ok(())
}
```

## Async API

Enable the optional `async` feature to access executor-agnostic futures for:

- `AsyncMessageActionDecision::decide`
- `AsyncContentBlocker::content_rules_json`
- `AsyncComposeSessionHandler::{annotate_addresses_for_session, allow_message_send_for_session}`
- `AsyncMessageSecurityHandler::{encoding_status_for_message, encode_message, primary_action_clicked_for_message_context}`
- `AsyncExtensionManager::{reload_visible_messages, reload_content_blocker}`

## Headless examples

The crate includes numbered examples for each logical area requested in the audit:

- `01_message_action_extension_decision`
- `02_compose_session_handler_roundtrip`
- `03_content_blocker_rules`
- `04_message_security_handler_roundtrip`
- `05_eml_attachment_parse`
- `06_me_message_snapshot`
- `07_me_address_roundtrip`
- `08_me_authentication_result_parse`
- `09_me_decoded_message_build`
- `10_me_encoded_outgoing_message_build`

Run all examples with:

```bash
for ex in examples/*.rs; do cargo run --example "$(basename "$ex" .rs)"; done
```

Run the async surface example with:

```bash
cargo run --example 01_async_content_blocker --features async
```

## Coverage audit

`COVERAGE.md` contains the header-by-header audit against the macOS SDK `MailKit` headers, including the requested logical-area mapping.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
