use std::ffi::CString;

use serde::{de::DeserializeOwned, Serialize};

use crate::error::{take_owned_c_string, MailKitError};
use crate::ffi;

/// Converts a Rust string into a NUL-terminated C string accepted by the MailKit bridge.
pub fn to_cstring(value: &str) -> Result<CString, MailKitError> {
    CString::new(value).map_err(|_| {
        MailKitError::InvalidArgument(
            "strings passed to MailKit cannot contain interior NUL bytes".into(),
        )
    })
}

pub fn encode_json<T: Serialize>(value: &T) -> Result<CString, MailKitError> {
    let json = serde_json::to_string(value).map_err(|error| {
        MailKitError::FrameworkError(format!("failed to encode MailKit payload: {error}"))
    })?;
    to_cstring(&json)
}

pub unsafe fn decode_json_box_ref<T: DeserializeOwned>(
    raw: *mut core::ffi::c_void,
) -> Result<T, MailKitError> {
    if raw.is_null() {
        return Err(MailKitError::FrameworkError(
            "MailKit returned a null JSON box".into(),
        ));
    }
    let json = take_owned_c_string(unsafe { ffi::core::mail_json_box_copy(raw) });
    serde_json::from_str(&json).map_err(|error| {
        MailKitError::FrameworkError(format!("failed to decode MailKit JSON box: {error}"))
    })
}

pub unsafe fn decode_json_box<T: DeserializeOwned>(
    raw: *mut core::ffi::c_void,
) -> Result<T, MailKitError> {
    let value = unsafe { decode_json_box_ref(raw) }?;
    unsafe { ffi::core::mail_object_release(raw) };
    Ok(value)
}

pub fn encode_json_box<T: Serialize>(value: &T) -> Result<*mut core::ffi::c_void, MailKitError> {
    let json = encode_json(value)?;
    let raw = unsafe { ffi::core::mail_json_box_new(json.as_ptr()) };
    if raw.is_null() {
        return Err(MailKitError::FrameworkError(
            "MailKit failed to allocate a JSON box".into(),
        ));
    }
    Ok(raw)
}

/// Writes an owned C-string error message into a MailKit out-parameter.
pub unsafe fn write_error_out(error_out: *mut *mut i8, message: &str) {
    if error_out.is_null() {
        return;
    }
    if let Ok(c_string) = CString::new(message) {
        unsafe {
            *error_out = libc::strdup(c_string.as_ptr());
        }
    }
}

/// Returns pointer, length, and presence markers for an optional byte slice.
pub fn optional_bytes_parts(value: Option<&[u8]>) -> (*const u8, usize, bool) {
    value.map_or((core::ptr::null(), 0, false), |bytes| {
        (bytes.as_ptr(), bytes.len(), true)
    })
}
