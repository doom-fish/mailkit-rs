use core::ffi::{c_char, c_void};

pub type ComposeSessionVoidCallback =
    extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_char) -> i32;
pub type ComposeSessionControllerCallback =
    extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_char) -> *mut c_void;
pub type ComposeSessionJsonCallback =
    extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_char) -> *mut c_void;
pub type ReleaseUserDataCallback = extern "C" fn(*mut c_void);
pub type AsyncComposeSessionCallback = extern "C" fn(*const c_void, *const i8, *mut c_void);

unsafe extern "C" {
    pub fn mail_compose_session_new(
        json: *const c_char,
        out_session: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_compose_session_copy_json(session: *mut c_void) -> *mut c_char;
    pub fn mail_compose_session_reload(session: *mut c_void, error_out: *mut *mut c_char) -> i32;
    pub fn mail_compose_session_reload_count(session: *mut c_void) -> usize;

    pub fn mail_compose_session_handler_new(
        did_begin_callback: Option<ComposeSessionVoidCallback>,
        did_end_callback: Option<ComposeSessionVoidCallback>,
        view_controller_callback: Option<ComposeSessionControllerCallback>,
        annotate_callback: Option<ComposeSessionJsonCallback>,
        can_send_callback: Option<ComposeSessionJsonCallback>,
        additional_headers_callback: Option<ComposeSessionJsonCallback>,
        release_callback: Option<ReleaseUserDataCallback>,
        user_data: *mut c_void,
        out_handler: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_compose_session_handler_test_did_begin(
        handler: *mut c_void,
        session: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_compose_session_handler_test_did_end(
        handler: *mut c_void,
        session: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_compose_session_handler_test_view_controller(
        handler: *mut c_void,
        session: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn mail_compose_session_handler_test_annotate_addresses(
        handler: *mut c_void,
        session: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn mail_compose_session_handler_test_can_send_message(
        handler: *mut c_void,
        session: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn mail_compose_session_handler_test_additional_headers(
        handler: *mut c_void,
        session: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;

    pub fn mail_compose_session_handler_annotate_addresses_async(
        handler: *mut c_void,
        session: *mut c_void,
        cb: AsyncComposeSessionCallback,
        ctx: *mut c_void,
    );
    pub fn mail_compose_session_handler_allow_message_send_async(
        handler: *mut c_void,
        session: *mut c_void,
        cb: AsyncComposeSessionCallback,
        ctx: *mut c_void,
    );
}
