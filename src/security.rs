use core::ffi::c_void;
use std::mem;
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::address::{EmailAddress, EmailAddressInfo};
use crate::error::{from_swift, take_owned_c_string, MailKitError, NSErrorInfo};
use crate::ffi;
use crate::private::{encode_json, optional_bytes_parts, read_bytes, to_cstring};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEDecodedMessageBanner`.
pub struct DecodedMessageBannerInfo {
    /// Stores the banner title shown by MailKit.
    pub title: String,
    /// Stores the title for the banner's primary action.
    pub primary_action_title: String,
    /// Indicates whether MailKit allows dismissing the banner.
    pub dismissable: bool,
}

/// Wraps `MEDecodedMessageBanner`.
pub struct DecodedMessageBanner {
    pub(crate) raw: *mut c_void,
    info: DecodedMessageBannerInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEMessageSigner`.
pub struct MessageSignerInfo {
    /// Stores the signer addresses reported by MailKit.
    pub email_addresses: Vec<EmailAddressInfo>,
    /// Stores the signer label shown by MailKit.
    pub label: String,
    /// Stores signer-specific opaque context bytes.
    pub context: Option<Vec<u8>>,
}

/// Wraps `MEMessageSigner`.
pub struct MessageSigner {
    pub(crate) raw: *mut c_void,
    info: MessageSignerInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures the remote-content policy reported by MailKit security information.
pub struct RemoteContentBlocking {
    /// Indicates whether remote content should be blocked.
    pub should_block: bool,
    /// Stores the localized reason for the remote-content policy.
    pub localized_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEMessageSecurityInformation`.
pub struct MessageSecurityInformationInfo {
    /// Stores the message signers reported by MailKit.
    pub signers: Vec<MessageSignerInfo>,
    /// Indicates whether the message is encrypted.
    pub is_encrypted: bool,
    /// Stores a signing error when one is present.
    pub signing_error: Option<NSErrorInfo>,
    /// Stores an encryption error when one is present.
    pub encryption_error: Option<NSErrorInfo>,
    /// Stores the remote-content blocking policy.
    pub remote_content: RemoteContentBlocking,
}

/// Wraps `MEMessageSecurityInformation`.
pub struct MessageSecurityInformation {
    pub(crate) raw: *mut c_void,
    info: MessageSecurityInformationInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEDecodedMessage`.
pub struct DecodedMessageInfo {
    /// Stores decoded RFC 822 message bytes when available.
    pub raw_data: Option<Vec<u8>>,
    /// Stores the message security information snapshot.
    pub security_information: MessageSecurityInformationInfo,
    /// Stores opaque context bytes for follow-up actions.
    pub context: Option<Vec<u8>>,
    /// Stores the optional banner MailKit should display.
    pub banner: Option<DecodedMessageBannerInfo>,
}

/// Wraps `MEDecodedMessage`.
pub struct DecodedMessage {
    raw: *mut c_void,
    info: DecodedMessageInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures the signing and encryption choices for MailKit outgoing-message encoding.
pub struct MessageEncoding {
    /// Indicates whether outgoing content should be signed.
    pub is_signed: bool,
    /// Indicates whether outgoing content should be encrypted.
    pub is_encrypted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEEncodedOutgoingMessage`.
pub struct EncodedOutgoingMessageInfo {
    /// Stores encoded RFC 822 bytes ready to send.
    pub raw_data: Vec<u8>,
    /// Stores the signing and encryption choices used to encode the message.
    pub encoding: MessageEncoding,
}

/// Wraps `MEEncodedOutgoingMessage`.
pub struct EncodedOutgoingMessage {
    pub(crate) raw: *mut c_void,
    info: EncodedOutgoingMessageInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures the capabilities MailKit reports for outgoing-message security.
pub struct OutgoingMessageCapabilities {
    /// Indicates whether signing is available.
    pub can_sign: bool,
    /// Indicates whether encryption is available.
    pub can_encrypt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEOutgoingMessageEncodingStatus`.
pub struct OutgoingMessageEncodingStatusInfo {
    /// Stores the available signing and encryption capabilities.
    pub capabilities: OutgoingMessageCapabilities,
    /// Stores the primary security error when one is present.
    pub security_error: Option<NSErrorInfo>,
    /// Stores recipient addresses that cannot be encrypted.
    pub addresses_failing_encryption: Vec<EmailAddressInfo>,
}

/// Wraps `MEOutgoingMessageEncodingStatus`.
pub struct OutgoingMessageEncodingStatus {
    raw: *mut c_void,
    info: OutgoingMessageEncodingStatusInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Captures a Rust snapshot of `MEMessageEncodingResult`.
pub struct MessageEncodingResultInfo {
    /// Stores the encoded outgoing message when one was produced.
    pub encoded_message: Option<EncodedOutgoingMessageInfo>,
    /// Stores a signing error when one occurred.
    pub signing_error: Option<NSErrorInfo>,
    /// Stores an encryption error when one occurred.
    pub encryption_error: Option<NSErrorInfo>,
}

/// Wraps `MEMessageEncodingResult`.
pub struct MessageEncodingResult {
    raw: *mut c_void,
    info: MessageEncodingResultInfo,
}

impl DecodedMessageBanner {
    /// Creates a `MEDecodedMessageBanner` wrapper.
    pub fn new(
        title: &str,
        primary_action_title: &str,
        dismissable: bool,
    ) -> Result<Self, MailKitError> {
        let title = to_cstring(title)?;
        let primary_action_title = to_cstring(primary_action_title)?;
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_decoded_message_banner_new(
                title.as_ptr(),
                primary_action_title.as_ptr(),
                dismissable,
                &raw mut raw,
                &raw mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self {
            raw,
            info: DecodedMessageBannerInfo {
                title: title.to_string_lossy().into_owned(),
                primary_action_title: primary_action_title.to_string_lossy().into_owned(),
                dismissable,
            },
        })
    }

    #[must_use]
    /// Returns the cached decoded-message banner snapshot.
    pub const fn info(&self) -> &DecodedMessageBannerInfo {
        &self.info
    }
}

impl MessageSigner {
    /// Creates a `MEMessageSigner` wrapper.
    pub fn new(
        email_addresses: &[&EmailAddress],
        label: &str,
        context: Option<&[u8]>,
    ) -> Result<Self, MailKitError> {
        let label = to_cstring(label)?;
        let raw_addresses: Vec<*mut c_void> =
            email_addresses.iter().map(|address| address.raw).collect();
        let info_addresses: Vec<EmailAddressInfo> = email_addresses
            .iter()
            .map(|address| address.info())
            .collect::<Result<_, _>>()?;
        let (context_ptr, context_len, has_context) = optional_bytes_parts(context);
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_message_signer_new(
                raw_addresses.as_ptr(),
                raw_addresses.len(),
                label.as_ptr(),
                context_ptr,
                context_len,
                has_context,
                &raw mut raw,
                &raw mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self {
            raw,
            info: MessageSignerInfo {
                email_addresses: info_addresses,
                label: label.to_string_lossy().into_owned(),
                context: context.map(<[_]>::to_vec),
            },
        })
    }

    #[must_use]
    /// Returns the cached message-signer snapshot.
    pub const fn info(&self) -> &MessageSignerInfo {
        &self.info
    }
}

impl MessageSecurityInformation {
    /// Creates a `MEMessageSecurityInformation` wrapper.
    pub fn new(
        signers: &[&MessageSigner],
        is_encrypted: bool,
        signing_error: Option<&NSErrorInfo>,
        encryption_error: Option<&NSErrorInfo>,
        remote_content: RemoteContentBlocking,
    ) -> Result<Self, MailKitError> {
        let raw_signers: Vec<*mut c_void> = signers.iter().map(|signer| signer.raw).collect();
        let signer_infos = signers.iter().map(|signer| signer.info.clone()).collect();
        let signing_error_json = signing_error.map(encode_json).transpose()?;
        let encryption_error_json = encryption_error.map(encode_json).transpose()?;
        let localized_reason = remote_content
            .localized_reason
            .as_deref()
            .map(to_cstring)
            .transpose()?;
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_message_security_information_new(
                raw_signers.as_ptr(),
                raw_signers.len(),
                is_encrypted,
                signing_error_json
                    .as_ref()
                    .map_or(ptr::null(), |json| json.as_ptr()),
                encryption_error_json
                    .as_ref()
                    .map_or(ptr::null(), |json| json.as_ptr()),
                remote_content.should_block,
                localized_reason
                    .as_ref()
                    .map_or(ptr::null(), |reason| reason.as_ptr()),
                &raw mut raw,
                &raw mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self {
            raw,
            info: MessageSecurityInformationInfo {
                signers: signer_infos,
                is_encrypted,
                signing_error: signing_error.cloned(),
                encryption_error: encryption_error.cloned(),
                remote_content,
            },
        })
    }

    #[must_use]
    /// Returns the cached message-security-information snapshot.
    pub const fn info(&self) -> &MessageSecurityInformationInfo {
        &self.info
    }
}

impl DecodedMessage {
    /// Creates a `MEDecodedMessage` wrapper.
    pub fn new(
        raw_data: Option<&[u8]>,
        security_information: &MessageSecurityInformation,
        context: Option<&[u8]>,
        banner: Option<&DecodedMessageBanner>,
    ) -> Result<Self, MailKitError> {
        let (raw_data_ptr, raw_data_len, has_raw_data) = optional_bytes_parts(raw_data);
        let (context_ptr, context_len, has_context) = optional_bytes_parts(context);
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_decoded_message_new(
                raw_data_ptr,
                raw_data_len,
                has_raw_data,
                security_information.raw,
                context_ptr,
                context_len,
                has_context,
                banner.map_or(ptr::null_mut(), |banner| banner.raw),
                &raw mut raw,
                &raw mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self {
            raw,
            info: DecodedMessageInfo {
                raw_data: raw_data.map(<[_]>::to_vec),
                security_information: security_information.info.clone(),
                context: context.map(<[_]>::to_vec),
                banner: banner.map(|banner| banner.info.clone()),
            },
        })
    }

    #[must_use]
    /// Returns the cached decoded-message snapshot.
    pub const fn info(&self) -> &DecodedMessageInfo {
        &self.info
    }

    pub(crate) fn into_raw(mut self) -> *mut c_void {
        mem::replace(&mut self.raw, ptr::null_mut())
    }

    pub(crate) fn into_box_raw(mut self) -> Result<*mut c_void, MailKitError> {
        self.info.raw_data = None;
        let info_json = encode_json(&self.info)?;
        let box_raw =
            unsafe { ffi::core::mail_decoded_message_box_new(self.into_raw(), info_json.as_ptr()) };
        if box_raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit failed to allocate a decoded-message box".into(),
            ));
        }
        Ok(box_raw)
    }

    pub(crate) unsafe fn from_box_raw(box_raw: *mut c_void) -> Result<Self, MailKitError> {
        if box_raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit returned a null decoded-message box".into(),
            ));
        }
        let info_json =
            take_owned_c_string(unsafe { ffi::core::mail_decoded_message_box_copy_json(box_raw) });
        let mut info = match serde_json::from_str::<DecodedMessageInfo>(&info_json) {
            Ok(info) => info,
            Err(error) => {
                unsafe { ffi::core::mail_object_release(box_raw) };
                return Err(MailKitError::FrameworkError(format!(
                    "failed to decode MailKit decoded-message box: {error}"
                )));
            }
        };
        info.raw_data = read_bytes(|sink, context| unsafe {
            ffi::core::mail_decoded_message_box_copy_raw_data(box_raw, sink, context);
        });
        let raw = unsafe { ffi::core::mail_decoded_message_box_copy_value(box_raw) };
        unsafe { ffi::core::mail_object_release(box_raw) };
        Ok(Self { raw, info })
    }
}

impl EncodedOutgoingMessage {
    /// Creates a `MEEncodedOutgoingMessage` wrapper.
    pub fn new(raw_data: &[u8], encoding: MessageEncoding) -> Result<Self, MailKitError> {
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_encoded_outgoing_message_new(
                raw_data.as_ptr(),
                raw_data.len(),
                encoding.is_signed,
                encoding.is_encrypted,
                &raw mut raw,
                &raw mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self {
            raw,
            info: EncodedOutgoingMessageInfo {
                raw_data: raw_data.to_vec(),
                encoding,
            },
        })
    }

    #[must_use]
    /// Returns the cached encoded-outgoing-message snapshot.
    pub const fn info(&self) -> &EncodedOutgoingMessageInfo {
        &self.info
    }
}

impl OutgoingMessageEncodingStatus {
    /// Creates a `MEOutgoingMessageEncodingStatus` wrapper.
    pub fn new(
        capabilities: OutgoingMessageCapabilities,
        security_error: Option<&NSErrorInfo>,
        addresses_failing_encryption: &[&EmailAddress],
    ) -> Result<Self, MailKitError> {
        let raw_addresses: Vec<*mut c_void> = addresses_failing_encryption
            .iter()
            .map(|address| address.raw)
            .collect();
        let address_infos = addresses_failing_encryption
            .iter()
            .map(|address| address.info())
            .collect::<Result<_, _>>()?;
        let security_error_json = security_error.map(encode_json).transpose()?;
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_outgoing_message_encoding_status_new(
                capabilities.can_sign,
                capabilities.can_encrypt,
                security_error_json
                    .as_ref()
                    .map_or(ptr::null(), |json| json.as_ptr()),
                raw_addresses.as_ptr(),
                raw_addresses.len(),
                &raw mut raw,
                &raw mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self {
            raw,
            info: OutgoingMessageEncodingStatusInfo {
                capabilities,
                security_error: security_error.cloned(),
                addresses_failing_encryption: address_infos,
            },
        })
    }

    #[must_use]
    /// Returns the cached outgoing-message-encoding-status snapshot.
    pub const fn info(&self) -> &OutgoingMessageEncodingStatusInfo {
        &self.info
    }

    pub(crate) fn into_raw(mut self) -> *mut c_void {
        mem::replace(&mut self.raw, ptr::null_mut())
    }

    pub(crate) fn into_box_raw(self) -> Result<*mut c_void, MailKitError> {
        let info_json = encode_json(&self.info)?;
        let box_raw = unsafe {
            ffi::core::mail_outgoing_message_encoding_status_box_new(
                self.into_raw(),
                info_json.as_ptr(),
            )
        };
        if box_raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit failed to allocate an outgoing-message status box".into(),
            ));
        }
        Ok(box_raw)
    }

    pub(crate) unsafe fn from_box_raw(box_raw: *mut c_void) -> Result<Self, MailKitError> {
        if box_raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit returned a null outgoing-message status box".into(),
            ));
        }
        let info_json = take_owned_c_string(unsafe {
            ffi::core::mail_outgoing_message_encoding_status_box_copy_json(box_raw)
        });
        let info = match serde_json::from_str::<OutgoingMessageEncodingStatusInfo>(&info_json) {
            Ok(info) => info,
            Err(error) => {
                unsafe { ffi::core::mail_object_release(box_raw) };
                return Err(MailKitError::FrameworkError(format!(
                    "failed to decode MailKit outgoing-message status box: {error}"
                )));
            }
        };
        let raw =
            unsafe { ffi::core::mail_outgoing_message_encoding_status_box_copy_value(box_raw) };
        unsafe { ffi::core::mail_object_release(box_raw) };
        Ok(Self { raw, info })
    }
}

impl MessageEncodingResult {
    /// Creates a `MEMessageEncodingResult` wrapper.
    pub fn new(
        encoded_message: Option<&EncodedOutgoingMessage>,
        signing_error: Option<&NSErrorInfo>,
        encryption_error: Option<&NSErrorInfo>,
    ) -> Result<Self, MailKitError> {
        let signing_error_json = signing_error.map(encode_json).transpose()?;
        let encryption_error_json = encryption_error.map(encode_json).transpose()?;
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_message_encoding_result_new(
                encoded_message.map_or(ptr::null_mut(), |message| message.raw),
                signing_error_json
                    .as_ref()
                    .map_or(ptr::null(), |json| json.as_ptr()),
                encryption_error_json
                    .as_ref()
                    .map_or(ptr::null(), |json| json.as_ptr()),
                &raw mut raw,
                &raw mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(Self {
            raw,
            info: MessageEncodingResultInfo {
                encoded_message: encoded_message.map(|message| message.info.clone()),
                signing_error: signing_error.cloned(),
                encryption_error: encryption_error.cloned(),
            },
        })
    }

    #[must_use]
    /// Returns the cached message-encoding-result snapshot.
    pub const fn info(&self) -> &MessageEncodingResultInfo {
        &self.info
    }

    pub(crate) fn into_raw(mut self) -> *mut c_void {
        mem::replace(&mut self.raw, ptr::null_mut())
    }

    pub(crate) fn into_box_raw(mut self) -> Result<*mut c_void, MailKitError> {
        if let Some(encoded) = self.info.encoded_message.as_mut() {
            encoded.raw_data = Vec::new();
        }
        let info_json = encode_json(&self.info)?;
        let box_raw = unsafe {
            ffi::core::mail_message_encoding_result_box_new(self.into_raw(), info_json.as_ptr())
        };
        if box_raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit failed to allocate a message-encoding result box".into(),
            ));
        }
        Ok(box_raw)
    }

    pub(crate) unsafe fn from_box_raw(box_raw: *mut c_void) -> Result<Self, MailKitError> {
        if box_raw.is_null() {
            return Err(MailKitError::FrameworkError(
                "MailKit returned a null message-encoding result box".into(),
            ));
        }
        let info_json = take_owned_c_string(unsafe {
            ffi::core::mail_message_encoding_result_box_copy_json(box_raw)
        });
        let mut info = match serde_json::from_str::<MessageEncodingResultInfo>(&info_json) {
            Ok(info) => info,
            Err(error) => {
                unsafe { ffi::core::mail_object_release(box_raw) };
                return Err(MailKitError::FrameworkError(format!(
                    "failed to decode MailKit message-encoding result box: {error}"
                )));
            }
        };
        if let Some(encoded) = info.encoded_message.as_mut() {
            encoded.raw_data = read_bytes(|sink, context| unsafe {
                ffi::core::mail_message_encoding_result_box_copy_raw_data(box_raw, sink, context);
            })
            .unwrap_or_default();
        }
        let raw = unsafe { ffi::core::mail_message_encoding_result_box_copy_value(box_raw) };
        unsafe { ffi::core::mail_object_release(box_raw) };
        Ok(Self { raw, info })
    }
}

impl Drop for DecodedMessageBanner {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

impl Drop for MessageSigner {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

impl Drop for MessageSecurityInformation {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

impl Drop for DecodedMessage {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

impl Drop for EncodedOutgoingMessage {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

impl Drop for OutgoingMessageEncodingStatus {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

impl Drop for MessageEncodingResult {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { ffi::mail_object_release(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DecodedMessage, EncodedOutgoingMessage, MessageEncoding, MessageEncodingResult,
        MessageSecurityInformation, RemoteContentBlocking,
    };
    use crate::error::take_owned_c_string;
    use crate::ffi;

    fn mebibyte() -> Vec<u8> {
        (0..=255u8).rev().cycle().take(1 << 20).collect()
    }

    #[test]
    fn decoded_message_box_round_trips_raw_data_without_json() {
        let raw = mebibyte();
        let security = MessageSecurityInformation::new(
            &[],
            true,
            None,
            None,
            RemoteContentBlocking {
                should_block: false,
                localized_reason: None,
            },
        )
        .expect("security information");
        let message =
            DecodedMessage::new(Some(&raw), &security, Some(b"ctx"), None).expect("message");
        let boxed = message.into_box_raw().expect("box");
        let json =
            take_owned_c_string(unsafe { ffi::core::mail_decoded_message_box_copy_json(boxed) });
        assert!(json.len() < 1024);
        let restored = unsafe { DecodedMessage::from_box_raw(boxed) }.expect("restored");
        assert_eq!(restored.info().raw_data.as_deref(), Some(&raw[..]));
        assert_eq!(restored.info().context.as_deref(), Some(&b"ctx"[..]));
        assert!(restored.info().security_information.is_encrypted);
    }

    #[test]
    fn encoding_result_box_round_trips_raw_data_without_json() {
        let raw = mebibyte();
        let encoded = EncodedOutgoingMessage::new(
            &raw,
            MessageEncoding {
                is_signed: true,
                is_encrypted: false,
            },
        )
        .expect("encoded message");
        let result = MessageEncodingResult::new(Some(&encoded), None, None).expect("result");
        let boxed = result.into_box_raw().expect("box");
        let json = take_owned_c_string(unsafe {
            ffi::core::mail_message_encoding_result_box_copy_json(boxed)
        });
        assert!(json.len() < 1024);
        let restored = unsafe { MessageEncodingResult::from_box_raw(boxed) }.expect("restored");
        let restored = restored
            .info()
            .encoded_message
            .as_ref()
            .expect("encoded message");
        assert_eq!(restored.raw_data, raw);
        assert!(restored.encoding.is_signed);
        assert!(!restored.encoding.is_encrypted);
    }
}
