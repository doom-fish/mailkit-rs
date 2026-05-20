# Changelog

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

All notable changes to this project will be documented in this file.

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
