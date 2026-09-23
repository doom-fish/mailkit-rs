use core::ffi::{c_void, CStr};
use std::ffi::CString;

use doom_fish_utils::panic_safe::catch_user_panic;
use serde::{de::DeserializeOwned, Serialize};

use crate::compose_session_handler::{ComposeContextInfo, ComposeSessionInfo};
use crate::error::{take_owned_c_string, MailKitError};
use crate::ffi;
use crate::message::MessageInfo;

pub const RAW_DATA: &CStr = c"rawData";
pub const ORIGINAL_RAW_DATA: &CStr = c"originalMessage.rawData";
pub const SESSION_RAW_DATA: &CStr = c"mailMessage.rawData";
pub const SESSION_ORIGINAL_RAW_DATA: &CStr = c"composeContext.originalMessage.rawData";

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

pub unsafe fn bytes_from_raw<'a>(bytes: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        Some(&[])
    } else if bytes.is_null() {
        None
    } else {
        Some(unsafe { core::slice::from_raw_parts(bytes, len) })
    }
}

unsafe extern "C" fn collect_bytes(context: *mut c_void, bytes: *const u8, len: usize) {
    catch_user_panic("mailkit byte sink", || {
        if context.is_null() {
            return;
        }
        let out = unsafe { &mut *context.cast::<Option<Vec<u8>>>() };
        *out = unsafe { bytes_from_raw(bytes, len) }.map(<[u8]>::to_vec);
    });
}

pub fn read_bytes(copy: impl FnOnce(ffi::core::ByteSink, *mut c_void)) -> Option<Vec<u8>> {
    let mut out: Option<Vec<u8>> = None;
    copy(collect_bytes, (&raw mut out).cast());
    out
}

pub unsafe fn box_payload(raw: *mut c_void, key: &CStr) -> Option<Vec<u8>> {
    read_bytes(|sink, context| unsafe {
        ffi::core::mail_box_copy_payload(raw, key.as_ptr(), sink, context);
    })
}

fn encode_payload_box<T: Serialize>(
    value: &T,
    payloads: &[(&CStr, Option<&[u8]>)],
) -> Result<*mut c_void, MailKitError> {
    let raw = encode_json_box(value)?;
    for (key, bytes) in payloads {
        let Some(bytes) = bytes else {
            continue;
        };
        let attached = unsafe {
            ffi::core::mail_json_box_set_payload(raw, key.as_ptr(), bytes.as_ptr(), bytes.len())
        };
        if !attached {
            unsafe { ffi::core::mail_object_release(raw) };
            return Err(MailKitError::FrameworkError(
                "MailKit failed to attach raw data to a JSON box".into(),
            ));
        }
    }
    Ok(raw)
}

pub fn encode_message_box(info: &MessageInfo) -> Result<*mut c_void, MailKitError> {
    let mut wire = info.clone();
    let raw_data = wire.raw_data.take();
    encode_payload_box(&wire, &[(RAW_DATA, raw_data.as_deref())])
}

pub fn encode_compose_context_box(info: &ComposeContextInfo) -> Result<*mut c_void, MailKitError> {
    let mut wire = info.clone();
    let original = wire
        .original_message
        .as_deref_mut()
        .and_then(|message| message.raw_data.take());
    encode_payload_box(&wire, &[(ORIGINAL_RAW_DATA, original.as_deref())])
}

pub fn encode_compose_session_box(info: &ComposeSessionInfo) -> Result<*mut c_void, MailKitError> {
    let mut wire = info.clone();
    let message = wire.mail_message.raw_data.take();
    let original = wire
        .compose_context
        .original_message
        .as_deref_mut()
        .and_then(|message| message.raw_data.take());
    encode_payload_box(
        &wire,
        &[
            (SESSION_RAW_DATA, message.as_deref()),
            (SESSION_ORIGINAL_RAW_DATA, original.as_deref()),
        ],
    )
}

pub fn encode_message_and_context_boxes(
    message: &MessageInfo,
    compose_context: &ComposeContextInfo,
) -> Result<(*mut c_void, *mut c_void), MailKitError> {
    let message_box = encode_message_box(message)?;
    match encode_compose_context_box(compose_context) {
        Ok(context_box) => Ok((message_box, context_box)),
        Err(error) => {
            unsafe { ffi::core::mail_object_release(message_box) };
            Err(error)
        }
    }
}

pub unsafe fn decode_message_box(raw: *mut c_void) -> Result<MessageInfo, MailKitError> {
    let mut info: MessageInfo = unsafe { decode_json_box_ref(raw) }?;
    info.raw_data = unsafe { box_payload(raw, RAW_DATA) };
    Ok(info)
}

pub unsafe fn decode_compose_context_box(
    raw: *mut c_void,
) -> Result<ComposeContextInfo, MailKitError> {
    let mut info: ComposeContextInfo = unsafe { decode_json_box_ref(raw) }?;
    if let Some(original) = info.original_message.as_deref_mut() {
        original.raw_data = unsafe { box_payload(raw, ORIGINAL_RAW_DATA) };
    }
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_compose_context_box, decode_message_box, encode_compose_context_box,
        encode_message_box,
    };
    use crate::address::EmailAddressInfo;
    use crate::compose_session_handler::{ComposeContextInfo, ComposeUserAction};
    use crate::error::take_owned_c_string;
    use crate::ffi;
    use crate::message::{MessageEncryptionState, MessageInfo, MessageState};

    fn message(raw_data: Option<Vec<u8>>) -> MessageInfo {
        MessageInfo {
            state: MessageState::Sending,
            encryption_state: MessageEncryptionState::Unknown,
            subject: "Payload".into(),
            from_address: EmailAddressInfo {
                raw_string: "alice@example.com".into(),
                address_string: Some("alice@example.com".into()),
            },
            to_addresses: Vec::new(),
            cc_addresses: Vec::new(),
            bcc_addresses: Vec::new(),
            reply_to_addresses: Vec::new(),
            all_recipient_addresses: Vec::new(),
            date_sent: None,
            date_received: None,
            headers: None,
            raw_data,
        }
    }

    fn json_len(raw: *mut core::ffi::c_void) -> usize {
        take_owned_c_string(unsafe { ffi::core::mail_json_box_copy(raw) }).len()
    }

    fn mebibyte() -> Vec<u8> {
        (0..=255u8).cycle().take(1 << 20).collect()
    }

    #[test]
    fn message_box_keeps_raw_data_out_of_the_json() {
        let info = message(Some(mebibyte()));
        let boxed = encode_message_box(&info).expect("message box");
        assert!(json_len(boxed) < 1024);
        let decoded = unsafe { decode_message_box(boxed) }.expect("decoded message");
        unsafe { ffi::core::mail_object_release(boxed) };
        assert_eq!(decoded, info);

        let empty = message(Some(Vec::new()));
        let boxed = encode_message_box(&empty).expect("message box");
        let decoded = unsafe { decode_message_box(boxed) }.expect("decoded message");
        unsafe { ffi::core::mail_object_release(boxed) };
        assert_eq!(decoded.raw_data, Some(Vec::new()));
    }

    #[test]
    fn compose_context_box_keeps_original_raw_data_out_of_the_json() {
        let info = ComposeContextInfo {
            context_id: "00000000-0000-0000-0000-000000000003".into(),
            original_message: Some(Box::new(message(Some(mebibyte())))),
            action: ComposeUserAction::Forward,
            is_encrypted: false,
            should_encrypt: true,
            is_signed: false,
            should_sign: false,
        };
        let boxed = encode_compose_context_box(&info).expect("context box");
        assert!(json_len(boxed) < 1024);
        let decoded = unsafe { decode_compose_context_box(boxed) }.expect("decoded context");
        unsafe { ffi::core::mail_object_release(boxed) };
        assert_eq!(decoded, info);
    }
}
