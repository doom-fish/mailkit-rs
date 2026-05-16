use core::ffi::{c_char, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use crate::action::MessageActionDecision;
use crate::error::{from_swift, MailKitError};
use crate::ffi;
use crate::message::{Message, MessageInfo};
use crate::private::{decode_json_box_ref, encode_json_box, write_error_out};

pub trait MessageActionHandler: Send + Sync + 'static {
    fn decide_action_for_message(
        &self,
        message: &Message,
    ) -> Result<Option<MessageActionDecision>, MailKitError>;

    fn required_headers(&self) -> Result<Vec<String>, MailKitError> {
        Ok(Vec::new())
    }
}

struct MessageActionHandlerState {
    handler: Box<dyn MessageActionHandler>,
}

pub struct MessageActionExtension {
    raw: *mut c_void,
}

impl MessageActionExtension {
    #[must_use]
    pub const fn raw(&self) -> *mut c_void {
        self.raw
    }

    pub fn new<H: MessageActionHandler>(handler: H) -> Result<Self, MailKitError> {
        let state = Box::new(MessageActionHandlerState {
            handler: Box::new(handler),
        });
        let user_data = Box::into_raw(state).cast::<c_void>();
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::message_action_extension::mail_message_action_extension_new(
                Some(decide_callback),
                Some(required_headers_callback),
                Some(release_user_data),
                user_data,
                &mut raw,
                &mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            unsafe {
                drop(Box::from_raw(user_data.cast::<MessageActionHandlerState>()));
            }
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self { raw })
    }

    pub fn decide_action_for_message(
        &self,
        message: &Message,
    ) -> Result<Option<MessageActionDecision>, MailKitError> {
        let message_box = encode_json_box(message.info())?;
        let mut error_ptr = ptr::null_mut();
        let result = unsafe {
            ffi::message_action_extension::mail_message_action_extension_test_decide_action(
                self.raw,
                message_box,
                &mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        if result.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe {
            MessageActionDecision::from_box_raw(result)
        }?))
    }

    pub fn required_headers(&self) -> Result<Vec<String>, MailKitError> {
        let mut error_ptr = ptr::null_mut();
        let result = unsafe {
            ffi::message_action_extension::mail_message_action_extension_test_required_headers(
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
        unsafe { crate::private::decode_json_box(result) }
    }
}

impl Drop for MessageActionExtension {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::core::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

extern "C" fn decide_callback(
    message_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    match catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_data.cast::<MessageActionHandlerState>() };
        let info = unsafe { decode_json_box_ref::<MessageInfo>(message_box) }?;
        let message = Message::new(info);
        state
            .handler
            .decide_action_for_message(&message)?
            .map(MessageActionDecision::into_box_raw)
            .transpose()
    })) {
        Ok(Ok(Some(result))) => result,
        Ok(Ok(None)) => ptr::null_mut(),
        Ok(Err(error)) => {
            unsafe { write_error_out(error_out, &error.to_string()) };
            ptr::null_mut()
        }
        Err(_) => {
            unsafe { write_error_out(error_out, "message-action callback panicked") };
            ptr::null_mut()
        }
    }
}

extern "C" fn required_headers_callback(
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    match catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_data.cast::<MessageActionHandlerState>() };
        let headers = state.handler.required_headers()?;
        encode_json_box(&headers)
    })) {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            unsafe { write_error_out(error_out, &error.to_string()) };
            ptr::null_mut()
        }
        Err(_) => {
            unsafe { write_error_out(error_out, "message-action callback panicked") };
            ptr::null_mut()
        }
    }
}

extern "C" fn release_user_data(user_data: *mut c_void) {
    if !user_data.is_null() {
        unsafe {
            drop(Box::from_raw(user_data.cast::<MessageActionHandlerState>()));
        }
    }
}
