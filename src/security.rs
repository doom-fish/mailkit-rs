use core::ffi::c_void;
use std::mem::ManuallyDrop;
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::address::{EmailAddress, EmailAddressInfo};
use crate::error::{from_swift, take_owned_c_string, MailKitError, NSErrorInfo};
use crate::ffi;
use crate::private::{encode_json, optional_bytes_parts, to_cstring};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedMessageBannerInfo {
    pub title: String,
    pub primary_action_title: String,
    pub dismissable: bool,
}

pub struct DecodedMessageBanner {
    pub(crate) raw: *mut c_void,
    info: DecodedMessageBannerInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSignerInfo {
    pub email_addresses: Vec<EmailAddressInfo>,
    pub label: String,
    pub context: Option<Vec<u8>>,
}

pub struct MessageSigner {
    pub(crate) raw: *mut c_void,
    info: MessageSignerInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteContentBlocking {
    pub should_block: bool,
    pub localized_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSecurityInformationInfo {
    pub signers: Vec<MessageSignerInfo>,
    pub is_encrypted: bool,
    pub signing_error: Option<NSErrorInfo>,
    pub encryption_error: Option<NSErrorInfo>,
    pub remote_content: RemoteContentBlocking,
}

pub struct MessageSecurityInformation {
    pub(crate) raw: *mut c_void,
    info: MessageSecurityInformationInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedMessageInfo {
    pub raw_data: Option<Vec<u8>>,
    pub security_information: MessageSecurityInformationInfo,
    pub context: Option<Vec<u8>>,
    pub banner: Option<DecodedMessageBannerInfo>,
}

pub struct DecodedMessage {
    raw: *mut c_void,
    info: DecodedMessageInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageEncoding {
    pub is_signed: bool,
    pub is_encrypted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodedOutgoingMessageInfo {
    pub raw_data: Vec<u8>,
    pub encoding: MessageEncoding,
}

pub struct EncodedOutgoingMessage {
    pub(crate) raw: *mut c_void,
    info: EncodedOutgoingMessageInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutgoingMessageCapabilities {
    pub can_sign: bool,
    pub can_encrypt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutgoingMessageEncodingStatusInfo {
    pub capabilities: OutgoingMessageCapabilities,
    pub security_error: Option<NSErrorInfo>,
    pub addresses_failing_encryption: Vec<EmailAddressInfo>,
}

pub struct OutgoingMessageEncodingStatus {
    raw: *mut c_void,
    info: OutgoingMessageEncodingStatusInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageEncodingResultInfo {
    pub encoded_message: Option<EncodedOutgoingMessageInfo>,
    pub signing_error: Option<NSErrorInfo>,
    pub encryption_error: Option<NSErrorInfo>,
}

pub struct MessageEncodingResult {
    raw: *mut c_void,
    info: MessageEncodingResultInfo,
}

impl DecodedMessageBanner {
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
                &mut raw,
                &mut error_ptr,
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
    pub const fn info(&self) -> &DecodedMessageBannerInfo {
        &self.info
    }
}

impl MessageSigner {
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
                &mut raw,
                &mut error_ptr,
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
    pub const fn info(&self) -> &MessageSignerInfo {
        &self.info
    }
}

impl MessageSecurityInformation {
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
                &mut raw,
                &mut error_ptr,
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
    pub const fn info(&self) -> &MessageSecurityInformationInfo {
        &self.info
    }
}

impl DecodedMessage {
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
                &mut raw,
                &mut error_ptr,
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
    pub const fn info(&self) -> &DecodedMessageInfo {
        &self.info
    }

    pub(crate) fn into_raw(self) -> *mut c_void {
        let mut this = ManuallyDrop::new(self);
        let raw = this.raw;
        this.raw = ptr::null_mut();
        raw
    }

    pub(crate) fn into_box_raw(self) -> Result<*mut c_void, MailKitError> {
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
        let info = serde_json::from_str::<DecodedMessageInfo>(&info_json).map_err(|error| {
            MailKitError::FrameworkError(format!(
                "failed to decode MailKit decoded-message box: {error}"
            ))
        })?;
        let raw = unsafe { ffi::core::mail_decoded_message_box_copy_value(box_raw) };
        unsafe { ffi::core::mail_object_release(box_raw) };
        Ok(Self { raw, info })
    }
}

impl EncodedOutgoingMessage {
    pub fn new(raw_data: &[u8], encoding: MessageEncoding) -> Result<Self, MailKitError> {
        let mut raw = ptr::null_mut();
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_encoded_outgoing_message_new(
                raw_data.as_ptr(),
                raw_data.len(),
                encoding.is_signed,
                encoding.is_encrypted,
                &mut raw,
                &mut error_ptr,
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
    pub const fn info(&self) -> &EncodedOutgoingMessageInfo {
        &self.info
    }
}

impl OutgoingMessageEncodingStatus {
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
                &mut raw,
                &mut error_ptr,
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
    pub const fn info(&self) -> &OutgoingMessageEncodingStatusInfo {
        &self.info
    }

    pub(crate) fn into_raw(self) -> *mut c_void {
        let mut this = ManuallyDrop::new(self);
        let raw = this.raw;
        this.raw = ptr::null_mut();
        raw
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
        let info = serde_json::from_str::<OutgoingMessageEncodingStatusInfo>(&info_json).map_err(
            |error| {
                MailKitError::FrameworkError(format!(
                    "failed to decode MailKit outgoing-message status box: {error}"
                ))
            },
        )?;
        let raw =
            unsafe { ffi::core::mail_outgoing_message_encoding_status_box_copy_value(box_raw) };
        unsafe { ffi::core::mail_object_release(box_raw) };
        Ok(Self { raw, info })
    }
}

impl MessageEncodingResult {
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
                &mut raw,
                &mut error_ptr,
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
    pub const fn info(&self) -> &MessageEncodingResultInfo {
        &self.info
    }

    pub(crate) fn into_raw(self) -> *mut c_void {
        let mut this = ManuallyDrop::new(self);
        let raw = this.raw;
        this.raw = ptr::null_mut();
        raw
    }

    pub(crate) fn into_box_raw(self) -> Result<*mut c_void, MailKitError> {
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
        let info =
            serde_json::from_str::<MessageEncodingResultInfo>(&info_json).map_err(|error| {
                MailKitError::FrameworkError(format!(
                    "failed to decode MailKit message-encoding result box: {error}"
                ))
            })?;
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
