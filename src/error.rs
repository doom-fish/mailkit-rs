use core::ffi::c_char;
use core::fmt;

use libc::free;
use serde::{Deserialize, Serialize};

use crate::ffi;
use crate::message_security_handler::{message_security_error_domain, MessageSecurityErrorCode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a serializable snapshot of `NSError`.
pub struct NSErrorInfo {
    /// Stores the `NSError` domain.
    pub domain: String,
    /// Stores the `NSError` code.
    pub code: i64,
    /// Stores the localized error description.
    pub localized_description: String,
}

impl NSErrorInfo {
    #[must_use]
    /// Creates a serializable `NSError` snapshot.
    pub fn new(
        domain: impl Into<String>,
        code: i64,
        localized_description: impl Into<String>,
    ) -> Self {
        Self {
            domain: domain.into(),
            code,
            localized_description: localized_description.into(),
        }
    }

    #[must_use]
    /// Converts this `NSError` into a MailKit message-security error code when the domains match.
    pub fn message_security_error_code(&self) -> Option<MessageSecurityErrorCode> {
        if self.domain == message_security_error_domain() {
            MessageSecurityErrorCode::from_code(self.code)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// Represents errors produced by the safe MailKit wrappers.
pub enum MailKitError {
    /// Indicates that Rust code passed an invalid argument into MailKit.
    InvalidArgument(String),
    /// Wraps an `NSError` returned by MailKit.
    OperationFailed(Box<NSErrorInfo>),
    /// Reports a framework-side failure without structured `NSError` data.
    FrameworkError(String),
    /// Reports that a MailKit operation timed out.
    Timeout(String),
    /// Captures an unrecognized MailKit status code.
    Unknown {
        /// Stores the raw MailKit status code.
        code: i32,
        /// Stores the framework message that accompanied the status code.
        message: String,
    },
}

impl MailKitError {
    #[must_use]
    /// Returns the MailKit status code represented by this error.
    pub const fn code(&self) -> i32 {
        match self {
            Self::InvalidArgument(_) => ffi::status::INVALID_ARGUMENT,
            Self::OperationFailed(_) => ffi::status::OPERATION_FAILED,
            Self::FrameworkError(_) => ffi::status::FRAMEWORK_ERROR,
            Self::Timeout(_) => ffi::status::TIMED_OUT,
            Self::Unknown { code, .. } => *code,
        }
    }

    #[must_use]
    /// Returns the human-readable message for this error.
    pub fn message(&self) -> String {
        match self {
            Self::OperationFailed(error) => error.localized_description.clone(),
            Self::InvalidArgument(message)
            | Self::FrameworkError(message)
            | Self::Timeout(message)
            | Self::Unknown { message, .. } => message.clone(),
        }
    }
}

impl fmt::Display for MailKitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (code {})", self.message(), self.code())
    }
}

impl std::error::Error for MailKitError {}

pub(crate) fn take_owned_c_string(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let string = unsafe { core::ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { free(ptr.cast()) };
    string
}

pub(crate) fn from_swift(status: i32, error_str: *mut c_char) -> MailKitError {
    let message = take_owned_c_string(error_str);
    if status == ffi::status::OPERATION_FAILED {
        if let Ok(error) = serde_json::from_str::<NSErrorInfo>(&message) {
            return MailKitError::OperationFailed(Box::new(error));
        }
    }
    from_status_message(status, message)
}

pub(crate) fn from_status_message(status: i32, message: String) -> MailKitError {
    match status {
        ffi::status::INVALID_ARGUMENT => MailKitError::InvalidArgument(message),
        ffi::status::FRAMEWORK_ERROR => MailKitError::FrameworkError(message),
        ffi::status::TIMED_OUT => MailKitError::Timeout(message),
        code => MailKitError::Unknown { code, message },
    }
}
