use mailkit::{ContentBlocker, ContentBlockerProvider};

struct ExampleContentBlocker;

impl ContentBlockerProvider for ExampleContentBlocker {
    fn content_rules_json(&self) -> Result<Vec<u8>, mailkit::MailKitError> {
        Ok(
            br#"[{\"trigger\":{\"url-filter\":\".*tracking.*\"},\"action\":{\"type\":\"block\"}}]"#
                .to_vec(),
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let blocker = ContentBlocker::new(ExampleContentBlocker)?;
    let rules = String::from_utf8(blocker.content_rules_json()?)?;
    assert!(rules.contains("tracking"));
    println!("content blocker returned MailKit rules JSON");
    Ok(())
}
