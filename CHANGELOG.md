# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.2] - 2026-10-26

### Fixed

- Update the test assertions for Clippy 1.99 on hosted runners.

## [0.4.1] - 2026-10-26

### Fixed

- Build the Swift bridge with the default SwiftPM build engine and use its reported product directory.
- Export the C entry points from the Swift static library.

## [0.4.0] - 2026-09-24

### Security

- `MessageSecurityHandler` fails closed. An `Err` or panic from
  `encode_message` used to become `MEMessageEncodingResult(nil, nil, nil)`,
  which Mail reads as "no encoding needed", so a message the user asked to
  sign or encrypt went out unsigned and in plaintext without any error. Such
  failures, and results that leave a requested signature or encryption undone
  without reporting an error, now reach Mail as an `EncodingError` in
  `MEMessageSecurityErrorDomain` in `signing_error` and/or `encryption_error`,
  and Mail refuses to send the message.
- Decode failures are no longer dropped. An `Err` or panic from
  `decoded_message_for_message_data` becomes a decoded message without raw
  data whose security information carries a `DecodingError`, so Mail reports
  the failure instead of showing the undecoded message.
- Encoding-status failures now carry an `EncodingError` as the status's
  `securityError` instead of silently reporting that the message can be
  neither signed nor encrypted.

### Fixed

- Raw MIME data crosses the bridge as bytes instead of a JSON number array
  about four times its size: the data Mail asks the decoder to decode,
  `MEMessage.rawData` (also for a compose context's original message and for
  compose sessions), and decoded and encoded message bodies. Large S/MIME
  messages no longer multiply the extension's memory use.
- Every decoded message, encoding status, encoding result and message-action
  decision handed to Mail leaked its cached Rust snapshot, including the whole
  decoded MIME body. The snapshots are now freed, and a result box is released
  when its snapshot fails to decode.
- `AsyncMessageActionDecision::decide` always failed because it called a stub;
  it now runs the handler. `AsyncContentBlocker::content_rules_json` ignored
  the blocker and returned `{"rules":[]}`, which then failed to decode; it now
  returns the provider's rules.
- The security harness no longer leaks the message box when encoding the
  compose context fails.
- The README, the `async_api` docs and the `01_async_content_blocker` example
  describe and exercise the real async behaviour.

### Changed

- **Breaking:** `AsyncMessageActionDecision::decide(&extension, &message)` takes
  the `MessageActionExtension` whose handler decides, and resolves to
  `Ok(None)` when the handler returns no decision.
- **Breaking:** `MessageSecurityHandler::decoded_message_for_message_data`,
  `encoding_status_for_message` and `encode_message`, and the matching
  `AsyncMessageSecurityHandler` futures, return what Mail receives. Delegate
  failures therefore arrive as `Ok` values carrying the security errors above
  instead of as `Err`.
- **Breaking:** low-level `ffi` signatures changed. The decode callback and
  `mail_message_security_handler_test_decode_message` take a byte pointer and
  length, `mail_compose_session_new` takes a JSON box, and
  `mail_message_action_decision_async` takes the handler.
- **Breaking:** the minimum supported Rust version is now 1.82 (was 1.76). The
  crate already used `unsafe extern` blocks, which need 1.82.
- `doom-fish-utils` requirement is now `>=0.4.1, <0.5`.

### Added

- `RustMailExtension` can be a Mail extension's `NSExtensionPrincipalClass`.
  It has a stable Objective-C name and an `init` that serves the handlers
  registered with the new `MailExtension::register_as_principal`, calling an
  exported `mailkit_install_extension` function first when nothing is
  registered. The README explains how to host the handlers in a real Mail
  extension.

## [0.3.7] - 2026-06-06

### Fixed

- The async message-action stub no longer passes a dangling C string to its
  callback.

## [0.3.6] - 2026-05-20

### Added
- Added executor-agnostic async wrappers for compose-session address annotations and send validation.
- Added executor-agnostic async wrappers for message-security encoding status, message encoding, and primary-action handling.
- Added executor-agnostic async wrappers for `MEExtensionManager` reload requests.

### Notes
- Updated async tests/examples plus coverage docs for the expanded MailKit async surface audited against MacOSX26.5.sdk.
- Phase 32 completeness + async sweep.

## [0.3.5] - 2026-05-20

- Clippy hygiene sweep: cleared all `-D warnings` lints across the crate. No public API change.

## [0.3.4] - 2026-05-20

- Widen `doom-fish-utils` dependency bound to `<0.4` so the 0.3.x SPSC-ring release resolves cleanly. No source changes.

## [0.3.3] - 2026-05-18

- Added rustdoc coverage across the safe public MailKit surface, bringing crate documentation to 76.7%
- Documented public wrappers with MailKit counterpart references while intentionally leaving `src/ffi/` undocumented

## [0.3.2] - 2026-05-18

- Widen doom-fish-utils version bound to `<0.3` so 0.2.x resolves.

## [0.3.1] - 2026-05-17

- Fixed panic safety in async callbacks: `message_action_decision_callback` and `content_blocker_content_rules_callback` now use `catch_unwind` to prevent panics from unwinding across FFI boundaries

## [0.3.0] - 2026-05-17

- Added `async_api` module (Tier 1) with `AsyncMessageActionDecision` and `AsyncContentBlocker` for callback-based Swift FFI async operations
- Added async feature gate for opt-in async API support
- Added `pollster` dev dependency for async examples and tests
- Added `doom-fish-utils` dependency for `AsyncCompletion` primitives
- Added Swift bridge thunks for async message action decisions and content blocker queries
- Added examples and tests for async API usage (note: most APIs are extension-only)

## [0.2.1] - 2026-05-16

- Added `MessageSecurityErrorCode` plus `NSErrorInfo::message_security_error_code()` for `MEMessageSecurityHandler` failures
- Closed the final top-level MailKit header audit gap for `MEMessageSecurityErrorCode`

## [0.2.0] - 2026-05-16

- Added safe snapshot wrappers for `MEMessage`, `MEComposeContext`, and `MEComposeSession`
- Added Rust-driven protocol adapters for `MEExtension`, `MEComposeSessionHandler`, `MEMessageActionHandler`, `MEContentBlocker`, and `MEMessageSecurityHandler`
- Added helper layers for parsing `Authentication-Results` headers and RFC822/EML attachments from MailKit message data
- Split the Swift bridge into logical-area files and added numbered examples and tests for every requested area
- Added `COVERAGE.md` auditing the public MailKit headers against the crate surface

## [0.1.0] - 2026-05-16

- Initial release of `mailkit-rs`
- Added safe Rust wrappers for constructible MailKit value objects and `MEExtensionManager`
- Added a Swift bridge, smoke example, and docs
