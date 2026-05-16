use core::ffi::c_void;
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::error::{from_swift, MailKitError};
use crate::ffi;
use crate::private::to_cstring;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAddressInfo {
    pub raw_string: String,
    pub address_string: Option<String>,
}

pub struct EmailAddress {
    pub(crate) raw: *mut c_void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AddressAnnotationKind {
    Error,
    Warning,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressAnnotationInfo {
    pub kind: AddressAnnotationKind,
    pub localized_description: String,
}

pub struct AddressAnnotation {
    raw: *mut c_void,
    info: AddressAnnotationInfo,
}

impl EmailAddress {
    pub fn new(raw_string: &str) -> Result<Self, MailKitError> {
        let raw_string = to_cstring(raw_string)?;
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status =
            unsafe { ffi::mail_email_address_new(raw_string.as_ptr(), &mut raw, &mut error_ptr) };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self { raw })
    }

    pub fn raw_string(&self) -> Result<String, MailKitError> {
        let ptr = unsafe { ffi::mail_email_address_raw_string(self.raw) };
        if ptr.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit returned a null MEEmailAddress.rawString".into(),
            ));
        }
        Ok(crate::error::take_owned_c_string(ptr))
    }

    pub fn address_string(&self) -> Result<Option<String>, MailKitError> {
        let ptr = unsafe { ffi::mail_email_address_address_string(self.raw) };
        if ptr.is_null() {
            return Ok(None);
        }
        Ok(Some(crate::error::take_owned_c_string(ptr)))
    }

    pub fn info(&self) -> Result<EmailAddressInfo, MailKitError> {
        Ok(EmailAddressInfo {
            raw_string: self.raw_string()?,
            address_string: self.address_string()?,
        })
    }
}

impl AddressAnnotation {
    pub fn error(localized_description: &str) -> Result<Self, MailKitError> {
        Self::new(localized_description, AddressAnnotationKind::Error)
    }

    pub fn warning(localized_description: &str) -> Result<Self, MailKitError> {
        Self::new(localized_description, AddressAnnotationKind::Warning)
    }

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
    pub const fn info(&self) -> &AddressAnnotationInfo {
        &self.info
    }

    #[must_use]
    pub const fn kind(&self) -> AddressAnnotationKind {
        self.info.kind
    }

    #[must_use]
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
