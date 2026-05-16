use mailkit::{AddressAnnotation, AddressAnnotationKind, EmailAddress};

#[test]
fn address_area_round_trips_mailkit_values() {
    let address = EmailAddress::new("Alice Example <alice@example.com>").expect("email address");
    let info = address.info().expect("address info");
    assert_eq!(info.address_string.as_deref(), Some("alice@example.com"));

    let annotation = AddressAnnotation::success("Looks good").expect("address annotation");
    assert_eq!(annotation.kind(), AddressAnnotationKind::Success);
    assert_eq!(annotation.localized_description(), "Looks good");
}
