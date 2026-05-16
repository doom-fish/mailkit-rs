# Changelog

All notable changes to this project will be documented in this file.

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
