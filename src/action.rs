use core::ffi::c_void;
use std::mem;
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::error::{take_owned_c_string, MailKitError};
use crate::ffi;
use crate::private::encode_json;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents MailKit message background colors used by `MEMessageAction`.
pub enum MessageColor {
    /// Does not apply a MailKit background color.
    None = 0,
    /// Applies the green MailKit background color.
    Green = 1,
    /// Applies the yellow MailKit background color.
    Yellow = 2,
    /// Applies the orange MailKit background color.
    Orange = 3,
    /// Applies the red MailKit background color.
    Red = 4,
    /// Applies the purple MailKit background color.
    Purple = 5,
    /// Applies the blue MailKit background color.
    Blue = 6,
    /// Applies the gray MailKit background color.
    Gray = 7,
}

impl MessageColor {
    #[must_use]
    /// Returns the MailKit integer constant for this background color.
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents MailKit flag values used by `MEMessageAction`.
pub enum MessageFlag {
    /// Clears MailKit flag coloring.
    None = 0,
    /// Uses MailKit's default flag color.
    DefaultColor = 1,
    /// Uses the red MailKit flag color.
    Red = 2,
    /// Uses the orange MailKit flag color.
    Orange = 3,
    /// Uses the yellow MailKit flag color.
    Yellow = 4,
    /// Uses the green MailKit flag color.
    Green = 5,
    /// Uses the blue MailKit flag color.
    Blue = 6,
    /// Uses the purple MailKit flag color.
    Purple = 7,
    /// Uses the gray MailKit flag color.
    Gray = 8,
}

impl MessageFlag {
    #[must_use]
    /// Returns the MailKit integer constant for this flag.
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Describes the logical action wrapped by `MEMessageAction`.
pub enum MessageActionKind {
    /// Moves the message to Trash.
    MoveToTrash,
    /// Moves the message to Archive.
    MoveToArchive,
    /// Moves the message to Junk.
    MoveToJunk,
    /// Marks the message as read.
    MarkAsRead,
    /// Marks the message as unread.
    MarkAsUnread,
    /// Flags the message with a MailKit flag color.
    Flag(MessageFlag),
    /// Applies a MailKit message background color.
    SetBackgroundColor(MessageColor),
}

/// Wraps `MEMessageAction`.
pub struct MessageAction {
    pub(crate) raw: *mut c_void,
    kind: MessageActionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Describes the logical decision wrapped by `MEMessageActionDecision`.
pub enum MessageActionDecisionKind {
    /// Asks MailKit to call the handler again after fetching message body data.
    InvokeAgainWithBody,
    /// Returns one MailKit message action.
    Action(MessageActionKind),
    /// Returns multiple MailKit message actions.
    Actions(Vec<MessageActionKind>),
}

/// Wraps `MEMessageActionDecision`.
pub struct MessageActionDecision {
    raw: *mut c_void,
    kind: MessageActionDecisionKind,
}

impl MessageAction {
    /// Creates an action that moves a message to Trash.
    pub fn move_to_trash() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_move_to_trash() },
            MessageActionKind::MoveToTrash,
        )
    }

    /// Creates an action that moves a message to Archive.
    pub fn move_to_archive() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_move_to_archive() },
            MessageActionKind::MoveToArchive,
        )
    }

    /// Creates an action that moves a message to Junk.
    pub fn move_to_junk() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_move_to_junk() },
            MessageActionKind::MoveToJunk,
        )
    }

    /// Creates an action that marks a message as read.
    pub fn mark_as_read() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_mark_as_read() },
            MessageActionKind::MarkAsRead,
        )
    }

    /// Creates an action that marks a message as unread.
    pub fn mark_as_unread() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_mark_as_unread() },
            MessageActionKind::MarkAsUnread,
        )
    }

    /// Creates an action that applies the supplied MailKit flag.
    pub fn flag(flag: MessageFlag) -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_flag(flag.as_i32()) },
            MessageActionKind::Flag(flag),
        )
    }

    /// Creates an action that applies the supplied MailKit background color.
    pub fn set_background_color(color: MessageColor) -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_set_background_color(color.as_i32()) },
            MessageActionKind::SetBackgroundColor(color),
        )
    }

    fn from_raw(raw: *mut c_void, kind: MessageActionKind) -> Result<Self, MailKitError> {
        if raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit failed to create an MEMessageAction".into(),
            ));
        }
        Ok(Self { raw, kind })
    }

    #[must_use]
    /// Returns the logical action snapshot for this wrapper.
    pub const fn kind(&self) -> &MessageActionKind {
        &self.kind
    }
}

impl MessageActionDecision {
    /// Creates a decision that asks MailKit to retry once message body data is available.
    pub fn invoke_again_with_body() -> Result<Self, MailKitError> {
        let raw = unsafe { ffi::mail_message_action_decision_invoke_again_with_body() };
        Self::from_raw(raw, MessageActionDecisionKind::InvokeAgainWithBody)
    }

    /// Creates a decision containing one message action.
    pub fn action(action: &MessageAction) -> Result<Self, MailKitError> {
        let raw = unsafe { ffi::mail_message_action_decision_action(action.raw) };
        Self::from_raw(raw, MessageActionDecisionKind::Action(action.kind.clone()))
    }

    /// Creates a decision containing multiple message actions.
    pub fn actions(actions: &[&MessageAction]) -> Result<Self, MailKitError> {
        if actions.is_empty() {
            return Err(MailKitError::InvalidArgument(
                "MessageActionDecision::actions requires at least one action".into(),
            ));
        }
        let raw_actions: Vec<*mut c_void> = actions.iter().map(|action| action.raw).collect();
        let raw = unsafe {
            ffi::mail_message_action_decision_actions(raw_actions.as_ptr(), raw_actions.len())
        };
        let kinds = actions.iter().map(|action| action.kind.clone()).collect();
        Self::from_raw(raw, MessageActionDecisionKind::Actions(kinds))
    }

    fn from_raw(raw: *mut c_void, kind: MessageActionDecisionKind) -> Result<Self, MailKitError> {
        if raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit failed to create an MEMessageActionDecision".into(),
            ));
        }
        Ok(Self { raw, kind })
    }

    #[must_use]
    /// Returns the logical decision snapshot for this wrapper.
    pub const fn kind(&self) -> &MessageActionDecisionKind {
        &self.kind
    }

    pub(crate) fn into_raw(mut self) -> *mut c_void {
        mem::replace(&mut self.raw, ptr::null_mut())
    }

    pub(crate) fn into_box_raw(self) -> Result<*mut c_void, MailKitError> {
        let kind_json = encode_json(&self.kind)?;
        let box_raw = unsafe {
            ffi::core::mail_message_action_decision_box_new(self.into_raw(), kind_json.as_ptr())
        };
        if box_raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit failed to allocate a message-action decision box".into(),
            ));
        }
        Ok(box_raw)
    }

    pub(crate) unsafe fn from_box_raw(box_raw: *mut c_void) -> Result<Self, MailKitError> {
        if box_raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit returned a null message-action decision box".into(),
            ));
        }
        let kind_json = take_owned_c_string(unsafe {
            ffi::core::mail_message_action_decision_box_copy_json(box_raw)
        });
        let kind = match serde_json::from_str::<MessageActionDecisionKind>(&kind_json) {
            Ok(kind) => kind,
            Err(error) => {
                unsafe { ffi::core::mail_object_release(box_raw) };
                return Err(MailKitError::FrameworkError(format!(
                    "failed to decode MailKit message-action decision box: {error}"
                )));
            }
        };
        let raw = unsafe { ffi::core::mail_message_action_decision_box_copy_value(box_raw) };
        unsafe { ffi::core::mail_object_release(box_raw) };
        Self::from_raw(raw, kind)
    }
}

impl Drop for MessageAction {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

impl Drop for MessageActionDecision {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}
