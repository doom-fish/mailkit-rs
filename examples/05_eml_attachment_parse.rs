use mailkit::EmlAttachment;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = concat!(
        "From: sender@example.com\r\n",
        "To: recipient@example.com\r\n",
        "Subject: attachment test\r\n",
        "MIME-Version: 1.0\r\n",
        "Content-Type: multipart/mixed; boundary=frontier\r\n",
        "\r\n",
        "--frontier\r\n",
        "Content-Type: text/plain\r\n\r\nHello\r\n",
        "--frontier\r\n",
        "Content-Type: text/plain; name=hello.txt\r\n",
        "Content-Disposition: attachment; filename=hello.txt\r\n",
        "Content-Transfer-Encoding: 7bit\r\n\r\nhello attachment\r\n",
        "--frontier--\r\n"
    );
    let attachments = EmlAttachment::parse_all(raw.as_bytes())?;
    assert_eq!(attachments[0].file_name.as_deref(), Some("hello.txt"));
    println!("parsed {} EML attachment(s)", attachments.len());
    Ok(())
}
