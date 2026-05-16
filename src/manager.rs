use std::ptr;
use std::time::Duration;

use crate::error::{from_swift, MailKitError};
use crate::ffi;
use crate::private::to_cstring;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct ExtensionManager;

impl ExtensionManager {
    pub fn reload_visible_messages() -> Result<(), MailKitError> {
        Self::reload_visible_messages_with_timeout(DEFAULT_TIMEOUT)
    }

    pub fn reload_visible_messages_with_timeout(timeout: Duration) -> Result<(), MailKitError> {
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_extension_manager_reload_visible_messages(
                duration_to_millis(timeout),
                &mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(())
    }

    pub fn reload_content_blocker(identifier: &str) -> Result<(), MailKitError> {
        Self::reload_content_blocker_with_timeout(identifier, DEFAULT_TIMEOUT)
    }

    pub fn reload_content_blocker_with_timeout(
        identifier: &str,
        timeout: Duration,
    ) -> Result<(), MailKitError> {
        let identifier = to_cstring(identifier)?;
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_extension_manager_reload_content_blocker(
                identifier.as_ptr(),
                duration_to_millis(timeout),
                &mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(())
    }
}

fn duration_to_millis(timeout: Duration) -> u64 {
    u64::try_from(timeout.as_millis().min(u128::from(u64::MAX))).unwrap_or(u64::MAX)
}
