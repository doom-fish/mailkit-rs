mod support;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use mailkit::{
    compose_session_error_domain, AddressAnnotation, ComposeSession, ComposeSessionHandler,
    ComposeSessionHandlerDelegate, ExtensionViewController, NSErrorInfo,
};

struct RecordingComposeHandler {
    events: Arc<Mutex<Vec<String>>>,
}

impl ComposeSessionHandlerDelegate for RecordingComposeHandler {
    fn mail_compose_session_did_begin(
        &self,
        session: &ComposeSession,
    ) -> Result<(), mailkit::MailKitError> {
        self.events
            .lock()
            .expect("events")
            .push(session.info().session_id.clone());
        session.reload_session()?;
        Ok(())
    }

    fn mail_compose_session_did_end(
        &self,
        _session: &ComposeSession,
    ) -> Result<(), mailkit::MailKitError> {
        self.events.lock().expect("events").push("ended".into());
        Ok(())
    }

    fn view_controller_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<ExtensionViewController, mailkit::MailKitError> {
        ExtensionViewController::new()
    }

    fn annotate_addresses_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<Vec<mailkit::AddressAnnotationMatch>, mailkit::MailKitError> {
        let annotation = AddressAnnotation::warning("Missing S/MIME certificate")?;
        Ok(vec![mailkit::AddressAnnotationMatch::new(
            "Bob Example <bob@example.com>",
            &annotation,
        )])
    }

    fn allow_message_send_for_session(&self, _session: &ComposeSession) -> Result<(), NSErrorInfo> {
        Err(NSErrorInfo::new(
            compose_session_error_domain(),
            mailkit::ComposeSessionErrorCode::InvalidRecipients.as_code(),
            "Missing recipient certificate",
        ))
    }

    fn additional_headers_for_session(
        &self,
        _session: &ComposeSession,
    ) -> Result<BTreeMap<String, Vec<String>>, mailkit::MailKitError> {
        Ok(BTreeMap::from([(
            "X-MailKit-Test".into(),
            vec!["compose-session".into()],
        )]))
    }
}

#[test]
fn compose_session_handler_exposes_round_trip_behaviour() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let handler = ComposeSessionHandler::new(RecordingComposeHandler {
        events: Arc::clone(&events),
    })
    .expect("compose handler");
    let session =
        ComposeSession::new(support::sample_compose_session_info()).expect("compose session");

    handler
        .mail_compose_session_did_begin(&session)
        .expect("begin callback");
    handler
        .mail_compose_session_did_end(&session)
        .expect("end callback");
    let _controller = handler
        .view_controller_for_session(&session)
        .expect("view controller");
    let annotations = handler
        .annotate_addresses_for_session(&session)
        .expect("annotations");
    assert_eq!(annotations.len(), 1);
    let send_error = handler
        .allow_message_send_for_session(&session)
        .expect_err("send error");
    assert!(send_error
        .message()
        .contains("Missing recipient certificate"));
    let headers = handler
        .additional_headers_for_session(&session)
        .expect("additional headers");
    assert_eq!(headers["X-MailKit-Test"], vec!["compose-session"]);
    assert_eq!(
        events.lock().expect("events").as_slice(),
        [
            "00000000-0000-0000-0000-000000000002".to_string(),
            "ended".to_string(),
        ]
    );
}
