use core::ffi::{c_char, c_void};

unsafe extern "C" {
    pub fn mail_email_address_new(
        raw_string: *const c_char,
        out_address: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_email_address_raw_string(address: *mut c_void) -> *mut c_char;
    pub fn mail_email_address_address_string(address: *mut c_void) -> *mut c_char;

    pub fn mail_address_annotation_error(localized_description: *const c_char) -> *mut c_void;
    pub fn mail_address_annotation_warning(localized_description: *const c_char) -> *mut c_void;
    pub fn mail_address_annotation_success(localized_description: *const c_char) -> *mut c_void;
}
