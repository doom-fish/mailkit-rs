# Changelog

All notable changes to this project will be documented in this file.

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
