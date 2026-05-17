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
//!
//! ## Limitations
//!
//! `MailKit` is primarily intended for use in Mail extensions rather than regular processes.
//! Most async operations below are available in extension contexts only:
//!
//! - **`ContentBlocker.contentRulesURL()`** — Extension API only (macOS 12.0+)
//! - **`MEMessageActionHandler.decide()`** — Extension API only (macOS 12.0+)
//! - **`MEMessageSecurityHandler.{sign,encrypt,decode,decrypt}()`** — Extension APIs (macOS 12.0+)
//! - **`MEComposeSessionHandler`** — Extension API only (macOS 14.0+)
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

use crate::error::MailKitError;
use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
use std::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
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
    if !error.is_null() {
        let error_msg = unsafe { error_from_cstr(error) };
        unsafe {
            AsyncCompletion::<crate::action::MessageActionDecision>::complete_err(
                user_data,
                error_msg,
            );
        }
    } else if !result.is_null() {
        let decision = unsafe {
            crate::action::MessageActionDecision::from_box_raw(result.cast_mut())
        };
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
}

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
    if !error.is_null() {
        let error_msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<Vec<u8>>::complete_err(user_data, error_msg); }
    } else if !result.is_null() {
        let rules = unsafe { crate::private::decode_json_box::<Vec<u8>>(result.cast_mut()) };
        match rules {
            Ok(r) => unsafe { AsyncCompletion::complete_ok(user_data, r); },
            Err(e) => unsafe {
                AsyncCompletion::<Vec<u8>>::complete_err(user_data, e.to_string());
            },
        }
    } else {
        unsafe {
            AsyncCompletion::<Vec<u8>>::complete_err(
                user_data,
                "Unknown error".to_string(),
            );
        }
    }
}

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
