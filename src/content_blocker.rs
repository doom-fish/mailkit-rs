use core::ffi::{c_char, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use crate::error::{from_swift, MailKitError};
use crate::ffi;
use crate::private::{decode_json_box, encode_json_box, write_error_out};

/// Provides the content rules that back `MEContentBlocker`.
pub trait ContentBlockerProvider: Send + Sync + 'static {
    /// Returns the JSON rule payload that should back `MEContentBlocker`.
    fn content_rules_json(&self) -> Result<Vec<u8>, MailKitError>;
}

struct ContentBlockerState {
    provider: Box<dyn ContentBlockerProvider>,
}

/// Wraps `MEContentBlocker`.
pub struct ContentBlocker {
    raw: *mut c_void,
}

impl ContentBlocker {
    #[must_use]
    /// Returns the retained native MailKit object pointer.
    pub const fn raw(&self) -> *mut c_void {
        self.raw
    }

    /// Creates a `MEContentBlocker` wrapper from a Rust provider.
    pub fn new<H: ContentBlockerProvider>(provider: H) -> Result<Self, MailKitError> {
        let state = Box::new(ContentBlockerState {
            provider: Box::new(provider),
        });
        let user_data = Box::into_raw(state).cast::<c_void>();
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::content_blocker::mail_content_blocker_new(
                Some(content_rules_callback),
                Some(release_user_data),
                user_data,
                &mut raw,
                &mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            unsafe {
                drop(Box::from_raw(user_data.cast::<ContentBlockerState>()));
            }
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self { raw })
    }

    /// Returns the content blocker rules as JSON bytes.
    pub fn content_rules_json(&self) -> Result<Vec<u8>, MailKitError> {
        let mut error_ptr = ptr::null_mut();
        let result = unsafe {
            ffi::content_blocker::mail_content_blocker_test_content_rules_json(
                self.raw,
                &mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        if result.is_null() {
            return Ok(Vec::new());
        }
        unsafe { decode_json_box(result) }
    }
}

impl Drop for ContentBlocker {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::core::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

extern "C" fn content_rules_callback(
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    match catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_data.cast::<ContentBlockerState>() };
        let rules = state.provider.content_rules_json()?;
        encode_json_box(&rules)
    })) {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            unsafe { write_error_out(error_out, &error.to_string()) };
            ptr::null_mut()
        }
        Err(_) => {
            unsafe { write_error_out(error_out, "content blocker callback panicked") };
            ptr::null_mut()
        }
    }
}

extern "C" fn release_user_data(user_data: *mut c_void) {
    if !user_data.is_null() {
        unsafe {
            drop(Box::from_raw(user_data.cast::<ContentBlockerState>()));
        }
    }
}
