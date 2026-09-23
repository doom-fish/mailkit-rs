#ifndef MAILKIT_BRIDGE_H
#define MAILKIT_BRIDGE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef int32_t (*mail_compose_session_void_callback)(void *session_box, void *user_data, char **error_out);
typedef void *(*mail_compose_session_controller_callback)(void *session_box, void *user_data, char **error_out);
typedef void *(*mail_compose_session_json_callback)(void *session_box, void *user_data, char **error_out);
typedef void *(*mail_message_action_decision_callback)(void *message_box, void *user_data, char **error_out);
typedef void *(*mail_message_action_headers_callback)(void *user_data, char **error_out);
typedef void *(*mail_content_blocker_json_callback)(void *user_data, char **error_out);
typedef void *(*mail_message_security_decode_callback)(const uint8_t *bytes, size_t len, void *user_data, char **error_out);
typedef void *(*mail_message_security_encode_callback)(void *message_box, void *context_box, void *user_data, char **error_out);
typedef void *(*mail_message_security_controller_callback)(void *json_box, void *user_data, char **error_out);
typedef void (*mail_release_user_data_callback)(void *user_data);
typedef void (*mail_async_callback)(const void *result, const char *error, void *ctx);
typedef void (*mail_byte_sink)(void *ctx, const uint8_t *bytes, size_t len);

void mail_object_release(void *ptr);
void *mail_json_box_new(const char *json);
char *mail_json_box_copy(void *box);
bool mail_json_box_set_payload(void *box, const char *key, const uint8_t *bytes, size_t len);
void mail_box_copy_payload(void *box, const char *key, mail_byte_sink sink, void *ctx);
char *mail_compose_session_error_domain_copy(void);
char *mail_message_security_error_domain_copy(void);
void *mail_extension_view_controller_new(void);

void *mail_message_action_decision_box_new(void *decision, const char *json);
char *mail_message_action_decision_box_copy_json(void *box);
void *mail_message_action_decision_box_copy_value(void *box);

void *mail_decoded_message_box_new(void *message, const char *json);
char *mail_decoded_message_box_copy_json(void *box);
void *mail_decoded_message_box_copy_value(void *box);
void mail_decoded_message_box_copy_raw_data(void *box, mail_byte_sink sink, void *ctx);

void *mail_outgoing_message_encoding_status_box_new(void *status, const char *json);
char *mail_outgoing_message_encoding_status_box_copy_json(void *box);
void *mail_outgoing_message_encoding_status_box_copy_value(void *box);

void *mail_message_encoding_result_box_new(void *result, const char *json);
char *mail_message_encoding_result_box_copy_json(void *box);
void *mail_message_encoding_result_box_copy_value(void *box);
void mail_message_encoding_result_box_copy_raw_data(void *box, mail_byte_sink sink, void *ctx);

int32_t mail_email_address_new(const char *raw_string, void **out_address, char **error_out);
char *mail_email_address_raw_string(void *address);
char *mail_email_address_address_string(void *address);

void *mail_address_annotation_error(const char *localized_description);
void *mail_address_annotation_warning(const char *localized_description);
void *mail_address_annotation_success(const char *localized_description);

void *mail_message_action_move_to_trash(void);
void *mail_message_action_move_to_archive(void);
void *mail_message_action_move_to_junk(void);
void *mail_message_action_mark_as_read(void);
void *mail_message_action_mark_as_unread(void);
void *mail_message_action_flag(int32_t flag);
void *mail_message_action_set_background_color(int32_t color);

void *mail_message_action_decision_invoke_again_with_body(void);
void *mail_message_action_decision_action(void *action);
void *mail_message_action_decision_actions(void *const *actions, size_t action_count);

int32_t mail_message_action_extension_new(
    mail_message_action_decision_callback decide_callback,
    mail_message_action_headers_callback required_headers_callback,
    mail_release_user_data_callback release_callback,
    void *user_data,
    void **out_handler,
    char **error_out
);
void *mail_message_action_extension_test_decide_action(void *handler, void *message_box, char **error_out);
void *mail_message_action_extension_test_required_headers(void *handler, char **error_out);

int32_t mail_compose_session_new(void *session_box, void **out_session, char **error_out);
char *mail_compose_session_copy_json(void *session);
int32_t mail_compose_session_reload(void *session, char **error_out);
size_t mail_compose_session_reload_count(void *session);

int32_t mail_compose_session_handler_new(
    mail_compose_session_void_callback did_begin_callback,
    mail_compose_session_void_callback did_end_callback,
    mail_compose_session_controller_callback view_controller_callback,
    mail_compose_session_json_callback annotate_callback,
    mail_compose_session_json_callback can_send_callback,
    mail_compose_session_json_callback additional_headers_callback,
    mail_release_user_data_callback release_callback,
    void *user_data,
    void **out_handler,
    char **error_out
);
int32_t mail_compose_session_handler_test_did_begin(void *handler, void *session, char **error_out);
int32_t mail_compose_session_handler_test_did_end(void *handler, void *session, char **error_out);
void *mail_compose_session_handler_test_view_controller(void *handler, void *session, char **error_out);
void *mail_compose_session_handler_test_annotate_addresses(void *handler, void *session, char **error_out);
void *mail_compose_session_handler_test_can_send_message(void *handler, void *session, char **error_out);
void *mail_compose_session_handler_test_additional_headers(void *handler, void *session, char **error_out);

int32_t mail_content_blocker_new(
    mail_content_blocker_json_callback content_rules_callback,
    mail_release_user_data_callback release_callback,
    void *user_data,
    void **out_blocker,
    char **error_out
);
void *mail_content_blocker_test_content_rules_json(void *blocker, char **error_out);

int32_t mail_extension_new(
    void *compose_session_handler,
    void *message_action_handler,
    void *content_blocker,
    void *message_security_handler,
    void **out_extension,
    char **error_out
);
bool mail_extension_test_has_compose_session_handler(void *extension_ptr);
bool mail_extension_test_has_message_action_handler(void *extension_ptr);
bool mail_extension_test_has_content_blocker(void *extension_ptr);
bool mail_extension_test_has_message_security_handler(void *extension_ptr);

int32_t mail_decoded_message_banner_new(
    const char *title,
    const char *primary_action_title,
    bool dismissable,
    void **out_banner,
    char **error_out
);
int32_t mail_message_signer_new(
    void *const *addresses,
    size_t address_count,
    const char *label,
    const uint8_t *context_bytes,
    size_t context_len,
    bool has_context,
    void **out_signer,
    char **error_out
);
int32_t mail_message_security_information_new(
    void *const *signers,
    size_t signer_count,
    bool is_encrypted,
    const char *signing_error_json,
    const char *encryption_error_json,
    bool should_block_remote_content,
    const char *localized_reason,
    void **out_info,
    char **error_out
);
int32_t mail_decoded_message_new(
    const uint8_t *raw_data_bytes,
    size_t raw_data_len,
    bool has_raw_data,
    void *security_information,
    const uint8_t *context_bytes,
    size_t context_len,
    bool has_context,
    void *banner,
    void **out_message,
    char **error_out
);
int32_t mail_encoded_outgoing_message_new(
    const uint8_t *raw_data_bytes,
    size_t raw_data_len,
    bool is_signed,
    bool is_encrypted,
    void **out_message,
    char **error_out
);
int32_t mail_outgoing_message_encoding_status_new(
    bool can_sign,
    bool can_encrypt,
    const char *security_error_json,
    void *const *addresses_failing_encryption,
    size_t failing_address_count,
    void **out_status,
    char **error_out
);
int32_t mail_message_encoding_result_new(
    void *encoded_message,
    const char *signing_error_json,
    const char *encryption_error_json,
    void **out_result,
    char **error_out
);

int32_t mail_message_security_handler_new(
    mail_message_security_decode_callback decode_callback,
    mail_message_security_encode_callback encoding_status_callback,
    mail_message_security_encode_callback encode_message_callback,
    mail_message_security_controller_callback signers_view_controller_callback,
    mail_message_security_controller_callback message_context_view_controller_callback,
    mail_message_security_controller_callback primary_action_callback,
    mail_release_user_data_callback release_callback,
    void *user_data,
    void **out_handler,
    char **error_out
);
void *mail_message_security_handler_test_decode_message(void *handler, const uint8_t *bytes, size_t len, char **error_out);
void *mail_message_security_handler_test_encoding_status(void *handler, void *message_box, void *context_box, char **error_out);
void *mail_message_security_handler_test_encode_message(void *handler, void *message_box, void *context_box, char **error_out);
void *mail_message_security_handler_test_view_controller_for_signers(void *handler, void *signers_box, char **error_out);
void *mail_message_security_handler_test_view_controller_for_message_context(void *handler, void *context_box, char **error_out);
void *mail_message_security_handler_test_primary_action_clicked(void *handler, void *context_box, char **error_out);

int32_t mail_extension_manager_reload_visible_messages(uint64_t timeout_millis, char **error_out);
int32_t mail_extension_manager_reload_content_blocker(
    const char *identifier,
    uint64_t timeout_millis,
    char **error_out
);

void mail_message_action_decision_async(void *handler, void *message_box, mail_async_callback cb, void *ctx);
void mail_content_blocker_content_rules_async(void *blocker, mail_async_callback cb, void *ctx);
void mail_compose_session_handler_annotate_addresses_async(
    void *handler,
    void *session,
    mail_async_callback cb,
    void *ctx
);
void mail_compose_session_handler_allow_message_send_async(
    void *handler,
    void *session,
    mail_async_callback cb,
    void *ctx
);
void mail_message_security_handler_encoding_status_async(
    void *handler,
    void *message_box,
    void *context_box,
    mail_async_callback cb,
    void *ctx
);
void mail_message_security_handler_encode_message_async(
    void *handler,
    void *message_box,
    void *context_box,
    mail_async_callback cb,
    void *ctx
);
void mail_message_security_handler_primary_action_clicked_async(
    void *handler,
    void *context_box,
    mail_async_callback cb,
    void *ctx
);
void mail_extension_manager_reload_visible_messages_async(mail_async_callback cb, void *ctx);
void mail_extension_manager_reload_content_blocker_async(
    const char *identifier,
    mail_async_callback cb,
    void *ctx
);

#endif
