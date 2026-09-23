use std::ptr;
use std::time::Duration;

use crate::error::{from_swift, MailKitError};
use crate::ffi;
use crate::private::to_cstring;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Wraps `MEExtensionManager` utility operations.
pub struct ExtensionManager;

impl ExtensionManager {
    /// Requests MailKit to reload messages visible in Mail.
    pub fn reload_visible_messages() -> Result<(), MailKitError> {
        Self::reload_visible_messages_with_timeout(DEFAULT_TIMEOUT)
    }

    /// Requests MailKit to reload visible messages with a custom timeout.
    pub fn reload_visible_messages_with_timeout(timeout: Duration) -> Result<(), MailKitError> {
        let mut error_ptr = ptr::null_mut();
        let status = unsafe {
            ffi::mail_extension_manager_reload_visible_messages(
                duration_to_millis(timeout),
                &raw mut error_ptr,
            )
        };
        if status != ffi::status::OK {
            return Err(from_swift(status, error_ptr));
        }
        Ok(())
    }

    /// Requests MailKit to reload the named content blocker.
    pub fn reload_content_blocker(identifier: &str) -> Result<(), MailKitError> {
        Self::reload_content_blocker_with_timeout(identifier, DEFAULT_TIMEOUT)
    }

    /// Requests MailKit to reload the named content blocker with a custom timeout.
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
                &raw mut error_ptr,
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
