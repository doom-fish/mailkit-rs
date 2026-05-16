# mailkit-rs coverage audit (vs MacOSX26.2.sdk)

Scope: top-level macOS-available public MailKit symbols (interfaces, protocols, enums, and exported constants) from `MailKit.framework/Headers`. Member-level method/property coverage is already documented in `COVERAGE.md`.

SDK_PUBLIC_SYMBOLS: 32
VERIFIED: 31
GAPS: 1
EXEMPT: 0
COVERAGE_PCT: 96.9%

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| MEAddressAnnotation | class | MEAddressAnnotation.h | `AddressAnnotation` |
| MEComposeUserAction | enum | MEComposeContext.h | `ComposeUserAction` |
| MEComposeContext | class | MEComposeContext.h | `ComposeContext`, `MEComposeContext` |
| MEComposeSession | class | MEComposeSession.h | `ComposeSession`, `MEComposeSession` |
| MEComposeSessionErrorDomain | constant | MEComposeSession.h | `compose_session_error_domain()` |
| MEComposeSessionErrorCode | enum | MEComposeSession.h | `ComposeSessionErrorCode` |
| MEComposeSessionHandler | protocol | MEComposeSession.h | `ComposeSessionHandler`, `ComposeSessionHandlerDelegate` |
| MEContentBlocker | protocol | MEContentBlocker.h | `ContentBlocker`, `ContentBlockerProvider` |
| MEDecodedMessage | class | MEDecodedMessage.h | `DecodedMessage` |
| MEDecodedMessageBanner | class | MEDecodedMessageBanner.h | `DecodedMessageBanner` |
| MEEmailAddress | class | MEEmailAddress.h | `EmailAddress` |
| MEEncodedOutgoingMessage | class | MEEncodedOutgoingMessage.h | `EncodedOutgoingMessage` |
| MEExtension | protocol | MEExtension.h | `MailExtension`, `MailExtensionBuilder`, `MEExtension` |
| MEExtensionManager | class | MEExtensionManager.h | `ExtensionManager` |
| MEExtensionViewController | class | MEExtensionViewController.h | `ExtensionViewController` |
| MEMessageState | enum | MEMessage.h | `MessageState` |
| MEMessageEncryptionState | enum | MEMessage.h | `MessageEncryptionState` |
| MEMessage | class | MEMessage.h | `Message`, `MEMessage` |
| MEMessageActionMessageColor | enum | MEMessageAction.h | `MessageColor` |
| MEMessageActionFlag | enum | MEMessageAction.h | `MessageFlag` |
| MEMessageAction | class | MEMessageAction.h | `MessageAction` |
| MEMessageActionDecision | class | MEMessageActionDecision.h | `MessageActionDecision` |
| MEMessageActionHandler | protocol | MEMessageActionHandler.h | `MessageActionExtension`, `MessageActionHandler` |
| MEMessageDecoder | protocol | MEMessageDecoder.h | `MessageSecurityHandlerDelegate::decoded_message_for_message_data` |
| MEMessageEncoder | protocol | MEMessageEncoder.h | `MessageSecurityHandlerDelegate::encoding_status_for_message`, `MessageSecurityHandlerDelegate::encode_message` |
| MEMessageEncodingResult | class | MEMessageEncodingResult.h | `MessageEncodingResult` |
| MEMessageSecurityErrorDomain | constant | MEMessageSecurityHandler.h | `message_security_error_domain()` |
| MEMessageSecurityHandler | protocol | MEMessageSecurityHandler.h | `MessageSecurityHandler`, `MessageSecurityHandlerDelegate` |
| MEMessageSecurityInformation | class | MEMessageSecurityInformation.h | `MessageSecurityInformation` |
| MEMessageSigner | class | MEMessageSigner.h | `MessageSigner` |
| MEOutgoingMessageEncodingStatus | class | MEOutgoingMessageEncodingStatus.h | `OutgoingMessageEncodingStatus` |

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| MEMessageSecurityErrorCode | enum | MEMessageSecurityHandler.h | The crate exposes `MEMessageSecurityErrorDomain` and raw `NSErrorInfo.code`, but it does not provide a dedicated Rust enum for `MEMessageSecurityEncodingError` / `MEMessageSecurityDecodingError`. |

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |

No exempt symbols: MailKit.framework's macOS headers do not expose deprecated or macOS-unavailable public symbols that need to be filtered from this audit.
