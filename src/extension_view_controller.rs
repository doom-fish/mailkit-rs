use core::ffi::c_void;
use std::ptr;

use crate::error::MailKitError;
use crate::ffi;

pub struct ExtensionViewController {
    pub(crate) raw: *mut c_void,
}

impl ExtensionViewController {
    pub fn new() -> Result<Self, MailKitError> {
        let raw = unsafe { ffi::core::mail_extension_view_controller_new() };
        if raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit failed to create an MEExtensionViewController".into(),
            ));
        }
        Ok(Self { raw })
    }

    pub(crate) fn from_raw(raw: *mut c_void) -> Result<Self, MailKitError> {
        if raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit returned a null MEExtensionViewController".into(),
            ));
        }
        Ok(Self { raw })
    }

    pub(crate) fn into_raw(self) -> *mut c_void {
        let mut this = std::mem::ManuallyDrop::new(self);
        let raw = this.raw;
        this.raw = ptr::null_mut();
        raw
    }
}

impl Drop for ExtensionViewController {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::core::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}
