use core::ffi::c_void;

unsafe extern "C" {
    pub fn mail_message_action_move_to_trash() -> *mut c_void;
    pub fn mail_message_action_move_to_archive() -> *mut c_void;
    pub fn mail_message_action_move_to_junk() -> *mut c_void;
    pub fn mail_message_action_mark_as_read() -> *mut c_void;
    pub fn mail_message_action_mark_as_unread() -> *mut c_void;
    pub fn mail_message_action_flag(flag: i32) -> *mut c_void;
    pub fn mail_message_action_set_background_color(color: i32) -> *mut c_void;

    pub fn mail_message_action_decision_invoke_again_with_body() -> *mut c_void;
    pub fn mail_message_action_decision_action(action: *mut c_void) -> *mut c_void;
    pub fn mail_message_action_decision_actions(
        actions: *const *mut c_void,
        action_count: usize,
    ) -> *mut c_void;
}
