use mailkit::AuthenticationResult;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = AuthenticationResult::parse(
        "mx.example.com; dkim=pass header.i=@example.com; spf=pass smtp.mailfrom=example.com",
    )?;
    assert_eq!(result.methods.len(), 2);
    println!("parsed {} authentication checks", result.methods.len());
    Ok(())
}
