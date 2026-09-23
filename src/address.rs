use core::ffi::c_void;
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::error::{from_swift, MailKitError};
use crate::ffi;
use crate::private::to_cstring;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEEmailAddress`.
pub struct EmailAddressInfo {
    /// Preserves the original address string returned by MailKit.
    pub raw_string: String,
    /// Stores the normalized mailbox address when one is available.
    pub address_string: Option<String>,
}

/// Wraps `MEEmailAddress`.
pub struct EmailAddress {
    pub(crate) raw: *mut c_void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents MailKit address-annotation severities.
pub enum AddressAnnotationKind {
    /// Shows an error annotation in MailKit.
    Error,
    /// Shows a warning annotation in MailKit.
    Warning,
    /// Shows a success annotation in MailKit.
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEAddressAnnotation`.
pub struct AddressAnnotationInfo {
    /// Stores the MailKit annotation severity.
    pub kind: AddressAnnotationKind,
    /// Stores the localized annotation message shown by MailKit.
    pub localized_description: String,
}

/// Wraps `MEAddressAnnotation`.
pub struct AddressAnnotation {
    raw: *mut c_void,
    info: AddressAnnotationInfo,
}

impl EmailAddress {
    /// Creates a new `MEEmailAddress` from the supplied display string.
    pub fn new(raw_string: &str) -> Result<Self, MailKitError> {
        let raw_string = to_cstring(raw_string)?;
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_email_address_new(raw_string.as_ptr(), &raw mut raw, &raw mut error_ptr)
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self { raw })
    }

    /// Returns the original address string reported by MailKit.
    pub fn raw_string(&self) -> Result<String, MailKitError> {
        let ptr = unsafe { ffi::mail_email_address_raw_string(self.raw) };
        if ptr.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit returned a null MEEmailAddress.rawString".into(),
            ));
        }
        Ok(crate::error::take_owned_c_string(ptr))
    }

    /// Returns the normalized mailbox address when one is available.
    pub fn address_string(&self) -> Result<Option<String>, MailKitError> {
        let ptr = unsafe { ffi::mail_email_address_address_string(self.raw) };
        if ptr.is_null() {
            return Ok(None);
        }
        Ok(Some(crate::error::take_owned_c_string(ptr)))
    }

    /// Returns a snapshot of this `MEEmailAddress`.
    pub fn info(&self) -> Result<EmailAddressInfo, MailKitError> {
        Ok(EmailAddressInfo {
            raw_string: self.raw_string()?,
            address_string: self.address_string()?,
        })
    }
}

impl AddressAnnotation {
    /// Creates an error annotation for MailKit recipient validation.
    pub fn error(localized_description: &str) -> Result<Self, MailKitError> {
        Self::new(localized_description, AddressAnnotationKind::Error)
    }

    /// Creates a warning annotation for MailKit recipient validation.
    pub fn warning(localized_description: &str) -> Result<Self, MailKitError> {
        Self::new(localized_description, AddressAnnotationKind::Warning)
    }

    /// Creates a success annotation for MailKit recipient validation.
    pub fn success(localized_description: &str) -> Result<Self, MailKitError> {
        Self::new(localized_description, AddressAnnotationKind::Success)
    }

    fn new(localized_description: &str, kind: AddressAnnotationKind) -> Result<Self, MailKitError> {
        let localized_description = to_cstring(localized_description)?;
        let raw = unsafe {
            match kind {
                AddressAnnotationKind::Error => {
                    ffi::mail_address_annotation_error(localized_description.as_ptr())
                }
                AddressAnnotationKind::Warning => {
                    ffi::mail_address_annotation_warning(localized_description.as_ptr())
                }
                AddressAnnotationKind::Success => {
                    ffi::mail_address_annotation_success(localized_description.as_ptr())
                }
            }
        };
        if raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit failed to create an address annotation".into(),
            ));
        }
        Ok(Self {
            raw,
            info: AddressAnnotationInfo {
                kind,
                localized_description: localized_description.to_string_lossy().into_owned(),
            },
        })
    }

    #[must_use]
    /// Returns a snapshot of this `MEAddressAnnotation`.
    pub const fn info(&self) -> &AddressAnnotationInfo {
        &self.info
    }

    #[must_use]
    /// Returns the MailKit annotation severity.
    pub const fn kind(&self) -> AddressAnnotationKind {
        self.info.kind
    }

    #[must_use]
    /// Returns the localized description shown by MailKit.
    pub fn localized_description(&self) -> &str {
        &self.info.localized_description
    }
}

impl Drop for EmailAddress {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

impl Drop for AddressAnnotation {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}
