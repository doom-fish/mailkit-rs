use core::ffi::{c_char, c_void};

pub type MessageActionDecisionCallback =
    extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_char) -> *mut c_void;
pub type MessageActionHeadersCallback = extern "C" fn(*mut c_void, *mut *mut c_char) -> *mut c_void;
pub type ReleaseUserDataCallback = extern "C" fn(*mut c_void);

// Async callback signature
pub type AsyncMessageActionDecisionCallback = extern "C" fn(*const c_void, *const i8, *mut c_void);

unsafe extern "C" {
    pub fn mail_message_action_extension_new(
        decide_callback: Option<MessageActionDecisionCallback>,
        required_headers_callback: Option<MessageActionHeadersCallback>,
        release_callback: Option<ReleaseUserDataCallback>,
        user_data: *mut c_void,
        out_handler: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_message_action_extension_test_decide_action(
        handler: *mut c_void,
        message_box: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn mail_message_action_extension_test_required_headers(
        handler: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;

    // Async API
    pub fn mail_message_action_decision_async(
        handler: *mut c_void,
        message_box: *mut c_void,
        cb: AsyncMessageActionDecisionCallback,
        ctx: *mut c_void,
    );
}
