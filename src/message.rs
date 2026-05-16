use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::address::EmailAddressInfo;
use crate::authentication_result::AuthenticationResult;
use crate::eml_attachment::EmlAttachment;
use crate::error::MailKitError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageState {
    Received,
    Draft,
    Sending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageEncryptionState {
    Unknown,
    NotEncrypted,
    Encrypted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    pub state: MessageState,
    pub encryption_state: MessageEncryptionState,
    pub subject: String,
    pub from_address: EmailAddressInfo,
    pub to_addresses: Vec<EmailAddressInfo>,
    pub cc_addresses: Vec<EmailAddressInfo>,
    pub bcc_addresses: Vec<EmailAddressInfo>,
    pub reply_to_addresses: Vec<EmailAddressInfo>,
    pub all_recipient_addresses: Vec<EmailAddressInfo>,
    pub date_sent: Option<String>,
    pub date_received: Option<String>,
    pub headers: Option<BTreeMap<String, Vec<String>>>,
    pub raw_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    info: MessageInfo,
}

pub type MEMessage = Message;

impl Message {
    #[must_use]
    pub const fn new(info: MessageInfo) -> Self {
        Self { info }
    }

    #[must_use]
    pub const fn info(&self) -> &MessageInfo {
        &self.info
    }

    pub fn authentication_results(&self) -> Result<Vec<AuthenticationResult>, MailKitError> {
        AuthenticationResult::parse_headers(self.info.headers.as_ref())
    }

    pub fn attachments(&self) -> Result<Vec<EmlAttachment>, MailKitError> {
        self.info
            .raw_data
            .as_deref()
            .map_or_else(|| Ok(Vec::new()), EmlAttachment::parse_all)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{Message, MessageEncryptionState, MessageInfo, MessageState};
    use crate::address::EmailAddressInfo;

    fn message_info() -> MessageInfo {
        MessageInfo {
            state: MessageState::Received,
            encryption_state: MessageEncryptionState::Encrypted,
            subject: "Quarterly report".into(),
            from_address: EmailAddressInfo {
                raw_string: "Alice <alice@example.com>".into(),
                address_string: Some("alice@example.com".into()),
            },
            to_addresses: vec![EmailAddressInfo {
                raw_string: "Bob <bob@example.com>".into(),
                address_string: Some("bob@example.com".into()),
            }],
            cc_addresses: Vec::new(),
            bcc_addresses: Vec::new(),
            reply_to_addresses: Vec::new(),
            all_recipient_addresses: vec![EmailAddressInfo {
                raw_string: "Bob <bob@example.com>".into(),
                address_string: Some("bob@example.com".into()),
            }],
            date_sent: Some("2026-05-16T00:00:00Z".into()),
            date_received: Some("2026-05-16T00:00:01Z".into()),
            headers: Some(BTreeMap::from([(
                "Authentication-Results".into(),
                vec![
                    "mx.example.com; dkim=pass header.i=@example.com; spf=pass smtp.mailfrom=example.com"
                        .into(),
                ],
            )])),
            raw_data: Some(
                concat!(
                    "From: Alice <alice@example.com>\r\n",
                    "To: Bob <bob@example.com>\r\n",
                    "Subject: Quarterly report\r\n",
                    "MIME-Version: 1.0\r\n",
                    "Content-Type: multipart/mixed; boundary=frontier\r\n",
                    "\r\n",
                    "--frontier\r\n",
                    "Content-Type: text/plain\r\n\r\nHello\r\n",
                    "--frontier\r\n",
                    "Content-Type: text/plain; name=report.txt\r\n",
                    "Content-Disposition: attachment; filename=report.txt\r\n\r\nreport body\r\n",
                    "--frontier--\r\n"
                )
                .as_bytes()
                .to_vec(),
            ),
        }
    }

    #[test]
    fn parses_attachments_from_message_raw_data() {
        let message = Message::new(message_info());
        let attachments = message.attachments().expect("attachments");
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].file_name.as_deref(), Some("report.txt"));
    }

    #[test]
    fn parses_authentication_results_from_headers() {
        let message = Message::new(message_info());
        let results = message.authentication_results().expect("auth results");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].methods[0].method, "dkim");
    }
}
