mod support;

use mailkit::{MessageEncodingResult, OutgoingMessageCapabilities};

#[test]
fn encoded_outgoing_message_round_trips_encoding_types() {
    let encoded = support::sample_encoded_message();
    assert!(encoded.info().encoding.is_signed);
    assert!(encoded.info().encoding.is_encrypted);

    let status = support::sample_encoding_status();
    assert_eq!(
        status.info().capabilities,
        OutgoingMessageCapabilities {
            can_sign: true,
            can_encrypt: true,
        }
    );

    let result = MessageEncodingResult::new(Some(&encoded), None, None).expect("encoding result");
    assert!(result.info().encoded_message.is_some());
}
