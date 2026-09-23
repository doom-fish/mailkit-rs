# MailKit coverage audit

Audited against `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk/System/Library/Frameworks/MailKit.framework/Headers`. The macOS 27.0 SDK's MailKit headers are identical.

## Requested logical-area mapping

| Requested area | Public MailKit surface | Status | Notes |
| --- | --- | --- | --- |
| `MessageActionExtension` | `MEMessageActionHandler` + `MEExtension` | ✅ implemented | `MessageActionExtension` wraps the handler protocol; `MailExtension` bridges the principal extension protocol, and `RustMailExtension` can be named as a real extension's `NSExtensionPrincipalClass` (see the README). |
| `ComposeSessionHandler` | `MEComposeSessionHandler` + `MEComposeSession` + `MEComposeContext` | ✅ implemented | Includes synthetic session/context snapshots for headless tests and examples. |
| `ContentBlocker` | `MEContentBlocker` | ✅ implemented | Also keeps `MEExtensionManager.reloadContentBlocker(withIdentifier:)`. |
| `MessageSecurityHandler` | `MEMessageSecurityHandler` + `MEMessageEncoder` + `MEMessageDecoder` | ✅ implemented | Fails closed: delegate errors and panics reach Mail as `MEMessageSecurityErrorDomain` errors. Includes encode/decode test harnesses and view-controller hooks. |
| `EMLAttachment` | _no public MailKit type_ | ✅ helper | Implemented as `EmlAttachment` over `MEMessage.rawData`. MailKit only exposes raw RFC822 data. |
| `MEMessage` | `MEMessage` | ✅ implemented | Read-only snapshot wrapper with helpers for auth results and attachments. |
| `MEAddress` | `MEEmailAddress` + `MEAddressAnnotation` | ✅ mapped | MailKit uses `MEEmailAddress` / `MEAddressAnnotation` instead of a standalone `MEAddress` symbol. |
| `MEAuthenticationResult` | _no public MailKit type_ | ✅ helper | Implemented as `AuthenticationResult` over `Authentication-Results` headers. |
| `MEDecodedMessage` | `MEDecodedMessage` + `MEDecodedMessageBanner` | ✅ implemented | Includes safe constructors and round-trip box support for handler callbacks. |
| `MEEncodedOutgoingMessage` | `MEEncodedOutgoingMessage` + related encoding result/status types | ✅ implemented | Includes encode-status/result round-trip support. |

## Async surface

With the optional `async` feature enabled, `mailkit::async_api` exposes executor-agnostic futures that drive the Rust handlers through the bridge (`AsyncExtensionManager` calls `MEExtensionManager` and only takes effect inside a Mail extension):

- `AsyncMessageActionDecision::decide`
- `AsyncContentBlocker::content_rules_json`
- `AsyncComposeSessionHandler::{annotate_addresses_for_session, allow_message_send_for_session}`
- `AsyncMessageSecurityHandler::{encoding_status_for_message, encode_message, primary_action_clicked_for_message_context}`
- `AsyncExtensionManager::{reload_visible_messages, reload_content_blocker}`

## Header coverage

| Header | API | Status | Notes |
| --- | --- | --- | --- |
| `MEAddressAnnotation.h` | `MEAddressAnnotation.errorWithLocalizedDescription(_:)` | ✅ implemented | `AddressAnnotation::error`. |
| `MEAddressAnnotation.h` | `MEAddressAnnotation.warningWithLocalizedDescription(_:)` | ✅ implemented | `AddressAnnotation::warning`. |
| `MEAddressAnnotation.h` | `MEAddressAnnotation.successWithLocalizedDescription(_:)` | ✅ implemented | `AddressAnnotation::success`. |
| `MEComposeContext.h` | `MEComposeUserAction` | ✅ implemented | `ComposeUserAction`. |
| `MEComposeContext.h` | `MEComposeContext.contextID` | ✅ implemented | `ComposeContextInfo.context_id`. |
| `MEComposeContext.h` | `MEComposeContext.originalMessage` | ✅ implemented | `ComposeContextInfo.original_message`. |
| `MEComposeContext.h` | `MEComposeContext.action` | ✅ implemented | `ComposeContextInfo.action`. |
| `MEComposeContext.h` | `MEComposeContext.isEncrypted` | ✅ implemented | `ComposeContextInfo.is_encrypted`. |
| `MEComposeContext.h` | `MEComposeContext.shouldEncrypt` | ✅ implemented | `ComposeContextInfo.should_encrypt`. |
| `MEComposeContext.h` | `MEComposeContext.isSigned` | ✅ implemented | `ComposeContextInfo.is_signed`. |
| `MEComposeContext.h` | `MEComposeContext.shouldSign` | ✅ implemented | `ComposeContextInfo.should_sign`. |
| `MEComposeSession.h` | `MEComposeSession.sessionID` | ✅ implemented | `ComposeSessionInfo.session_id`. |
| `MEComposeSession.h` | `MEComposeSession.mailMessage` | ✅ implemented | `ComposeSessionInfo.mail_message`. |
| `MEComposeSession.h` | `MEComposeSession.composeContext` | ✅ implemented | `ComposeSessionInfo.compose_context`. |
| `MEComposeSession.h` | `MEComposeSession.reloadSession()` | ✅ implemented | `ComposeSession::reload_session`. |
| `MEComposeSession.h` | `MEComposeSessionErrorDomain` | ✅ implemented | `compose_session_error_domain()`. |
| `MEComposeSession.h` | `MEComposeSessionErrorCode` | ✅ implemented | `ComposeSessionErrorCode`. |
| `MEComposeSession.h` | `MEComposeSessionHandler.mailComposeSessionDidBegin(_:)` | ✅ implemented | `ComposeSessionHandlerDelegate::mail_compose_session_did_begin`. |
| `MEComposeSession.h` | `MEComposeSessionHandler.mailComposeSessionDidEnd(_:)` | ✅ implemented | `ComposeSessionHandlerDelegate::mail_compose_session_did_end`. |
| `MEComposeSession.h` | `MEComposeSessionHandler.viewControllerForSession(_:)` | ✅ implemented | `ComposeSessionHandlerDelegate::view_controller_for_session`. |
| `MEComposeSession.h` | `MEComposeSessionHandler.session(_:annotateAddressesWithCompletionHandler:)` | ✅ implemented | `ComposeSessionHandlerDelegate::annotate_addresses_for_session`; `async_api::AsyncComposeSessionHandler::annotate_addresses_for_session`. |
| `MEComposeSession.h` | `MEComposeSessionHandler.session(_:canSendMessageWithCompletionHandler:)` | ✅ implemented | `ComposeSessionHandlerDelegate::allow_message_send_for_session`; `async_api::AsyncComposeSessionHandler::allow_message_send_for_session`. |
| `MEComposeSession.h` | `MEComposeSessionHandler.additionalHeadersForSession(_:)` | ✅ implemented | `ComposeSessionHandlerDelegate::additional_headers_for_session`. |
| `MEContentBlocker.h` | `MEContentBlocker.contentRulesJSON()` | ✅ implemented | `ContentBlockerProvider::content_rules_json`; `async_api::AsyncContentBlocker::content_rules_json`. |
| `MEDecodedMessage.h` | `MEDecodedMessage.rawData` | ✅ implemented | `DecodedMessageInfo.raw_data`. |
| `MEDecodedMessage.h` | `MEDecodedMessage.securityInformation` | ✅ implemented | `DecodedMessageInfo.security_information`. |
| `MEDecodedMessage.h` | `MEDecodedMessage.context` | ✅ implemented | `DecodedMessageInfo.context`. |
| `MEDecodedMessage.h` | `MEDecodedMessage.banner` | ✅ implemented | `DecodedMessageInfo.banner`. |
| `MEDecodedMessage.h` | `MEDecodedMessage.init(data:securityInformation:context:)` | ✅ implemented | `DecodedMessage::new(..., banner: None)`. |
| `MEDecodedMessage.h` | `MEDecodedMessage.init(data:securityInformation:context:banner:)` | ✅ implemented | `DecodedMessage::new`. |
| `MEDecodedMessageBanner.h` | `MEDecodedMessageBanner.title` | ✅ implemented | `DecodedMessageBannerInfo.title`. |
| `MEDecodedMessageBanner.h` | `MEDecodedMessageBanner.primaryActionTitle` | ✅ implemented | `DecodedMessageBannerInfo.primary_action_title`. |
| `MEDecodedMessageBanner.h` | `MEDecodedMessageBanner.dismissable` | ✅ implemented | `DecodedMessageBannerInfo.dismissable`. |
| `MEDecodedMessageBanner.h` | `MEDecodedMessageBanner.init(title:primaryActionTitle:dismissable:)` | ✅ implemented | `DecodedMessageBanner::new`. |
| `MEEmailAddress.h` | `MEEmailAddress.rawString` | ✅ implemented | `EmailAddress::raw_string`. |
| `MEEmailAddress.h` | `MEEmailAddress.addressString` | ✅ implemented | `EmailAddress::address_string`. |
| `MEEmailAddress.h` | `MEEmailAddress.init(rawString:)` | ✅ implemented | `EmailAddress::new`. |
| `MEEncodedOutgoingMessage.h` | `MEEncodedOutgoingMessage.rawData` | ✅ implemented | `EncodedOutgoingMessageInfo.raw_data`. |
| `MEEncodedOutgoingMessage.h` | `MEEncodedOutgoingMessage.isSigned` | ✅ implemented | `EncodedOutgoingMessageInfo.encoding.is_signed`. |
| `MEEncodedOutgoingMessage.h` | `MEEncodedOutgoingMessage.isEncrypted` | ✅ implemented | `EncodedOutgoingMessageInfo.encoding.is_encrypted`. |
| `MEEncodedOutgoingMessage.h` | `MEEncodedOutgoingMessage.init(rawData:isSigned:isEncrypted:)` | ✅ implemented | `EncodedOutgoingMessage::new`. |
| `MEExtension.h` | `MEExtension.handlerForComposeSession(_:)` | ✅ implemented | `MailExtension` Swift bridge advertises the selector when a compose handler is configured; `RustMailExtension` serves the handlers registered with `MailExtension::register_as_principal`. |
| `MEExtension.h` | `MEExtension.handlerForMessageActions()` | ✅ implemented | `MailExtension` Swift bridge advertises the selector when a message-action handler is configured; `RustMailExtension` serves the handlers registered with `MailExtension::register_as_principal`. |
| `MEExtension.h` | `MEExtension.handlerForContentBlocker()` | ✅ implemented | `MailExtension` Swift bridge advertises the selector when a content blocker is configured; `RustMailExtension` serves the handlers registered with `MailExtension::register_as_principal`. |
| `MEExtension.h` | `MEExtension.handlerForMessageSecurity()` | ✅ implemented | `MailExtension` Swift bridge advertises the selector when a message-security handler is configured; `RustMailExtension` serves the handlers registered with `MailExtension::register_as_principal`. |
| `MEExtensionManager.h` | `MEExtensionManager.reloadContentBlocker(withIdentifier:completionHandler:)` | ✅ implemented | `ExtensionManager::reload_content_blocker`; `async_api::AsyncExtensionManager::reload_content_blocker`. |
| `MEExtensionManager.h` | `MEExtensionManager.reloadVisibleMessages(completionHandler:)` | ✅ implemented | `ExtensionManager::reload_visible_messages`; `async_api::AsyncExtensionManager::reload_visible_messages`. |
| `MEExtensionViewController.h` | `MEExtensionViewController` | ✅ implemented | `ExtensionViewController::new` and handler round-trips. |
| `MEMessage.h` | `MEMessageState` | ✅ implemented | `MessageState`. |
| `MEMessage.h` | `MEMessageEncryptionState` | ✅ implemented | `MessageEncryptionState`. |
| `MEMessage.h` | `MEMessage.state` | ✅ implemented | `MessageInfo.state`. |
| `MEMessage.h` | `MEMessage.encryptionState` | ✅ implemented | `MessageInfo.encryption_state`. |
| `MEMessage.h` | `MEMessage.subject` | ✅ implemented | `MessageInfo.subject`. |
| `MEMessage.h` | `MEMessage.fromAddress` | ✅ implemented | `MessageInfo.from_address`. |
| `MEMessage.h` | `MEMessage.toAddresses` | ✅ implemented | `MessageInfo.to_addresses`. |
| `MEMessage.h` | `MEMessage.ccAddresses` | ✅ implemented | `MessageInfo.cc_addresses`. |
| `MEMessage.h` | `MEMessage.bccAddresses` | ✅ implemented | `MessageInfo.bcc_addresses`. |
| `MEMessage.h` | `MEMessage.replyToAddresses` | ✅ implemented | `MessageInfo.reply_to_addresses`. |
| `MEMessage.h` | `MEMessage.allRecipientAddresses` | ✅ implemented | `MessageInfo.all_recipient_addresses`. |
| `MEMessage.h` | `MEMessage.dateSent` | ✅ implemented | Exposed as an ISO-8601 string via snapshot/KVC extraction. |
| `MEMessage.h` | `MEMessage.dateReceived` | ✅ implemented | Exposed as an ISO-8601 string via snapshot/KVC extraction. |
| `MEMessage.h` | `MEMessage.headers` | ✅ implemented | `MessageInfo.headers`. |
| `MEMessage.h` | `MEMessage.rawData` | ✅ implemented | `MessageInfo.raw_data`; also powers `EmlAttachment`. |
| `MEMessageAction.h` | `MEMessageActionMessageColor` | ✅ implemented | `MessageColor`. |
| `MEMessageAction.h` | `MEMessageActionFlag` | ✅ implemented | `MessageFlag`. |
| `MEMessageAction.h` | `MEMessageAction.moveToTrashAction` | ✅ implemented | `MessageAction::move_to_trash`. |
| `MEMessageAction.h` | `MEMessageAction.moveToArchiveAction` | ✅ implemented | `MessageAction::move_to_archive`. |
| `MEMessageAction.h` | `MEMessageAction.moveToJunkAction` | ✅ implemented | `MessageAction::move_to_junk`. |
| `MEMessageAction.h` | `MEMessageAction.markAsReadAction` | ✅ implemented | `MessageAction::mark_as_read`. |
| `MEMessageAction.h` | `MEMessageAction.markAsUnreadAction` | ✅ implemented | `MessageAction::mark_as_unread`. |
| `MEMessageAction.h` | `MEMessageAction.flag(_:)` | ✅ implemented | `MessageAction::flag`. |
| `MEMessageAction.h` | `MEMessageAction.setBackgroundColor(_:)` | ✅ implemented | `MessageAction::set_background_color`. |
| `MEMessageActionDecision.h` | `MEMessageActionDecision.invokeAgainWithBody` | ✅ implemented | `MessageActionDecision::invoke_again_with_body`. |
| `MEMessageActionDecision.h` | `MEMessageActionDecision.action(_:)` | ✅ implemented | `MessageActionDecision::action`. |
| `MEMessageActionDecision.h` | `MEMessageActionDecision.actions(_:)` | ✅ implemented | `MessageActionDecision::actions`. |
| `MEMessageActionHandler.h` | `MEMessageActionHandler.decideAction(for:completionHandler:)` | ✅ implemented | `MessageActionHandler::decide_action_for_message`; `async_api::AsyncMessageActionDecision::decide`. |
| `MEMessageActionHandler.h` | `MEMessageActionHandler.requiredHeaders` | ✅ implemented | `MessageActionHandler::required_headers`. |
| `MEMessageDecoder.h` | `MEMessageDecoder.decodedMessage(forMessageData:)` | ✅ implemented | `MessageSecurityHandlerDelegate::decoded_message_for_message_data`; the data arrives as raw bytes, and errors become a `DecodingError` Mail displays. |
| `MEMessageEncoder.h` | `MEMessageEncoder.getEncodingStatus(for:composeContext:completionHandler:)` | ✅ implemented | `MessageSecurityHandlerDelegate::encoding_status_for_message`; `async_api::AsyncMessageSecurityHandler::encoding_status_for_message`. Errors become the status's `securityError`. |
| `MEMessageEncoder.h` | `MEMessageEncoder.encode(_:composeContext:completionHandler:)` | ✅ implemented | `MessageSecurityHandlerDelegate::encode_message`; `async_api::AsyncMessageSecurityHandler::encode_message`. Errors, panics and unmet sign/encrypt requests become signing/encryption errors, never an empty "no encoding needed" result. |
| `MEMessageEncodingResult.h` | `MEMessageEncodingResult.encodedMessage` | ✅ implemented | `MessageEncodingResultInfo.encoded_message`. |
| `MEMessageEncodingResult.h` | `MEMessageEncodingResult.signingError` | ✅ implemented | `MessageEncodingResultInfo.signing_error`. |
| `MEMessageEncodingResult.h` | `MEMessageEncodingResult.encryptionError` | ✅ implemented | `MessageEncodingResultInfo.encryption_error`. |
| `MEMessageEncodingResult.h` | `MEMessageEncodingResult.init(encodedMessage:signingError:encryptionError:)` | ✅ implemented | `MessageEncodingResult::new`. |
| `MEMessageSecurityHandler.h` | `MEMessageSecurityErrorDomain` | ✅ implemented | `message_security_error_domain()`. |
| `MEMessageSecurityHandler.h` | `MEMessageSecurityErrorCode` | ✅ implemented | `MessageSecurityErrorCode` plus `NSErrorInfo::message_security_error_code()`. |
| `MEMessageSecurityHandler.h` | `MEMessageSecurityHandler.extensionViewController(signers:)` | ✅ implemented | `MessageSecurityHandlerDelegate::extension_view_controller_for_signers`. |
| `MEMessageSecurityHandler.h` | `MEMessageSecurityHandler.extensionViewController(messageContext:)` | ✅ implemented | `MessageSecurityHandlerDelegate::extension_view_controller_for_message_context`. |
| `MEMessageSecurityHandler.h` | `MEMessageSecurityHandler.primaryActionClicked(forMessageContext:completionHandler:)` | ✅ implemented | `MessageSecurityHandlerDelegate::primary_action_clicked_for_message_context`; `async_api::AsyncMessageSecurityHandler::primary_action_clicked_for_message_context`. |
| `MEMessageSecurityInformation.h` | `MEMessageSecurityInformation.signers` | ✅ implemented | `MessageSecurityInformationInfo.signers`. |
| `MEMessageSecurityInformation.h` | `MEMessageSecurityInformation.isEncrypted` | ✅ implemented | `MessageSecurityInformationInfo.is_encrypted`. |
| `MEMessageSecurityInformation.h` | `MEMessageSecurityInformation.signingError` | ✅ implemented | `MessageSecurityInformationInfo.signing_error`. |
| `MEMessageSecurityInformation.h` | `MEMessageSecurityInformation.encryptionError` | ✅ implemented | `MessageSecurityInformationInfo.encryption_error`. |
| `MEMessageSecurityInformation.h` | `MEMessageSecurityInformation.shouldBlockRemoteContent` | ✅ implemented | `MessageSecurityInformationInfo.remote_content.should_block`. |
| `MEMessageSecurityInformation.h` | `MEMessageSecurityInformation.localizedRemoteContentBlockingReason` | ✅ implemented | `MessageSecurityInformationInfo.remote_content.localized_reason`. |
| `MEMessageSecurityInformation.h` | `MEMessageSecurityInformation.init(signers:isEncrypted:signingError:encryptionError:)` | ✅ implemented | `MessageSecurityInformation::new(..., remote_content.should_block = false)`. |
| `MEMessageSecurityInformation.h` | `MEMessageSecurityInformation.init(signers:isEncrypted:signingError:encryptionError:shouldBlockRemoteContent:localizedRemoteContentBlockingReason:)` | ✅ implemented | `MessageSecurityInformation::new`. |
| `MEMessageSigner.h` | `MEMessageSigner.emailAddresses` | ✅ implemented | `MessageSignerInfo.email_addresses`. |
| `MEMessageSigner.h` | `MEMessageSigner.label` | ✅ implemented | `MessageSignerInfo.label`. |
| `MEMessageSigner.h` | `MEMessageSigner.context` | ✅ implemented | `MessageSignerInfo.context`. |
| `MEMessageSigner.h` | `MEMessageSigner.init(emailAddresses:signatureLabel:context:)` | ✅ implemented | `MessageSigner::new`. |
| `MEOutgoingMessageEncodingStatus.h` | `MEOutgoingMessageEncodingStatus.canSign` | ✅ implemented | `OutgoingMessageEncodingStatusInfo.capabilities.can_sign`. |
| `MEOutgoingMessageEncodingStatus.h` | `MEOutgoingMessageEncodingStatus.canEncrypt` | ✅ implemented | `OutgoingMessageEncodingStatusInfo.capabilities.can_encrypt`. |
| `MEOutgoingMessageEncodingStatus.h` | `MEOutgoingMessageEncodingStatus.securityError` | ✅ implemented | `OutgoingMessageEncodingStatusInfo.security_error`. |
| `MEOutgoingMessageEncodingStatus.h` | `MEOutgoingMessageEncodingStatus.addressesFailingEncryption` | ✅ implemented | `OutgoingMessageEncodingStatusInfo.addresses_failing_encryption`. |
| `MEOutgoingMessageEncodingStatus.h` | `MEOutgoingMessageEncodingStatus.init(canSign:canEncrypt:securityError:addressesFailingEncryption:)` | ✅ implemented | `OutgoingMessageEncodingStatus::new`. |

## Deferred / skipped APIs

No public MailKit header APIs were skipped in this pass.
