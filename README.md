# mailkit-rs

Safe Rust bindings for Apple’s [MailKit](https://developer.apple.com/documentation/mailkit) framework on macOS 13+.

> **Status:** `v0.4.0` makes the message-security bridge fail closed, passes raw MIME data across the bridge as bytes, implements the async message-action and content-blocker futures, and lets `RustMailExtension` serve as a Mail extension's principal class. The crate covers the public MailKit headers audited against the macOS 26.5 SDK (the 27.0 SDK adds nothing to MailKit) and adds headless test harnesses for the Mail-hosted protocols. The crate also ships helper layers for parsing `Authentication-Results` headers and RFC822/EML attachments from `MEMessage.rawData`, because MailKit doesn’t expose public `MEAuthenticationResult` or `EMLAttachment` types.

```toml
[dependencies]
mailkit-rs = "0.4"
```

The library is imported as `mailkit`.

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

Enable the optional `async` feature for executor-agnostic futures. Except for `AsyncExtensionManager`, they drive a Rust handler through the same bridge code Mail uses and resolve as soon as the handler returns, so they also work in tests and ordinary processes:

- `AsyncMessageActionDecision::decide(&extension, &message)` resolves to the handler's `Option<MessageActionDecision>`
- `AsyncContentBlocker::content_rules_json(&blocker)` resolves to the provider's rules JSON
- `AsyncComposeSessionHandler::{annotate_addresses_for_session, allow_message_send_for_session}`
- `AsyncMessageSecurityHandler::{encoding_status_for_message, encode_message, primary_action_clicked_for_message_context}`, with the same fail-closed results as the synchronous harness
- `AsyncExtensionManager::{reload_visible_messages, reload_content_blocker}`, which call `MEExtensionManager` and only take effect inside a Mail extension

## Message security fails closed

`MessageSecurityHandler` never lets a failure look like "nothing to do" to Mail:

- **Encoding.** Mail sends a message unchanged when the encoding result has no encoded message and no errors. An `Err` or panic from `encode_message`, or a result that leaves a requested signature or encryption undone without reporting an error, is replaced by a result with no encoded message and an `EncodingError` in `MEMessageSecurityErrorDomain` as its `signing_error` and/or `encryption_error`. Mail then shows the error and doesn't send the message. An empty result is only passed through when the compose context asks for neither signing nor encryption.
- **Encoding status.** An `Err` or panic from `encoding_status_for_message` becomes a status that can neither sign nor encrypt, with an `EncodingError` as its `security_error`.
- **Decoding.** Return `Ok(None)` when the extension isn't needed for a message. An `Err` or panic from `decoded_message_for_message_data` becomes a decoded message without raw data whose security information carries a `DecodingError`, so Mail shows the failure instead of the undecoded message.
- The `MessageSecurityHandler` harness methods and their async counterparts return exactly what Mail receives, so these failure results arrive as `Ok` values rather than `Err`.

Raw MIME data (`MEMessage.rawData`, the bytes Mail asks the decoder to decode, and decoded and encoded message bodies) crosses the bridge as raw bytes, not as JSON.

## Hosting the handlers in a Mail extension

Mail loads extensions as sandboxed, signed app extensions (`.appex`) embedded in a host app. The extension's `Info.plist` names an `NSExtensionPrincipalClass` that adopts `MEExtension`; Mail creates it with `init` and asks it for handlers. The Swift bridge provides that class under the Objective-C name `RustMailExtension`:

1. Create a macOS app with a Mail Extension target in Xcode, and declare the capabilities you implement under `NSExtensionAttributes` as the template does.
2. Build your handlers into a Rust `staticlib` that depends on `mailkit-rs`, link it into the extension target, and set `NSExtensionPrincipalClass` to `RustMailExtension`.
3. Export an installer from the static library. When Mail creates `RustMailExtension` and nothing is registered yet, it looks up `mailkit_install_extension` with `dlsym`, calls it, and serves the registered handlers:

   ```rust,ignore
   #[no_mangle]
   pub extern "C" fn mailkit_install_extension() {
       let extension = mailkit::MailExtension::builder()
           .with_message_security_handler(
               mailkit::MessageSecurityHandler::new(MySecurityHandler).expect("handler"),
           )
           .build()
           .expect("extension");
       extension.register_as_principal().expect("register");
   }
   ```

4. Keep the symbol in the linked extension: add `-u _mailkit_install_extension` to the target's Other Linker Flags. If rustc links the extension executable itself, reference the function from a `#[used]` static instead, for example `#[used] static KEEP: extern "C" fn() = mailkit_install_extension;`.

You can also call `MailExtension::register_as_principal()` yourself before Mail first connects. Each registration replaces the previous one for principal instances created afterwards. Handlers run synchronously on the thread Mail calls them on, and `ExtensionManager` / `AsyncExtensionManager` only take effect inside the extension process.

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
