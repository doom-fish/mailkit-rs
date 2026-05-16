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
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::new_without_default
)]

pub mod action;
pub mod address;
pub mod authentication_result;
pub mod compose_session_handler;
pub mod content_blocker;
pub mod decoded_message;
pub mod eml_attachment;
pub mod encoded_outgoing_message;
pub mod error;
pub mod extension;
pub mod extension_view_controller;
pub mod ffi;
pub mod manager;
pub mod message;
pub mod message_action_extension;
pub mod message_security_handler;
mod private;
pub mod security;

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
    message_security_error_domain, MessageSecurityHandler, MessageSecurityHandlerDelegate,
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
        MessageSecurityHandler, MessageSecurityHandlerDelegate,
    };
}
