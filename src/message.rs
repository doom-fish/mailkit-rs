use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::address::EmailAddressInfo;
use crate::authentication_result::AuthenticationResult;
use crate::eml_attachment::EmlAttachment;
use crate::error::MailKitError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents MailKit message states.
pub enum MessageState {
    /// Represents a received message.
    Received,
    /// Represents a draft message.
    Draft,
    /// Represents a message that is being sent.
    Sending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Represents MailKit message encryption states.
pub enum MessageEncryptionState {
    /// Indicates that MailKit did not report encryption state.
    Unknown,
    /// Indicates that the message is not encrypted.
    NotEncrypted,
    /// Indicates that the message is encrypted.
    Encrypted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEMessage`.
pub struct MessageInfo {
    /// Stores the MailKit message state.
    pub state: MessageState,
    /// Stores the MailKit encryption state.
    pub encryption_state: MessageEncryptionState,
    /// Stores the message subject.
    pub subject: String,
    /// Stores the sender address.
    pub from_address: EmailAddressInfo,
    /// Stores the primary recipient addresses.
    pub to_addresses: Vec<EmailAddressInfo>,
    /// Stores the carbon-copy recipient addresses.
    pub cc_addresses: Vec<EmailAddressInfo>,
    /// Stores the blind-carbon-copy recipient addresses.
    pub bcc_addresses: Vec<EmailAddressInfo>,
    /// Stores the reply-to addresses.
    pub reply_to_addresses: Vec<EmailAddressInfo>,
    /// Stores the full recipient list as reported by MailKit.
    pub all_recipient_addresses: Vec<EmailAddressInfo>,
    /// Stores the sent date string when MailKit provides one.
    pub date_sent: Option<String>,
    /// Stores the received date string when MailKit provides one.
    pub date_received: Option<String>,
    /// Stores message headers when available.
    pub headers: Option<BTreeMap<String, Vec<String>>>,
    /// Stores raw RFC 822 message data when available.
    pub raw_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `MEMessage`.
pub struct Message {
    info: MessageInfo,
}

/// Alias matching MailKit's `MEMessage` name.
pub type MEMessage = Message;

impl Message {
    #[must_use]
    /// Creates a `MEMessage` snapshot from Rust data.
    pub const fn new(info: MessageInfo) -> Self {
        Self { info }
    }

    #[must_use]
    /// Returns the cached message snapshot.
    pub const fn info(&self) -> &MessageInfo {
        &self.info
    }

    /// Parses `Authentication-Results` headers from this message.
    pub fn authentication_results(&self) -> Result<Vec<AuthenticationResult>, MailKitError> {
        AuthenticationResult::parse_headers(self.info.headers.as_ref())
    }

    /// Parses attachments from this message's raw RFC 822 data.
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
