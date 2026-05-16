use mailkit::{AddressAnnotation, AddressAnnotationKind, EmailAddress};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = EmailAddress::new("Alice Example <alice@example.com>")?;
    let info = address.info()?;
    assert_eq!(info.address_string.as_deref(), Some("alice@example.com"));
    let annotation = AddressAnnotation::success("Looks good")?;
    assert_eq!(annotation.kind(), AddressAnnotationKind::Success);
    println!("parsed {}", info.raw_string);
    Ok(())
}
