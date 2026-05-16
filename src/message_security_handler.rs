use core::ffi::{c_char, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use crate::compose_session_handler::{ComposeContext, ComposeContextInfo};
use crate::error::{from_swift, take_owned_c_string, MailKitError};
use crate::extension_view_controller::ExtensionViewController;
use crate::ffi;
use crate::message::{Message, MessageInfo};
use crate::private::{decode_json_box_ref, encode_json_box, write_error_out};
use crate::security::{
    DecodedMessage, MessageEncodingResult, MessageSignerInfo, OutgoingMessageEncodingStatus,
};

pub fn message_security_error_domain() -> String {
    take_owned_c_string(unsafe { ffi::core::mail_message_security_error_domain_copy() })
}

pub trait MessageSecurityHandlerDelegate: Send + Sync + 'static {
    fn decoded_message_for_message_data(
        &self,
        data: &[u8],
    ) -> Result<Option<DecodedMessage>, MailKitError>;

    fn encoding_status_for_message(
        &self,
        message: &Message,
        compose_context: &ComposeContext,
    ) -> Result<OutgoingMessageEncodingStatus, MailKitError>;

    fn encode_message(
        &self,
        message: &Message,
        compose_context: &ComposeContext,
    ) -> Result<MessageEncodingResult, MailKitError>;

    fn extension_view_controller_for_signers(
        &self,
        _message_signers: &[MessageSignerInfo],
    ) -> Result<Option<ExtensionViewController>, MailKitError> {
        Ok(None)
    }

    fn extension_view_controller_for_message_context(
        &self,
        _context: &[u8],
    ) -> Result<Option<ExtensionViewController>, MailKitError> {
        Ok(None)
    }

    fn primary_action_clicked_for_message_context(
        &self,
        _context: &[u8],
    ) -> Result<Option<ExtensionViewController>, MailKitError> {
        Ok(None)
    }
}

struct MessageSecurityHandlerState {
    handler: Box<dyn MessageSecurityHandlerDelegate>,
}

pub struct MessageSecurityHandler {
    raw: *mut c_void,
}

impl MessageSecurityHandler {
    #[must_use]
    pub const fn raw(&self) -> *mut c_void {
        self.raw
    }

    pub fn new<H: MessageSecurityHandlerDelegate>(handler: H) -> Result<Self, MailKitError> {
        let state = Box::new(MessageSecurityHandlerState {
            handler: Box::new(handler),
        });
        let user_data = Box::into_raw(state).cast::<c_void>();
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::security::mail_message_security_handler_new(
                Some(decode_callback),
                Some(encoding_status_callback),
                Some(encode_message_callback),
                Some(signers_view_controller_callback),
                Some(message_context_view_controller_callback),
                Some(primary_action_callback),
                Some(release_user_data),
                user_data,
                &mut raw,
                &mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            unsafe {
                drop(Box::from_raw(
                    user_data.cast::<MessageSecurityHandlerState>(),
                ));
            }
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self { raw })
    }

    pub fn decoded_message_for_message_data(
        &self,
        data: &[u8],
    ) -> Result<Option<DecodedMessage>, MailKitError> {
        let data_box = encode_json_box(&data)?;
        let mut error_ptr = ptr::null_mut();
        let result = unsafe {
            ffi::security::mail_message_security_handler_test_decode_message(
                self.raw,
                data_box,
                &mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        if result.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe { DecodedMessage::from_box_raw(result) }?))
    }

    pub fn encoding_status_for_message(
        &self,
        message: &Message,
        compose_context: &ComposeContext,
    ) -> Result<OutgoingMessageEncodingStatus, MailKitError> {
        let message_box = encode_json_box(message.info())?;
        let context_box = encode_json_box(compose_context.info())?;
        let mut error_ptr = ptr::null_mut();
        let result = unsafe {
            ffi::security::mail_message_security_handler_test_encoding_status(
                self.raw,
                message_box,
                context_box,
                &mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        unsafe { OutgoingMessageEncodingStatus::from_box_raw(result) }
    }

    pub fn encode_message(
        &self,
        message: &Message,
        compose_context: &ComposeContext,
    ) -> Result<MessageEncodingResult, MailKitError> {
        let message_box = encode_json_box(message.info())?;
        let context_box = encode_json_box(compose_context.info())?;
        let mut error_ptr = ptr::null_mut();
        let result = unsafe {
            ffi::security::mail_message_security_handler_test_encode_message(
                self.raw,
                message_box,
                context_box,
                &mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        unsafe { MessageEncodingResult::from_box_raw(result) }
    }

    pub fn extension_view_controller_for_signers(
        &self,
        message_signers: &[MessageSignerInfo],
    ) -> Result<Option<ExtensionViewController>, MailKitError> {
        let signers_box = encode_json_box(&message_signers)?;
        let mut error_ptr = ptr::null_mut();
        let result = unsafe {
            ffi::security::mail_message_security_handler_test_view_controller_for_signers(
                self.raw,
                signers_box,
                &mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        if result.is_null() {
            return Ok(None);
        }
        Ok(Some(ExtensionViewController::from_raw(result)?))
    }

    pub fn extension_view_controller_for_message_context(
        &self,
        context: &[u8],
    ) -> Result<Option<ExtensionViewController>, MailKitError> {
        let context_box = encode_json_box(&context)?;
        let mut error_ptr = ptr::null_mut();
        let result = unsafe {
            ffi::security::mail_message_security_handler_test_view_controller_for_message_context(
                self.raw,
                context_box,
                &mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        if result.is_null() {
            return Ok(None);
        }
        Ok(Some(ExtensionViewController::from_raw(result)?))
    }

    pub fn primary_action_clicked_for_message_context(
        &self,
        context: &[u8],
    ) -> Result<Option<ExtensionViewController>, MailKitError> {
        let context_box = encode_json_box(&context)?;
        let mut error_ptr = ptr::null_mut();
        let result = unsafe {
            ffi::security::mail_message_security_handler_test_primary_action_clicked(
                self.raw,
                context_box,
                &mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        if result.is_null() {
            return Ok(None);
        }
        Ok(Some(ExtensionViewController::from_raw(result)?))
    }
}

impl Drop for MessageSecurityHandler {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::core::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

extern "C" fn decode_callback(
    data_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    match catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_data.cast::<MessageSecurityHandlerState>() };
        let data = unsafe { decode_json_box_ref::<Vec<u8>>(data_box) }?;
        state
            .handler
            .decoded_message_for_message_data(&data)?
            .map(DecodedMessage::into_box_raw)
            .transpose()
    })) {
        Ok(Ok(Some(result))) => result,
        Ok(Ok(None)) => ptr::null_mut(),
        Ok(Err(error)) => {
            unsafe { write_error_out(error_out, &error.to_string()) };
            ptr::null_mut()
        }
        Err(_) => {
            unsafe { write_error_out(error_out, "message-security decode callback panicked") };
            ptr::null_mut()
        }
    }
}

extern "C" fn encoding_status_callback(
    message_box: *mut c_void,
    context_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    encode_with_context(
        message_box,
        context_box,
        user_data,
        error_out,
        |handler, message, context| {
            handler
                .encoding_status_for_message(message, context)
                .and_then(OutgoingMessageEncodingStatus::into_box_raw)
        },
    )
}

extern "C" fn encode_message_callback(
    message_box: *mut c_void,
    context_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    encode_with_context(
        message_box,
        context_box,
        user_data,
        error_out,
        |handler, message, context| {
            handler
                .encode_message(message, context)
                .and_then(MessageEncodingResult::into_box_raw)
        },
    )
}

extern "C" fn signers_view_controller_callback(
    signers_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    controller_from_json(
        signers_box,
        user_data,
        error_out,
        |handler, signers: Vec<MessageSignerInfo>| {
            handler
                .extension_view_controller_for_signers(&signers)
                .map(|controller| {
                    controller.map_or(ptr::null_mut(), ExtensionViewController::into_raw)
                })
        },
    )
}

extern "C" fn message_context_view_controller_callback(
    context_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    controller_from_json(
        context_box,
        user_data,
        error_out,
        |handler, context: Vec<u8>| {
            handler
                .extension_view_controller_for_message_context(&context)
                .map(|controller| {
                    controller.map_or(ptr::null_mut(), ExtensionViewController::into_raw)
                })
        },
    )
}

extern "C" fn primary_action_callback(
    context_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    controller_from_json(
        context_box,
        user_data,
        error_out,
        |handler, context: Vec<u8>| {
            handler
                .primary_action_clicked_for_message_context(&context)
                .map(|controller| {
                    controller.map_or(ptr::null_mut(), ExtensionViewController::into_raw)
                })
        },
    )
}

fn encode_with_context<F>(
    message_box: *mut c_void,
    context_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
    callback: F,
) -> *mut c_void
where
    F: FnOnce(
        &dyn MessageSecurityHandlerDelegate,
        &Message,
        &ComposeContext,
    ) -> Result<*mut c_void, MailKitError>,
{
    match catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_data.cast::<MessageSecurityHandlerState>() };
        let message_info = unsafe { decode_json_box_ref::<MessageInfo>(message_box) }?;
        let context_info = unsafe { decode_json_box_ref::<ComposeContextInfo>(context_box) }?;
        let message = Message::new(message_info);
        let context = ComposeContext::new(context_info);
        callback(state.handler.as_ref(), &message, &context)
    })) {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            unsafe { write_error_out(error_out, &error.to_string()) };
            ptr::null_mut()
        }
        Err(_) => {
            unsafe { write_error_out(error_out, "message-security callback panicked") };
            ptr::null_mut()
        }
    }
}

fn controller_from_json<T, F>(
    json_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
    callback: F,
) -> *mut c_void
where
    T: for<'de> serde::Deserialize<'de>,
    F: FnOnce(&dyn MessageSecurityHandlerDelegate, T) -> Result<*mut c_void, MailKitError>,
{
    match catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_data.cast::<MessageSecurityHandlerState>() };
        let payload = unsafe { decode_json_box_ref::<T>(json_box) }?;
        callback(state.handler.as_ref(), payload)
    })) {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            unsafe { write_error_out(error_out, &error.to_string()) };
            ptr::null_mut()
        }
        Err(_) => {
            unsafe { write_error_out(error_out, "message-security controller callback panicked") };
            ptr::null_mut()
        }
    }
}

extern "C" fn release_user_data(user_data: *mut c_void) {
    if !user_data.is_null() {
        unsafe {
            drop(Box::from_raw(
                user_data.cast::<MessageSecurityHandlerState>(),
            ));
        }
    }
}
