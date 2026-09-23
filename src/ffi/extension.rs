use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn mail_extension_new(
        compose_session_handler: *mut c_void,
        message_action_handler: *mut c_void,
        content_blocker: *mut c_void,
        message_security_handler: *mut c_void,
        out_extension: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_extension_register_principal(extension_ptr: *mut c_void) -> bool;
    pub fn mail_extension_test_new_principal() -> *mut c_void;
    pub fn mail_extension_test_has_compose_session_handler(extension_ptr: *mut c_void) -> bool;
    pub fn mail_extension_test_has_message_action_handler(extension_ptr: *mut c_void) -> bool;
    pub fn mail_extension_test_has_content_blocker(extension_ptr: *mut c_void) -> bool;
    pub fn mail_extension_test_has_message_security_handler(extension_ptr: *mut c_void) -> bool;
}
