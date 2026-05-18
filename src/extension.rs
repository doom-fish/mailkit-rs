use core::ffi::c_void;
use std::ptr;

use crate::compose_session_handler::ComposeSessionHandler;
use crate::content_blocker::ContentBlocker;
use crate::error::{from_swift, MailKitError};
use crate::ffi;
use crate::message_action_extension::MessageActionExtension;
use crate::message_security_handler::MessageSecurityHandler;

/// Wraps `MEExtension`.
pub struct MailExtension {
    raw: *mut c_void,
    compose_session_handler: Option<ComposeSessionHandler>,
    message_action_handler: Option<MessageActionExtension>,
    content_blocker: Option<ContentBlocker>,
    message_security_handler: Option<MessageSecurityHandler>,
}

/// Alias matching MailKit's `MEExtension` name.
pub type MEExtension = MailExtension;

impl MailExtension {
    #[must_use]
    /// Returns a builder for configuring a MailKit extension.
    pub const fn builder() -> MailExtensionBuilder {
        MailExtensionBuilder::new()
    }

    #[must_use]
    /// Returns the registered compose-session handler, when present.
    pub const fn compose_session_handler(&self) -> Option<&ComposeSessionHandler> {
        self.compose_session_handler.as_ref()
    }

    #[must_use]
    /// Returns the registered message-action handler, when present.
    pub const fn message_action_handler(&self) -> Option<&MessageActionExtension> {
        self.message_action_handler.as_ref()
    }

    #[must_use]
    /// Returns the registered content blocker, when present.
    pub const fn content_blocker(&self) -> Option<&ContentBlocker> {
        self.content_blocker.as_ref()
    }

    #[must_use]
    /// Returns the registered message-security handler, when present.
    pub const fn message_security_handler(&self) -> Option<&MessageSecurityHandler> {
        self.message_security_handler.as_ref()
    }

    #[must_use]
    /// Returns whether this extension exposes a compose-session handler.
    pub fn has_compose_session_handler(&self) -> bool {
        unsafe { ffi::extension::mail_extension_test_has_compose_session_handler(self.raw) }
    }

    #[must_use]
    /// Returns whether this extension exposes a message-action handler.
    pub fn has_message_action_handler(&self) -> bool {
        unsafe { ffi::extension::mail_extension_test_has_message_action_handler(self.raw) }
    }

    #[must_use]
    /// Returns whether this extension exposes a content blocker.
    pub fn has_content_blocker(&self) -> bool {
        unsafe { ffi::extension::mail_extension_test_has_content_blocker(self.raw) }
    }

    #[must_use]
    /// Returns whether this extension exposes a message-security handler.
    pub fn has_message_security_handler(&self) -> bool {
        unsafe { ffi::extension::mail_extension_test_has_message_security_handler(self.raw) }
    }
}

impl Drop for MailExtension {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::core::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

#[derive(Default)]
/// Builds a `MEExtension` registration from Rust handlers.
pub struct MailExtensionBuilder {
    compose_session_handler: Option<ComposeSessionHandler>,
    message_action_handler: Option<MessageActionExtension>,
    content_blocker: Option<ContentBlocker>,
    message_security_handler: Option<MessageSecurityHandler>,
}

impl MailExtensionBuilder {
    #[must_use]
    /// Creates an empty MailKit extension builder.
    pub const fn new() -> Self {
        Self {
            compose_session_handler: None,
            message_action_handler: None,
            content_blocker: None,
            message_security_handler: None,
        }
    }

    #[must_use]
    /// Registers a compose-session handler with the builder.
    pub fn with_compose_session_handler(mut self, handler: ComposeSessionHandler) -> Self {
        self.compose_session_handler = Some(handler);
        self
    }

    #[must_use]
    /// Registers a message-action handler with the builder.
    pub fn with_message_action_handler(mut self, handler: MessageActionExtension) -> Self {
        self.message_action_handler = Some(handler);
        self
    }

    #[must_use]
    /// Registers a content blocker with the builder.
    pub fn with_content_blocker(mut self, blocker: ContentBlocker) -> Self {
        self.content_blocker = Some(blocker);
        self
    }

    #[must_use]
    /// Registers a message-security handler with the builder.
    pub fn with_message_security_handler(mut self, handler: MessageSecurityHandler) -> Self {
        self.message_security_handler = Some(handler);
        self
    }

    /// Builds the `MEExtension` wrapper from the registered handlers.
    pub fn build(self) -> Result<MailExtension, MailKitError> {
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::extension::mail_extension_new(
                self.compose_session_handler
                    .as_ref()
                    .map_or(ptr::null_mut(), ComposeSessionHandler::raw),
                self.message_action_handler
                    .as_ref()
                    .map_or(ptr::null_mut(), MessageActionExtension::raw),
                self.content_blocker
                    .as_ref()
                    .map_or(ptr::null_mut(), ContentBlocker::raw),
                self.message_security_handler
                    .as_ref()
                    .map_or(ptr::null_mut(), MessageSecurityHandler::raw),
                &mut raw,
                &mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(MailExtension {
            raw,
            compose_session_handler: self.compose_session_handler,
            message_action_handler: self.message_action_handler,
            content_blocker: self.content_blocker,
            message_security_handler: self.message_security_handler,
        })
    }
}
