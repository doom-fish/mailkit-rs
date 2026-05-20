# mailkit-rs coverage audit v2 (vs MacOSX26.5.sdk)

SDK_PUBLIC_SYMBOLS: 32
VERIFIED: 32
GAPS: 0
EXEMPT: 0
COVERAGE_PCT: 100

Comprehensive audit of MailKit.framework public surface (top-level symbols only) against the mailkit-rs crate wrapper. All 32 public symbols from the SDK are verified as present in the crate's Rust and Swift bridge implementations. The optional `async_api` module also exposes executor-agnostic futures for the relevant completion-handler members on compose-session, message-security, and extension-manager surfaces. No deprecated or macOS-unavailable symbols were filtered; all symbols are API_AVAILABLE(macos(12.0)).

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| MEAddressAnnotation | class | MEAddressAnnotation.h | `AddressAnnotation` |
| MEComposeContext | class | MEComposeContext.h | `ComposeContext` |
| MEComposeSession | class | MEComposeSession.h | `ComposeSession`, `MEComposeSession` |
| MEComposeSessionErrorCode | enum | MEComposeSession.h | `ComposeSessionErrorCode` |
| MEComposeSessionErrorDomain | constant | MEComposeSession.h | `compose_session_error_domain()` |
| MEComposeSessionHandler | protocol | MEComposeSession.h | `ComposeSessionHandler`, `ComposeSessionHandlerDelegate` |
| MEComposeUserAction | enum | MEComposeContext.h | `ComposeUserAction` |
| MEContentBlocker | protocol | MEContentBlocker.h | `ContentBlocker`, `ContentBlockerProvider` |
| MEDecodedMessage | class | MEDecodedMessage.h | `DecodedMessage` |
| MEDecodedMessageBanner | class | MEDecodedMessageBanner.h | `DecodedMessageBanner` |
| MEEmailAddress | class | MEEmailAddress.h | `EmailAddress` |
| MEEncodedOutgoingMessage | class | MEEncodedOutgoingMessage.h | `EncodedOutgoingMessage` |
| MEExtension | protocol | MEExtension.h | `MEExtension`, `MailExtension`, `MailExtensionBuilder` |
| MEExtensionManager | class | MEExtensionManager.h | `ExtensionManager` |
| MEExtensionViewController | class | MEExtensionViewController.h | `ExtensionViewController` |
| MEMessage | class | MEMessage.h | `Message`, `MEMessage` |
| MEMessageAction | class | MEMessageAction.h | `MessageAction` |
| MEMessageActionDecision | class | MEMessageActionDecision.h | `MessageActionDecision` |
| MEMessageActionFlag | enum | MEMessageAction.h | `MessageFlag` |
| MEMessageActionHandler | protocol | MEMessageActionHandler.h | `MessageActionHandler`, `MessageActionExtension` |
| MEMessageActionMessageColor | enum | MEMessageAction.h | `MessageColor` |
| MEMessageDecoder | protocol | MEMessageDecoder.h | `MessageSecurityHandlerDelegate::decoded_message_for_message_data` |
| MEMessageEncoder | protocol | MEMessageEncoder.h | `MessageSecurityHandlerDelegate::encoding_status_for_message`, `MessageSecurityHandlerDelegate::encode_message` |
| MEMessageEncodingResult | class | MEMessageEncodingResult.h | `MessageEncodingResult` |
| MEMessageEncryptionState | enum | MEMessage.h | `MessageEncryptionState` |
| MEMessageSecurityErrorCode | enum | MEMessageSecurityHandler.h | `MessageSecurityErrorCode` |
| MEMessageSecurityErrorDomain | constant | MEMessageSecurityHandler.h | `message_security_error_domain()` |
| MEMessageSecurityHandler | protocol | MEMessageSecurityHandler.h | `MessageSecurityHandler`, `MessageSecurityHandlerDelegate` |
| MEMessageSecurityInformation | class | MEMessageSecurityInformation.h | `MessageSecurityInformation` |
| MEMessageSigner | class | MEMessageSigner.h | `MessageSigner` |
| MEMessageState | enum | MEMessage.h | `MessageState` |
| MEOutgoingMessageEncodingStatus | class | MEOutgoingMessageEncodingStatus.h | `OutgoingMessageEncodingStatus` |

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |

No gaps: all public macOS symbols from MailKit.framework are wrapped in mailkit-rs.

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |

No exempt symbols: all public MailKit symbols are macOS-available (API_AVAILABLE(macos(12.0))+) and not deprecated.
