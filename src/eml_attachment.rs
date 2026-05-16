use std::collections::BTreeMap;

use mailparse::{parse_mail, DispositionType, MailHeaderMap, ParsedMail};
use serde::{Deserialize, Serialize};

use crate::error::MailKitError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmlAttachment {
    pub content_type: String,
    pub file_name: Option<String>,
    pub content_id: Option<String>,
    pub content_disposition: Option<String>,
    pub content_transfer_encoding: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub data: Vec<u8>,
}

pub type EMLAttachment = EmlAttachment;

impl EmlAttachment {
    pub fn parse_all(raw_eml: &[u8]) -> Result<Vec<Self>, MailKitError> {
        let parsed = parse_mail(raw_eml).map_err(|error| {
            MailKitError::InvalidArgument(format!("failed to parse RFC822 message data: {error}"))
        })?;
        let mut attachments = Vec::new();
        collect_attachments(&parsed, &mut attachments)?;
        Ok(attachments)
    }
}

fn collect_attachments(
    part: &ParsedMail<'_>,
    attachments: &mut Vec<EmlAttachment>,
) -> Result<(), MailKitError> {
    let disposition = part.get_content_disposition();
    let file_name = disposition
        .params
        .get("filename")
        .cloned()
        .or_else(|| part.ctype.params.get("name").cloned());
    let content_disposition = part
        .headers
        .get_first_value("Content-Disposition")
        .map(|_| match &disposition.disposition {
            DispositionType::Inline => "inline".to_string(),
            DispositionType::Attachment => "attachment".to_string(),
            DispositionType::FormData => "form-data".to_string(),
            DispositionType::Extension(value) => value.clone(),
        });
    let content_id = part
        .headers
        .get_first_value("Content-ID")
        .map(|value| value.trim_matches(&['<', '>'][..]).to_string());
    let content_transfer_encoding = part.headers.get_first_value("Content-Transfer-Encoding");
    let is_attachment = file_name.is_some()
        || matches!(disposition.disposition, DispositionType::Attachment)
        || (matches!(disposition.disposition, DispositionType::Inline)
            && disposition.params.contains_key("filename"));

    if part.subparts.is_empty() && is_attachment {
        let data = part.get_body_raw().map_err(|error| {
            MailKitError::FrameworkError(format!("failed to read attachment body: {error}"))
        })?;
        attachments.push(EmlAttachment {
            content_type: part.ctype.mimetype.clone(),
            file_name,
            content_id,
            content_disposition,
            content_transfer_encoding,
            headers: part
                .headers
                .iter()
                .map(|header| (header.get_key(), header.get_value()))
                .collect(),
            data,
        });
    }

    for subpart in &part.subparts {
        collect_attachments(subpart, attachments)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::EmlAttachment;

    #[test]
    fn parses_attachment_metadata_and_body() {
        let raw = concat!(
            "From: sender@example.com\r\n",
            "To: recipient@example.com\r\n",
            "Subject: attachment test\r\n",
            "MIME-Version: 1.0\r\n",
            "Content-Type: multipart/mixed; boundary=frontier\r\n",
            "\r\n",
            "--frontier\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "Hello world\r\n",
            "--frontier\r\n",
            "Content-Type: text/plain; name=hello.txt\r\n",
            "Content-Disposition: attachment; filename=hello.txt\r\n",
            "Content-Transfer-Encoding: 7bit\r\n",
            "\r\n",
            "hello from attachment\r\n",
            "--frontier--\r\n"
        );

        let attachments = EmlAttachment::parse_all(raw.as_bytes()).expect("parse attachments");
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].file_name.as_deref(), Some("hello.txt"));
        assert_eq!(attachments[0].content_type, "text/plain");
        assert_eq!(attachments[0].data, b"hello from attachment");
    }
}
