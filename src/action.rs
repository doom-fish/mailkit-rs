use core::ffi::c_void;
use std::mem::ManuallyDrop;
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::error::{take_owned_c_string, MailKitError};
use crate::ffi;
use crate::private::encode_json;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageColor {
    None = 0,
    Green = 1,
    Yellow = 2,
    Orange = 3,
    Red = 4,
    Purple = 5,
    Blue = 6,
    Gray = 7,
}

impl MessageColor {
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageFlag {
    None = 0,
    DefaultColor = 1,
    Red = 2,
    Orange = 3,
    Yellow = 4,
    Green = 5,
    Blue = 6,
    Purple = 7,
    Gray = 8,
}

impl MessageFlag {
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageActionKind {
    MoveToTrash,
    MoveToArchive,
    MoveToJunk,
    MarkAsRead,
    MarkAsUnread,
    Flag(MessageFlag),
    SetBackgroundColor(MessageColor),
}

pub struct MessageAction {
    pub(crate) raw: *mut c_void,
    kind: MessageActionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageActionDecisionKind {
    InvokeAgainWithBody,
    Action(MessageActionKind),
    Actions(Vec<MessageActionKind>),
}

pub struct MessageActionDecision {
    raw: *mut c_void,
    kind: MessageActionDecisionKind,
}

impl MessageAction {
    pub fn move_to_trash() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_move_to_trash() },
            MessageActionKind::MoveToTrash,
        )
    }

    pub fn move_to_archive() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_move_to_archive() },
            MessageActionKind::MoveToArchive,
        )
    }

    pub fn move_to_junk() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_move_to_junk() },
            MessageActionKind::MoveToJunk,
        )
    }

    pub fn mark_as_read() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_mark_as_read() },
            MessageActionKind::MarkAsRead,
        )
    }

    pub fn mark_as_unread() -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_mark_as_unread() },
            MessageActionKind::MarkAsUnread,
        )
    }

    pub fn flag(flag: MessageFlag) -> Result<Self, MailKitError> {
        Self::from_raw(
            unsafe { ffi::mail_message_action_flag(flag.as_i32()) },
            MessageActionKind::Flag(flag),
        )
    }

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
    pub const fn kind(&self) -> &MessageActionKind {
        &self.kind
    }
}

impl MessageActionDecision {
    pub fn invoke_again_with_body() -> Result<Self, MailKitError> {
        let raw = unsafe { ffi::mail_message_action_decision_invoke_again_with_body() };
        Self::from_raw(raw, MessageActionDecisionKind::InvokeAgainWithBody)
    }

    pub fn action(action: &MessageAction) -> Result<Self, MailKitError> {
        let raw = unsafe { ffi::mail_message_action_decision_action(action.raw) };
        Self::from_raw(raw, MessageActionDecisionKind::Action(action.kind.clone()))
    }

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
    pub const fn kind(&self) -> &MessageActionDecisionKind {
        &self.kind
    }

    pub(crate) fn into_raw(self) -> *mut c_void {
        let mut this = ManuallyDrop::new(self);
        let raw = this.raw;
        this.raw = ptr::null_mut();
        raw
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
        let kind =
            serde_json::from_str::<MessageActionDecisionKind>(&kind_json).map_err(|error| {
                MailKitError::FrameworkError(format!(
                    "failed to decode MailKit message-action decision box: {error}"
                ))
            })?;
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
