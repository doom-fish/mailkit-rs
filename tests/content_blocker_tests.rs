use mailkit::{ContentBlocker, ContentBlockerProvider};

struct StaticRules;

impl ContentBlockerProvider for StaticRules {
    fn content_rules_json(&self) -> Result<Vec<u8>, mailkit::MailKitError> {
        Ok(br#"[{\"trigger\":{\"url-filter\":\".*\"},\"action\":{\"type\":\"block\"}}]"#.to_vec())
    }
}

#[test]
fn content_blocker_returns_json_rules() {
    let blocker = ContentBlocker::new(StaticRules).expect("content blocker");
    let rules = blocker.content_rules_json().expect("content rules");
    assert!(String::from_utf8(rules)
        .expect("utf8")
        .contains("url-filter"));
}
