use core::ffi::{c_char, c_void};

pub type ContentBlockerJsonCallback = extern "C" fn(*mut c_void, *mut *mut c_char) -> *mut c_void;
pub type ReleaseUserDataCallback = extern "C" fn(*mut c_void);

unsafe extern "C" {
    pub fn mail_content_blocker_new(
        content_rules_callback: Option<ContentBlockerJsonCallback>,
        release_callback: Option<ReleaseUserDataCallback>,
        user_data: *mut c_void,
        out_blocker: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_content_blocker_test_content_rules_json(
        blocker: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
}
