use core::ffi::{c_char, c_void};

pub type MessageSecurityDecodeCallback =
    extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_char) -> *mut c_void;
pub type MessageSecurityEncodeCallback =
    extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut *mut c_char) -> *mut c_void;
pub type MessageSecurityControllerCallback =
    extern "C" fn(*mut c_void, *mut c_void, *mut *mut c_char) -> *mut c_void;
pub type ReleaseUserDataCallback = extern "C" fn(*mut c_void);

unsafe extern "C" {
    pub fn mail_decoded_message_banner_new(
        title: *const c_char,
        primary_action_title: *const c_char,
        dismissable: bool,
        out_banner: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_message_signer_new(
        addresses: *const *mut c_void,
        address_count: usize,
        label: *const c_char,
        context_bytes: *const u8,
        context_len: usize,
        has_context: bool,
        out_signer: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_message_security_information_new(
        signers: *const *mut c_void,
        signer_count: usize,
        is_encrypted: bool,
        signing_error_json: *const c_char,
        encryption_error_json: *const c_char,
        should_block_remote_content: bool,
        localized_reason: *const c_char,
        out_info: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_decoded_message_new(
        raw_data_bytes: *const u8,
        raw_data_len: usize,
        has_raw_data: bool,
        security_information: *mut c_void,
        context_bytes: *const u8,
        context_len: usize,
        has_context: bool,
        banner: *mut c_void,
        out_message: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_encoded_outgoing_message_new(
        raw_data_bytes: *const u8,
        raw_data_len: usize,
        is_signed: bool,
        is_encrypted: bool,
        out_message: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_outgoing_message_encoding_status_new(
        can_sign: bool,
        can_encrypt: bool,
        security_error_json: *const c_char,
        addresses_failing_encryption: *const *mut c_void,
        failing_address_count: usize,
        out_status: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_message_encoding_result_new(
        encoded_message: *mut c_void,
        signing_error_json: *const c_char,
        encryption_error_json: *const c_char,
        out_result: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;

    pub fn mail_message_security_handler_new(
        decode_callback: Option<MessageSecurityDecodeCallback>,
        encoding_status_callback: Option<MessageSecurityEncodeCallback>,
        encode_message_callback: Option<MessageSecurityEncodeCallback>,
        signers_view_controller_callback: Option<MessageSecurityControllerCallback>,
        message_context_view_controller_callback: Option<MessageSecurityControllerCallback>,
        primary_action_callback: Option<MessageSecurityControllerCallback>,
        release_callback: Option<ReleaseUserDataCallback>,
        user_data: *mut c_void,
        out_handler: *mut *mut c_void,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn mail_message_security_handler_test_decode_message(
        handler: *mut c_void,
        data_box: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn mail_message_security_handler_test_encoding_status(
        handler: *mut c_void,
        message_box: *mut c_void,
        context_box: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn mail_message_security_handler_test_encode_message(
        handler: *mut c_void,
        message_box: *mut c_void,
        context_box: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn mail_message_security_handler_test_view_controller_for_signers(
        handler: *mut c_void,
        signers_box: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn mail_message_security_handler_test_view_controller_for_message_context(
        handler: *mut c_void,
        context_box: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn mail_message_security_handler_test_primary_action_clicked(
        handler: *mut c_void,
        context_box: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
}
