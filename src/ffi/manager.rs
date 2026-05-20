use core::ffi::{c_char, c_void};

pub type AsyncExtensionManagerCallback = extern "C" fn(*const c_void, *const i8, *mut c_void);

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
    pub fn mail_extension_manager_reload_visible_messages_async(
        cb: AsyncExtensionManagerCallback,
        ctx: *mut c_void,
    );
    pub fn mail_extension_manager_reload_content_blocker_async(
        identifier: *const c_char,
        cb: AsyncExtensionManagerCallback,
        ctx: *mut c_void,
    );
}
