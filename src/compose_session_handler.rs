use core::ffi::{c_char, c_void};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::address::{AddressAnnotation, AddressAnnotationKind};
use crate::error::{from_swift, take_owned_c_string, MailKitError, NSErrorInfo};
use crate::extension_view_controller::ExtensionViewController;
use crate::ffi;
use crate::message::{Message, MessageInfo};
use crate::private::{encode_json, encode_json_box, write_error_out};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents the MailKit compose action that produced a `MEComposeContext`.
pub enum ComposeUserAction {
    /// Represents composing a brand-new message.
    NewMessage,
    /// Represents replying to a message.
    Reply,
    /// Represents replying to all recipients.
    ReplyAll,
    /// Represents forwarding a message.
    Forward,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
/// Captures a Rust snapshot of `MEComposeContext`.
pub struct ComposeContextInfo {
    /// Stores the MailKit compose-context identifier.
    pub context_id: String,
    /// Stores the original message when the compose session started from an existing message.
    pub original_message: Option<Box<MessageInfo>>,
    /// Stores the user action that created the compose context.
    pub action: ComposeUserAction,
    /// Indicates whether the original message is encrypted.
    pub is_encrypted: bool,
    /// Indicates whether MailKit currently wants outgoing content encrypted.
    pub should_encrypt: bool,
    /// Indicates whether the original message is signed.
    pub is_signed: bool,
    /// Indicates whether MailKit currently wants outgoing content signed.
    pub should_sign: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `MEComposeContext`.
pub struct ComposeContext {
    info: ComposeContextInfo,
}

/// Alias matching MailKit's `MEComposeContext` name.
pub type MEComposeContext = ComposeContext;

impl ComposeContext {
    #[must_use]
    /// Creates a `MEComposeContext` snapshot from Rust data.
    pub const fn new(info: ComposeContextInfo) -> Self {
        Self { info }
    }

    #[must_use]
    /// Returns the cached compose-context snapshot.
    pub const fn info(&self) -> &ComposeContextInfo {
        &self.info
    }

    #[must_use]
    /// Returns the original message that seeded the compose context, when present.
    pub fn original_message(&self) -> Option<Message> {
        self.info
            .original_message
            .as_deref()
            .cloned()
            .map(Message::new)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEComposeSession`.
pub struct ComposeSessionInfo {
    /// Stores the MailKit compose-session identifier.
    pub session_id: String,
    /// Stores the editable message snapshot for the session.
    pub mail_message: MessageInfo,
    /// Stores the compose context associated with the session.
    pub compose_context: ComposeContextInfo,
}

/// Wraps `MEComposeSession`.
pub struct ComposeSession {
    raw: *mut c_void,
    info: ComposeSessionInfo,
    owned: bool,
}

/// Alias matching MailKit's `MEComposeSession` name.
pub type MEComposeSession = ComposeSession;

impl ComposeSession {
    /// Creates a new `MEComposeSession` wrapper from Rust data.
    pub fn new(info: ComposeSessionInfo) -> Result<Self, MailKitError> {
        let raw = Self::new_raw_from_info(&info)?;
        Ok(Self {
            raw,
            info,
            owned: true,
        })
    }

    fn new_raw_from_info(info: &ComposeSessionInfo) -> Result<*mut c_void, MailKitError> {
        let json = encode_json(info)?;
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::compose_session_handler::mail_compose_session_new(
                json.as_ptr(),
                &mut raw,
                &mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(raw)
    }

    unsafe fn from_callback_raw(raw: *mut c_void) -> Result<Self, MailKitError> {
        if raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit returned a null compose-session box".into(),
            ));
        }
        let json = take_owned_c_string(unsafe {
            ffi::compose_session_handler::mail_compose_session_copy_json(raw)
        });
        let info = serde_json::from_str::<ComposeSessionInfo>(&json).map_err(|error| {
            MailKitError::FrameworkError(format!(
                "failed to decode MailKit compose session: {error}"
            ))
        })?;
        Ok(Self {
            raw,
            info,
            owned: false,
        })
    }

    #[must_use]
    /// Returns the cached compose-session snapshot.
    pub const fn info(&self) -> &ComposeSessionInfo {
        &self.info
    }

    #[must_use]
    /// Returns the message snapshot associated with the session.
    pub fn mail_message(&self) -> Message {
        Message::new(self.info.mail_message.clone())
    }

    #[must_use]
    /// Returns the compose context associated with the session.
    pub fn compose_context(&self) -> ComposeContext {
        ComposeContext::new(self.info.compose_context.clone())
    }

    /// Asks MailKit to reload the compose session.
    pub fn reload_session(&self) -> Result<(), MailKitError> {
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::compose_session_handler::mail_compose_session_reload(self.raw, &raw mut error_ptr)
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(())
    }

    #[must_use]
    /// Returns how many times the test bridge has reloaded the session.
    pub fn reload_count(&self) -> usize {
        unsafe { ffi::compose_session_handler::mail_compose_session_reload_count(self.raw) }
    }
}

impl Drop for ComposeSession {
    fn drop(&mut self) {
        if self.owned && !self.raw.is_null() {
            unsafe { ffi::core::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents MailKit compose-session validation errors.
pub enum ComposeSessionErrorCode {
    /// Indicates that one or more recipients are invalid.
    InvalidRecipients = 0,
    /// Indicates that additional headers are invalid.
    InvalidHeaders = 1,
    /// Indicates that the message body is invalid.
    InvalidBody = 2,
}

impl ComposeSessionErrorCode {
    #[must_use]
    /// Returns the MailKit integer constant for this compose-session error.
    pub const fn as_code(self) -> i64 {
        self as i64
    }
}

/// Returns the `NSError` domain used by MailKit compose-session validation failures.
pub fn compose_session_error_domain() -> String {
    take_owned_c_string(unsafe { ffi::core::mail_compose_session_error_domain_copy() })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Associates a composed recipient string with a MailKit address annotation.
pub struct AddressAnnotationMatch {
    /// Stores the recipient string that MailKit asked to annotate.
    pub raw_string: String,
    /// Stores the annotation severity to display.
    pub kind: AddressAnnotationKind,
    /// Stores the localized annotation message to display.
    pub localized_description: String,
}

impl AddressAnnotationMatch {
    #[must_use]
    /// Creates a recipient-to-annotation mapping for MailKit compose validation.
    pub fn new(raw_string: impl Into<String>, annotation: &AddressAnnotation) -> Self {
        Self {
            raw_string: raw_string.into(),
            kind: annotation.kind(),
            localized_description: annotation.localized_description().to_string(),
        }
    }
}

/// Implements the callbacks MailKit expects from `MEComposeSessionHandler`.
pub trait ComposeSessionHandlerDelegate: Send + Sync + 'static {
    /// Handles MailKit's compose-session-begin callback.
    fn mail_compose_session_did_begin(&self, session: &ComposeSession) -> Result<(), MailKitError>;
    /// Handles MailKit's compose-session-end callback.
    fn mail_compose_session_did_end(&self, session: &ComposeSession) -> Result<(), MailKitError>;
    /// Returns the extension view controller for a compose session.
    fn view_controller_for_session(
        &self,
        session: &ComposeSession,
    ) -> Result<ExtensionViewController, MailKitError>;

    /// Returns address annotations for recipients in the compose session.
    fn annotate_addresses_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<Vec<AddressAnnotationMatch>, MailKitError> {
        Ok(Vec::new())
    }

    /// Decides whether MailKit may send the composed message.
    fn allow_message_send_for_session(&self, _session: &ComposeSession) -> Result<(), NSErrorInfo> {
        Ok(())
    }

    /// Returns extra headers that MailKit should inject before sending.
    fn additional_headers_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<BTreeMap<String, Vec<String>>, MailKitError> {
        Ok(BTreeMap::new())
    }
}

struct ComposeSessionHandlerState {
    handler: Box<dyn ComposeSessionHandlerDelegate>,
}

/// Wraps `MEComposeSessionHandler`.
pub struct ComposeSessionHandler {
    raw: *mut c_void,
}

impl ComposeSessionHandler {
    #[must_use]
    /// Returns the retained native MailKit object pointer.
    pub const fn raw(&self) -> *mut c_void {
        self.raw
    }

    /// Creates a `MEComposeSessionHandler` wrapper from a Rust delegate.
    pub fn new<H: ComposeSessionHandlerDelegate>(handler: H) -> Result<Self, MailKitError> {
        let state = Box::new(ComposeSessionHandlerState {
            handler: Box::new(handler),
        });
        let user_data = Box::into_raw(state).cast::<c_void>();
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::compose_session_handler::mail_compose_session_handler_new(
                Some(did_begin_callback),
                Some(did_end_callback),
                Some(view_controller_callback),
                Some(annotate_addresses_callback),
                Some(can_send_callback),
                Some(additional_headers_callback),
                Some(release_user_data),
                user_data,
                &raw mut raw,
                &raw mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            unsafe {
                drop(Box::from_raw(
                    user_data.cast::<ComposeSessionHandlerState>(),
                ));
            }
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self { raw })
    }

    /// Invokes the wrapped begin-session callback for a compose session.
    pub fn mail_compose_session_did_begin(
        &self,
        session: &ComposeSession,
    ) -> Result<(), MailKitError> {
        self.invoke_void(
            ffi::compose_session_handler::mail_compose_session_handler_test_did_begin,
            session,
        )
    }

    /// Invokes the wrapped end-session callback for a compose session.
    pub fn mail_compose_session_did_end(
        &self,
        session: &ComposeSession,
    ) -> Result<(), MailKitError> {
        self.invoke_void(
            ffi::compose_session_handler::mail_compose_session_handler_test_did_end,
            session,
        )
    }

    /// Returns the extension view controller for the supplied compose session.
    pub fn view_controller_for_session(
        &self,
        session: &ComposeSession,
    ) -> Result<ExtensionViewController, MailKitError> {
        let session_raw = ComposeSession::new_raw_from_info(session.info())?;
        let mut error_ptr = ptr::null_mut();
        let raw = unsafe {
            ffi::compose_session_handler::mail_compose_session_handler_test_view_controller(
                self.raw,
                session_raw,
                &raw mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        ExtensionViewController::from_raw(raw)
    }

    /// Returns address annotations for the supplied compose session.
    pub fn annotate_addresses_for_session(
        &self,
        session: &ComposeSession,
    ) -> Result<Vec<AddressAnnotationMatch>, MailKitError> {
        let session_raw = ComposeSession::new_raw_from_info(session.info())?;
        let mut error_ptr = ptr::null_mut();
        let raw = unsafe {
            ffi::compose_session_handler::mail_compose_session_handler_test_annotate_addresses(
                self.raw,
                session_raw,
                &raw mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        if raw.is_null() {
            return Ok(Vec::new());
        }
        unsafe { crate::private::decode_json_box(raw) }
    }

    /// Asks the wrapped delegate whether MailKit should send the message.
    pub fn allow_message_send_for_session(
        &self,
        session: &ComposeSession,
    ) -> Result<(), MailKitError> {
        let session_raw = ComposeSession::new_raw_from_info(session.info())?;
        let mut error_ptr = ptr::null_mut();
        let raw = unsafe {
            ffi::compose_session_handler::mail_compose_session_handler_test_can_send_message(
                self.raw,
                session_raw,
                &raw mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        if raw.is_null() {
            return Ok(());
        }
        let error: NSErrorInfo = unsafe { crate::private::decode_json_box(raw) }?;
        Err(MailKitError::OperationFailed(Box::new(error)))
    }

    /// Returns additional headers for the supplied compose session.
    pub fn additional_headers_for_session(
        &self,
        session: &ComposeSession,
    ) -> Result<BTreeMap<String, Vec<String>>, MailKitError> {
        let session_raw = ComposeSession::new_raw_from_info(session.info())?;
        let mut error_ptr = ptr::null_mut();
        let raw = unsafe {
            ffi::compose_session_handler::mail_compose_session_handler_test_additional_headers(
                self.raw,
                session_raw,
                &raw mut error_ptr,
            )
        };
        if !error_ptr.is_null() {
            return Err(from_swift(ffi::status::FRAMEWORK_ERROR, error_ptr));
        }
        if raw.is_null() {
            return Ok(BTreeMap::new());
        }
        unsafe { crate::private::decode_json_box(raw) }
    }

    fn invoke_void(
        &self,
        func: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_char) -> i32,
        session: &ComposeSession,
    ) -> Result<(), MailKitError> {
        let session_raw = ComposeSession::new_raw_from_info(session.info())?;
        let mut error_ptr = ptr::null_mut();
        let status = unsafe { func(self.raw, session_raw, &raw mut error_ptr) };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(())
    }
}

impl Drop for ComposeSessionHandler {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::core::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AddressAnnotationEntry {
    raw_string: String,
    kind: &'static str,
    localized_description: String,
}

impl From<AddressAnnotationMatch> for AddressAnnotationEntry {
    fn from(value: AddressAnnotationMatch) -> Self {
        let kind = match value.kind {
            AddressAnnotationKind::Error => "error",
            AddressAnnotationKind::Warning => "warning",
            AddressAnnotationKind::Success => "success",
        };
        Self {
            raw_string: value.raw_string,
            kind,
            localized_description: value.localized_description,
        }
    }
}

extern "C" fn did_begin_callback(
    session_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> i32 {
    invoke_session_void(session_box, user_data, error_out, |handler, session| {
        handler.mail_compose_session_did_begin(session)
    })
}

extern "C" fn did_end_callback(
    session_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> i32 {
    invoke_session_void(session_box, user_data, error_out, |handler, session| {
        handler.mail_compose_session_did_end(session)
    })
}

extern "C" fn view_controller_callback(
    session_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    invoke_session_pointer(session_box, user_data, error_out, |handler, session| {
        handler
            .view_controller_for_session(session)
            .map(ExtensionViewController::into_raw)
    })
}

extern "C" fn annotate_addresses_callback(
    session_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    invoke_session_pointer(session_box, user_data, error_out, |handler, session| {
        let entries: Vec<AddressAnnotationEntry> = handler
            .annotate_addresses_for_session(session)?
            .into_iter()
            .map(AddressAnnotationEntry::from)
            .collect();
        encode_json_box(&entries)
    })
}

extern "C" fn can_send_callback(
    session_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    invoke_session_pointer(
        session_box,
        user_data,
        error_out,
        |handler, session| match handler.allow_message_send_for_session(session) {
            Ok(()) => Ok(ptr::null_mut()),
            Err(error) => encode_json_box(&error),
        },
    )
}

extern "C" fn additional_headers_callback(
    session_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
) -> *mut c_void {
    invoke_session_pointer(session_box, user_data, error_out, |handler, session| {
        let headers = handler.additional_headers_for_session(session)?;
        encode_json_box(&headers)
    })
}

fn invoke_session_void<F>(
    session_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
    callback: F,
) -> i32
where
    F: FnOnce(&dyn ComposeSessionHandlerDelegate, &ComposeSession) -> Result<(), MailKitError>,
{
    match catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_data.cast::<ComposeSessionHandlerState>() };
        let session = unsafe { ComposeSession::from_callback_raw(session_box) }?;
        callback(state.handler.as_ref(), &session)
    })) {
        Ok(Ok(())) => ffi::status::OK,
        Ok(Err(error)) => {
            unsafe { write_error_out(error_out, &error.to_string()) };
            ffi::status::FRAMEWORK_ERROR
        }
        Err(_) => {
            unsafe { write_error_out(error_out, "compose-session callback panicked") };
            ffi::status::FRAMEWORK_ERROR
        }
    }
}

fn invoke_session_pointer<F>(
    session_box: *mut c_void,
    user_data: *mut c_void,
    error_out: *mut *mut c_char,
    callback: F,
) -> *mut c_void
where
    F: FnOnce(
        &dyn ComposeSessionHandlerDelegate,
        &ComposeSession,
    ) -> Result<*mut c_void, MailKitError>,
{
    match catch_unwind(AssertUnwindSafe(|| {
        let state = unsafe { &*user_data.cast::<ComposeSessionHandlerState>() };
        let session = unsafe { ComposeSession::from_callback_raw(session_box) }?;
        callback(state.handler.as_ref(), &session)
    })) {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            unsafe { write_error_out(error_out, &error.to_string()) };
            ptr::null_mut()
        }
        Err(_) => {
            unsafe { write_error_out(error_out, "compose-session callback panicked") };
            ptr::null_mut()
        }
    }
}

extern "C" fn release_user_data(user_data: *mut c_void) {
    if !user_data.is_null() {
        unsafe {
            drop(Box::from_raw(
                user_data.cast::<ComposeSessionHandlerState>(),
            ));
        }
    }
}
