use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn mail_object_release(ptr: *mut c_void);
    pub fn mail_json_box_new(json: *const c_char) -> *mut c_void;
    pub fn mail_json_box_copy(box_ptr: *mut c_void) -> *mut c_char;

    pub fn mail_message_action_decision_box_new(
        decision: *mut c_void,
        json: *const c_char,
    ) -> *mut c_void;
    pub fn mail_message_action_decision_box_copy_json(box_ptr: *mut c_void) -> *mut c_char;
    pub fn mail_message_action_decision_box_copy_value(box_ptr: *mut c_void) -> *mut c_void;

    pub fn mail_decoded_message_box_new(message: *mut c_void, json: *const c_char) -> *mut c_void;
    pub fn mail_decoded_message_box_copy_json(box_ptr: *mut c_void) -> *mut c_char;
    pub fn mail_decoded_message_box_copy_value(box_ptr: *mut c_void) -> *mut c_void;

    pub fn mail_outgoing_message_encoding_status_box_new(
        status: *mut c_void,
        json: *const c_char,
    ) -> *mut c_void;
    pub fn mail_outgoing_message_encoding_status_box_copy_json(box_ptr: *mut c_void)
        -> *mut c_char;
    pub fn mail_outgoing_message_encoding_status_box_copy_value(
        box_ptr: *mut c_void,
    ) -> *mut c_void;

    pub fn mail_message_encoding_result_box_new(
        result: *mut c_void,
        json: *const c_char,
    ) -> *mut c_void;
    pub fn mail_message_encoding_result_box_copy_json(box_ptr: *mut c_void) -> *mut c_char;
    pub fn mail_message_encoding_result_box_copy_value(box_ptr: *mut c_void) -> *mut c_void;

    pub fn mail_compose_session_error_domain_copy() -> *mut c_char;
    pub fn mail_message_security_error_domain_copy() -> *mut c_char;
    pub fn mail_extension_view_controller_new() -> *mut c_void;
}
