use core::ffi::c_char;

unsafe extern "C" {
    pub fn mail_extension_manager_reload_visible_messages(
        timeout_millis: u64,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_extension_manager_reload_content_blocker(
        identifier: *const c_char,
        timeout_millis: u64,
        error_out: *mut *mut c_char,
    ) -> i32;
}
