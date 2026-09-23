use mailkit::{ContentBlocker, ContentBlockerProvider, MailExtension};

struct Rules;

impl ContentBlockerProvider for Rules {
    fn content_rules_json(&self) -> Result<Vec<u8>, mailkit::MailKitError> {
        Ok(b"[]".to_vec())
    }
}

#[no_mangle]
#[allow(clippy::missing_panics_doc)]
pub extern "C" fn mailkit_install_extension() {
    let extension = MailExtension::builder()
        .with_content_blocker(ContentBlocker::new(Rules).expect("blocker"))
        .build()
        .expect("extension");
    extension.register_as_principal().expect("register");
}

#[used]
static KEEP_INSTALL_HOOK: extern "C" fn() = mailkit_install_extension;

#[test]
fn principal_class_installs_the_extension_through_the_hook() {
    let raw = unsafe { mailkit::ffi::extension::mail_extension_test_new_principal() };
    assert!(!raw.is_null());
    let handlers = unsafe {
        [
            mailkit::ffi::extension::mail_extension_test_has_compose_session_handler(raw),
            mailkit::ffi::extension::mail_extension_test_has_message_action_handler(raw),
            mailkit::ffi::extension::mail_extension_test_has_content_blocker(raw),
            mailkit::ffi::extension::mail_extension_test_has_message_security_handler(raw),
        ]
    };
    unsafe { mailkit::ffi::core::mail_object_release(raw) };
    assert_eq!(handlers, [false, false, true, false]);
}
