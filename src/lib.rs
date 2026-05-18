#![doc = include_str!("../README.md")]
//!
//! ---
//!
//! # API documentation
//!
//! Safe Rust bindings for Apple's
//! [MailKit](https://developer.apple.com/documentation/mailkit)
//! framework.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::doc_markdown,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::new_without_default
)]

/// MailKit message actions and action decisions.
pub mod action;
/// MailKit email-address wrappers and address annotations.
pub mod address;
/// Parsers and snapshots for MailKit authentication results.
pub mod authentication_result;
/// MailKit compose-session contexts, sessions, and handler callbacks.
pub mod compose_session_handler;
/// MailKit content blocker providers and wrappers.
pub mod content_blocker;
/// Re-exports decoded-message security wrappers.
pub mod decoded_message;
/// Helpers for parsing RFC 822 attachments from MailKit message data.
pub mod eml_attachment;
/// Re-exports encoded outgoing-message security wrappers.
pub mod encoded_outgoing_message;
/// Error types returned by the safe MailKit wrappers.
pub mod error;
/// MailKit extension registration and builder types.
pub mod extension;
/// MailKit extension view controller wrapper.
pub mod extension_view_controller;
/// Low-level FFI shims used by the safe MailKit wrappers.
pub mod ffi;
/// MailKit extension-manager utilities.
pub mod manager;
/// MailKit message snapshots and helpers.
pub mod message;
/// MailKit message-action handler wrappers.
pub mod message_action_extension;
/// MailKit message-security handler wrappers.
pub mod message_security_handler;
mod private;
/// MailKit decoded and encoded message-security wrappers.
pub mod security;

#[cfg(feature = "async")]
/// Executor-agnostic async wrappers for MailKit callbacks.
pub mod async_api;

pub use action::{
    MessageAction, MessageActionDecision, MessageActionDecisionKind, MessageActionKind,
    MessageColor, MessageFlag,
};
pub use address::{
    AddressAnnotation, AddressAnnotationInfo, AddressAnnotationKind, EmailAddress, EmailAddressInfo,
};
pub use authentication_result::{
    AuthenticationMethodResult, AuthenticationResult, MEAuthenticationResult,
};
pub use compose_session_handler::{
    compose_session_error_domain, AddressAnnotationMatch, ComposeContext, ComposeContextInfo,
    ComposeSession, ComposeSessionErrorCode, ComposeSessionHandler, ComposeSessionHandlerDelegate,
    ComposeSessionInfo, ComposeUserAction, MEComposeContext, MEComposeSession,
};
pub use content_blocker::{ContentBlocker, ContentBlockerProvider};
pub use decoded_message::{
    DecodedMessage, DecodedMessageBanner, DecodedMessageBannerInfo, DecodedMessageInfo,
    MessageSecurityInformation, MessageSecurityInformationInfo, MessageSigner, MessageSignerInfo,
    RemoteContentBlocking,
};
pub use eml_attachment::{EMLAttachment, EmlAttachment};
pub use encoded_outgoing_message::{
    EncodedOutgoingMessage, EncodedOutgoingMessageInfo, MessageEncoding, MessageEncodingResult,
    MessageEncodingResultInfo, OutgoingMessageCapabilities, OutgoingMessageEncodingStatus,
    OutgoingMessageEncodingStatusInfo,
};
pub use error::{MailKitError, NSErrorInfo};
pub use extension::{MEExtension, MailExtension, MailExtensionBuilder};
pub use extension_view_controller::ExtensionViewController;
pub use manager::ExtensionManager;
pub use message::{MEMessage, Message, MessageEncryptionState, MessageInfo, MessageState};
pub use message_action_extension::{MessageActionExtension, MessageActionHandler};
pub use message_security_handler::{
    message_security_error_domain, MessageSecurityErrorCode, MessageSecurityHandler,
    MessageSecurityHandlerDelegate,
};

/// Common imports.
pub mod prelude {
    pub use crate::action::{
        MessageAction, MessageActionDecision, MessageActionDecisionKind, MessageActionKind,
        MessageColor, MessageFlag,
    };
    pub use crate::address::{
        AddressAnnotation, AddressAnnotationInfo, AddressAnnotationKind, EmailAddress,
        EmailAddressInfo,
    };
    pub use crate::authentication_result::{
        AuthenticationMethodResult, AuthenticationResult, MEAuthenticationResult,
    };
    pub use crate::compose_session_handler::{
        AddressAnnotationMatch, ComposeContext, ComposeContextInfo, ComposeSession,
        ComposeSessionErrorCode, ComposeSessionHandler, ComposeSessionHandlerDelegate,
        ComposeSessionInfo, ComposeUserAction,
    };
    pub use crate::content_blocker::{ContentBlocker, ContentBlockerProvider};
    pub use crate::decoded_message::{
        DecodedMessage, DecodedMessageBanner, DecodedMessageBannerInfo, DecodedMessageInfo,
        MessageSecurityInformation, MessageSecurityInformationInfo, MessageSigner,
        MessageSignerInfo, RemoteContentBlocking,
    };
    pub use crate::eml_attachment::{EMLAttachment, EmlAttachment};
    pub use crate::encoded_outgoing_message::{
        EncodedOutgoingMessage, EncodedOutgoingMessageInfo, MessageEncoding, MessageEncodingResult,
        MessageEncodingResultInfo, OutgoingMessageCapabilities, OutgoingMessageEncodingStatus,
        OutgoingMessageEncodingStatusInfo,
    };
    pub use crate::error::{MailKitError, NSErrorInfo};
    pub use crate::extension::{MailExtension, MailExtensionBuilder};
    pub use crate::extension_view_controller::ExtensionViewController;
    pub use crate::manager::ExtensionManager;
    pub use crate::message::{Message, MessageEncryptionState, MessageInfo, MessageState};
    pub use crate::message_action_extension::{MessageActionExtension, MessageActionHandler};
    pub use crate::message_security_handler::{
        MessageSecurityErrorCode, MessageSecurityHandler, MessageSecurityHandlerDelegate,
    };
}
