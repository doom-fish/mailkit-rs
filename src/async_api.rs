//! Async API for `MailKit`
//!
//! This module provides async versions of `MailKit` operations when the `async` feature is enabled.
//! The async API is **executor-agnostic** and works with any async runtime (Tokio, async-std, smol, etc.).
//!
//! ## Available Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`AsyncContentBlocker`] | Async content rules retrieval |
//! | [`AsyncMessageActionDecision`] | Async message action decisions |
//! | [`AsyncComposeSessionHandler`] | Async compose-session validation callbacks |
//! | [`AsyncMessageSecurityHandler`] | Async message-security encoding and UI hooks |
//! | [`AsyncExtensionManager`] | Async extension-manager reload requests |
//!
//! ## Limitations
//!
//! `MailKit` is primarily intended for use in Mail extensions rather than regular processes.
//! Most async operations below are available in extension contexts only:
//!
//! - **`ContentBlocker.contentRulesURL()`** — Extension API only (macOS 12.0+)
//! - **`MEMessageActionHandler.decide()`** — Extension API only (macOS 12.0+)
//! - **`MEComposeSessionHandler`** — Extension API only (macOS 14.0+)
//! - **`MEMessageEncoder` / `MEMessageSecurityHandler` completion handlers** — Extension APIs (macOS 12.0+)
//! - **`MEExtensionManager.reloadVisibleMessages` / `reloadContentBlocker`** — Mail-hosted APIs (macOS 12.0+)
//!
//! The Rust async wrappers below can still be useful for **testing and prototyping** in
//! regular processes when the underlying Swift async methods are available.
//!
//! ## Runtime Agnostic Design
//!
//! This async API uses only `std` types and works with **any** async runtime:
//! - Uses callback-based Swift FFI for true async operations
//! - Uses `std::sync::{Arc, Mutex}` for synchronization
//! - Uses `std::task::{Poll, Waker}` for async primitives
//! - Uses `std::future::Future` trait
//!
//! ## Examples
//!
//! ### Async Message Action Decision
//!
//! ```rust,ignore
//! use mailkit::async_api::AsyncMessageActionDecision;
//! use mailkit::Message;
//!
//! // Create a message (typically from extension context)
//! async {
//!     let decision = AsyncMessageActionDecision::decide(&message).await?;
//!     if let Some(decision) = decision {
//!         println!("Got decision");
//!     }
//!     Ok::<(), Box<dyn std::error::Error>>(())
//! }
//! ```

use crate::error::{from_swift, MailKitError, NSErrorInfo};
use crate::private::{encode_json, encode_json_box, to_cstring};
use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
use std::ffi::c_void;
use std::future::Future;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll};

// ============================================================================
// AsyncMessageActionDecision - Async message action handling
// ============================================================================

/// Callback from Swift FFI for message action decision
extern "C" fn message_action_decision_callback(
    result: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !error.is_null() {
            let error_msg = unsafe { error_from_cstr(error) };
            unsafe {
                AsyncCompletion::<crate::action::MessageActionDecision>::complete_err(
                    user_data, error_msg,
                );
            }
        } else if !result.is_null() {
            let decision =
                unsafe { crate::action::MessageActionDecision::from_box_raw(result.cast_mut()) };
            match decision {
                Ok(dec) => unsafe {
                    AsyncCompletion::complete_ok(user_data, dec);
                },
                Err(e) => unsafe {
                    AsyncCompletion::<crate::action::MessageActionDecision>::complete_err(
                        user_data,
                        e.to_string(),
                    );
                },
            }
        } else {
            unsafe {
                AsyncCompletion::<crate::action::MessageActionDecision>::complete_err(
                    user_data,
                    "Unknown error".to_string(),
                );
            }
        }
    }));
}

/// Future returned by `AsyncMessageActionDecision::decide`.
pub struct MessageActionDecisionFuture {
    inner: AsyncCompletionFuture<crate::action::MessageActionDecision>,
}

impl Future for MessageActionDecisionFuture {
    type Output = Result<Option<crate::action::MessageActionDecision>, MailKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map(Some).map_err(MailKitError::FrameworkError))
    }
}

/// Provides async access to MailKit message-action decisions.
pub struct AsyncMessageActionDecision;

impl AsyncMessageActionDecision {
    /// Asynchronously decide an action for a message (extension API)
    pub fn decide(message: &crate::message::Message) -> MessageActionDecisionFuture {
        let (future, ctx) = AsyncCompletion::create();
        let message_info = message.info().clone();
        let message_box = match crate::private::encode_json_box(&message_info) {
            Ok(b) => b,
            Err(e) => {
                unsafe {
                    AsyncCompletion::<crate::action::MessageActionDecision>::complete_err(
                        ctx,
                        e.to_string(),
                    );
                }
                return MessageActionDecisionFuture { inner: future };
            }
        };

        unsafe {
            crate::ffi::message_action_extension::mail_message_action_decision_async(
                message_box,
                message_action_decision_callback,
                ctx,
            );
        }

        MessageActionDecisionFuture { inner: future }
    }
}

// ============================================================================
// AsyncContentBlocker - Async content rules retrieval
// ============================================================================

/// Callback from Swift FFI for content blocker content rules URL
extern "C" fn content_blocker_content_rules_callback(
    result: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !error.is_null() {
            let error_msg = unsafe { error_from_cstr(error) };
            unsafe {
                AsyncCompletion::<Vec<u8>>::complete_err(user_data, error_msg);
            }
        } else if !result.is_null() {
            let rules = unsafe { crate::private::decode_json_box::<Vec<u8>>(result.cast_mut()) };
            match rules {
                Ok(r) => unsafe {
                    AsyncCompletion::complete_ok(user_data, r);
                },
                Err(e) => unsafe {
                    AsyncCompletion::<Vec<u8>>::complete_err(user_data, e.to_string());
                },
            }
        } else {
            unsafe {
                AsyncCompletion::<Vec<u8>>::complete_err(user_data, "Unknown error".to_string());
            }
        }
    }));
}

/// Future returned by `AsyncContentBlocker::content_rules_json`.
pub struct ContentBlockerContentRulesFuture {
    inner: AsyncCompletionFuture<Vec<u8>>,
}

impl Future for ContentBlockerContentRulesFuture {
    type Output = Result<Vec<u8>, MailKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(MailKitError::FrameworkError))
    }
}

/// Provides async access to MailKit content-blocker rules.
pub struct AsyncContentBlocker;

impl AsyncContentBlocker {
    /// Asynchronously retrieve content rules from a content blocker (extension API)
    pub fn content_rules_json(
        content_blocker: &crate::content_blocker::ContentBlocker,
    ) -> ContentBlockerContentRulesFuture {
        let (future, ctx) = AsyncCompletion::create();

        unsafe {
            crate::ffi::content_blocker::mail_content_blocker_content_rules_async(
                content_blocker.raw(),
                content_blocker_content_rules_callback,
                ctx,
            );
        }

        ContentBlockerContentRulesFuture { inner: future }
    }
}

fn async_error_to_mailkit(error: String) -> MailKitError {
    serde_json::from_str::<NSErrorInfo>(&error)
        .map(|error| MailKitError::OperationFailed(Box::new(error)))
        .unwrap_or(MailKitError::FrameworkError(error))
}

fn complete_async_setup_error<T>(context: *mut c_void, error: &MailKitError) {
    unsafe {
        AsyncCompletion::<T>::complete_err(context, error.message());
    }
}

macro_rules! mailkit_result_future {
    ($name:ident, $output:ty) => {
        /// Future returned by the corresponding MailKit async wrapper.
        pub struct $name {
            inner: AsyncCompletionFuture<$output>,
        }

        impl Future for $name {
            type Output = Result<$output, MailKitError>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                Pin::new(&mut self.inner)
                    .poll(cx)
                    .map(|r| r.map_err(async_error_to_mailkit))
            }
        }
    };
}

mailkit_result_future!(
    ComposeSessionAnnotationsFuture,
    Vec<crate::compose_session_handler::AddressAnnotationMatch>
);
mailkit_result_future!(ComposeSessionAllowMessageSendFuture, ());
mailkit_result_future!(
    MessageSecurityEncodingStatusFuture,
    crate::security::OutgoingMessageEncodingStatus
);
mailkit_result_future!(
    MessageSecurityEncodeMessageFuture,
    crate::security::MessageEncodingResult
);
mailkit_result_future!(
    MessageSecurityPrimaryActionFuture,
    Option<crate::extension_view_controller::ExtensionViewController>
);
mailkit_result_future!(ReloadVisibleMessagesFuture, ());
mailkit_result_future!(ReloadContentBlockerFuture, ());

fn compose_session_raw(
    session: &crate::compose_session_handler::ComposeSession,
) -> Result<*mut c_void, MailKitError> {
    let json = encode_json(session.info())?;
    let mut raw = ptr::null_mut();
    let mut error_ptr = ptr::null_mut();
    let status = unsafe {
        crate::ffi::compose_session_handler::mail_compose_session_new(
            json.as_ptr(),
            &mut raw,
            &mut error_ptr,
        )
    };
    if status != crate::ffi::status::OK {
        return Err(from_swift(status, error_ptr));
    }
    Ok(raw)
}

fn message_and_context_boxes(
    message: &crate::message::Message,
    compose_context: &crate::compose_session_handler::ComposeContext,
) -> Result<(*mut c_void, *mut c_void), MailKitError> {
    let message_box = encode_json_box(message.info())?;
    match encode_json_box(compose_context.info()) {
        Ok(context_box) => Ok((message_box, context_box)),
        Err(error) => {
            unsafe {
                crate::ffi::core::mail_object_release(message_box);
            }
            Err(error)
        }
    }
}

extern "C" fn compose_session_annotations_callback(
    result: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !error.is_null() {
            let error_msg = unsafe { error_from_cstr(error) };
            unsafe {
                AsyncCompletion::<Vec<crate::compose_session_handler::AddressAnnotationMatch>>::complete_err(
                    user_data,
                    error_msg,
                );
            }
        } else if !result.is_null() {
            match unsafe {
                crate::private::decode_json_box::<
                    Vec<crate::compose_session_handler::AddressAnnotationMatch>,
                >(result.cast_mut())
            } {
                Ok(matches) => unsafe {
                    AsyncCompletion::complete_ok(user_data, matches);
                },
                Err(error) => unsafe {
                    AsyncCompletion::<Vec<crate::compose_session_handler::AddressAnnotationMatch>>::complete_err(
                        user_data,
                        error.message(),
                    );
                },
            }
        } else {
            unsafe {
                AsyncCompletion::<Vec<crate::compose_session_handler::AddressAnnotationMatch>>::complete_err(
                    user_data,
                    "Unknown error".to_string(),
                );
            }
        }
    }));
}

extern "C" fn unit_async_callback(result: *const c_void, error: *const i8, user_data: *mut c_void) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _ = result;
        if error.is_null() {
            unsafe {
                AsyncCompletion::complete_ok(user_data, ());
            }
        } else {
            let error_msg = unsafe { error_from_cstr(error) };
            unsafe {
                AsyncCompletion::<()>::complete_err(user_data, error_msg);
            }
        }
    }));
}

extern "C" fn message_security_encoding_status_callback(
    result: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !error.is_null() {
            let error_msg = unsafe { error_from_cstr(error) };
            unsafe {
                AsyncCompletion::<crate::security::OutgoingMessageEncodingStatus>::complete_err(
                    user_data, error_msg,
                );
            }
        } else if !result.is_null() {
            match unsafe {
                crate::security::OutgoingMessageEncodingStatus::from_box_raw(result.cast_mut())
            } {
                Ok(status) => unsafe {
                    AsyncCompletion::complete_ok(user_data, status);
                },
                Err(error) => unsafe {
                    AsyncCompletion::<crate::security::OutgoingMessageEncodingStatus>::complete_err(
                        user_data,
                        error.message(),
                    );
                },
            }
        } else {
            unsafe {
                AsyncCompletion::<crate::security::OutgoingMessageEncodingStatus>::complete_err(
                    user_data,
                    "Unknown error".to_string(),
                );
            }
        }
    }));
}

extern "C" fn message_security_encode_message_callback(
    result: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !error.is_null() {
            let error_msg = unsafe { error_from_cstr(error) };
            unsafe {
                AsyncCompletion::<crate::security::MessageEncodingResult>::complete_err(
                    user_data, error_msg,
                );
            }
        } else if !result.is_null() {
            match unsafe { crate::security::MessageEncodingResult::from_box_raw(result.cast_mut()) }
            {
                Ok(result) => unsafe {
                    AsyncCompletion::complete_ok(user_data, result);
                },
                Err(error) => unsafe {
                    AsyncCompletion::<crate::security::MessageEncodingResult>::complete_err(
                        user_data,
                        error.message(),
                    );
                },
            }
        } else {
            unsafe {
                AsyncCompletion::<crate::security::MessageEncodingResult>::complete_err(
                    user_data,
                    "Unknown error".to_string(),
                );
            }
        }
    }));
}

extern "C" fn message_security_primary_action_callback(
    result: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !error.is_null() {
            let error_msg = unsafe { error_from_cstr(error) };
            unsafe {
                AsyncCompletion::<Option<crate::extension_view_controller::ExtensionViewController>>::complete_err(
                    user_data,
                    error_msg,
                );
            }
        } else if result.is_null() {
            unsafe {
                AsyncCompletion::complete_ok(
                    user_data,
                    None::<crate::extension_view_controller::ExtensionViewController>,
                );
            }
        } else {
            match crate::extension_view_controller::ExtensionViewController::from_raw(
                result.cast_mut(),
            ) {
                Ok(controller) => unsafe {
                    AsyncCompletion::complete_ok(user_data, Some(controller));
                },
                Err(error) => unsafe {
                    AsyncCompletion::<
                        Option<crate::extension_view_controller::ExtensionViewController>,
                    >::complete_err(user_data, error.message());
                },
            }
        }
    }));
}

/// Provides async access to MailKit compose-session validation callbacks.
pub struct AsyncComposeSessionHandler;

impl AsyncComposeSessionHandler {
    /// Asynchronously collect address annotations for a compose session.
    pub fn annotate_addresses_for_session(
        handler: &crate::compose_session_handler::ComposeSessionHandler,
        session: &crate::compose_session_handler::ComposeSession,
    ) -> ComposeSessionAnnotationsFuture {
        let (future, ctx) = AsyncCompletion::create();
        let session_raw = match compose_session_raw(session) {
            Ok(session_raw) => session_raw,
            Err(error) => {
                complete_async_setup_error::<
                    Vec<crate::compose_session_handler::AddressAnnotationMatch>,
                >(ctx, &error);
                return ComposeSessionAnnotationsFuture { inner: future };
            }
        };

        unsafe {
            crate::ffi::compose_session_handler::mail_compose_session_handler_annotate_addresses_async(
                handler.raw(),
                session_raw,
                compose_session_annotations_callback,
                ctx,
            );
        }

        ComposeSessionAnnotationsFuture { inner: future }
    }

    /// Asynchronously ask whether MailKit may send a compose session.
    pub fn allow_message_send_for_session(
        handler: &crate::compose_session_handler::ComposeSessionHandler,
        session: &crate::compose_session_handler::ComposeSession,
    ) -> ComposeSessionAllowMessageSendFuture {
        let (future, ctx) = AsyncCompletion::create();
        let session_raw = match compose_session_raw(session) {
            Ok(session_raw) => session_raw,
            Err(error) => {
                complete_async_setup_error::<()>(ctx, &error);
                return ComposeSessionAllowMessageSendFuture { inner: future };
            }
        };

        unsafe {
            crate::ffi::compose_session_handler::mail_compose_session_handler_allow_message_send_async(
                handler.raw(),
                session_raw,
                unit_async_callback,
                ctx,
            );
        }

        ComposeSessionAllowMessageSendFuture { inner: future }
    }
}

/// Provides async access to MailKit message-security callbacks.
pub struct AsyncMessageSecurityHandler;

impl AsyncMessageSecurityHandler {
    /// Asynchronously request an outgoing-message encoding status.
    pub fn encoding_status_for_message(
        handler: &crate::message_security_handler::MessageSecurityHandler,
        message: &crate::message::Message,
        compose_context: &crate::compose_session_handler::ComposeContext,
    ) -> MessageSecurityEncodingStatusFuture {
        let (future, ctx) = AsyncCompletion::create();
        let (message_box, context_box) = match message_and_context_boxes(message, compose_context) {
            Ok(boxes) => boxes,
            Err(error) => {
                complete_async_setup_error::<crate::security::OutgoingMessageEncodingStatus>(
                    ctx, &error,
                );
                return MessageSecurityEncodingStatusFuture { inner: future };
            }
        };

        unsafe {
            crate::ffi::security::mail_message_security_handler_encoding_status_async(
                handler.raw(),
                message_box,
                context_box,
                message_security_encoding_status_callback,
                ctx,
            );
        }

        MessageSecurityEncodingStatusFuture { inner: future }
    }

    /// Asynchronously encode an outgoing message.
    pub fn encode_message(
        handler: &crate::message_security_handler::MessageSecurityHandler,
        message: &crate::message::Message,
        compose_context: &crate::compose_session_handler::ComposeContext,
    ) -> MessageSecurityEncodeMessageFuture {
        let (future, ctx) = AsyncCompletion::create();
        let (message_box, context_box) = match message_and_context_boxes(message, compose_context) {
            Ok(boxes) => boxes,
            Err(error) => {
                complete_async_setup_error::<crate::security::MessageEncodingResult>(ctx, &error);
                return MessageSecurityEncodeMessageFuture { inner: future };
            }
        };

        unsafe {
            crate::ffi::security::mail_message_security_handler_encode_message_async(
                handler.raw(),
                message_box,
                context_box,
                message_security_encode_message_callback,
                ctx,
            );
        }

        MessageSecurityEncodeMessageFuture { inner: future }
    }

    /// Asynchronously handle MailKit's primary-action click for banner context data.
    pub fn primary_action_clicked_for_message_context(
        handler: &crate::message_security_handler::MessageSecurityHandler,
        context: &[u8],
    ) -> MessageSecurityPrimaryActionFuture {
        let (future, ctx) = AsyncCompletion::create();
        let context_box = match encode_json_box(&context) {
            Ok(context_box) => context_box,
            Err(error) => {
                complete_async_setup_error::<
                    Option<crate::extension_view_controller::ExtensionViewController>,
                >(ctx, &error);
                return MessageSecurityPrimaryActionFuture { inner: future };
            }
        };

        unsafe {
            crate::ffi::security::mail_message_security_handler_primary_action_clicked_async(
                handler.raw(),
                context_box,
                message_security_primary_action_callback,
                ctx,
            );
        }

        MessageSecurityPrimaryActionFuture { inner: future }
    }
}

/// Provides async access to `MEExtensionManager` reload requests.
pub struct AsyncExtensionManager;

impl AsyncExtensionManager {
    /// Asynchronously request MailKit to reload messages visible in Mail.
    pub fn reload_visible_messages() -> ReloadVisibleMessagesFuture {
        let (future, ctx) = AsyncCompletion::create();

        unsafe {
            crate::ffi::manager::mail_extension_manager_reload_visible_messages_async(
                unit_async_callback,
                ctx,
            );
        }

        ReloadVisibleMessagesFuture { inner: future }
    }

    /// Asynchronously request MailKit to reload the named content blocker.
    pub fn reload_content_blocker(identifier: &str) -> ReloadContentBlockerFuture {
        let (future, ctx) = AsyncCompletion::create();
        let identifier = match to_cstring(identifier) {
            Ok(identifier) => identifier,
            Err(error) => {
                complete_async_setup_error::<()>(ctx, &error);
                return ReloadContentBlockerFuture { inner: future };
            }
        };

        unsafe {
            crate::ffi::manager::mail_extension_manager_reload_content_blocker_async(
                identifier.as_ptr(),
                unit_async_callback,
                ctx,
            );
        }

        ReloadContentBlockerFuture { inner: future }
    }
}
