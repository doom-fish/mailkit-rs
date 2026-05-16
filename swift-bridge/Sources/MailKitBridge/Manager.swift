import Dispatch
import Foundation
import MailKit

private func mailDispatchTime(timeoutMilliseconds: UInt64) -> DispatchTime {
    let clamped = min(timeoutMilliseconds, UInt64(Int.max))
    return .now() + .milliseconds(Int(clamped))
}

private func mailWaitForNSErrorCompletion(
    timeoutMilliseconds: UInt64,
    errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    operation: (@escaping (NSError?) -> Void) -> Void
) -> Int32 {
    if Thread.isMainThread {
        let deadline = Date().addingTimeInterval(Double(timeoutMilliseconds) / 1000.0)
        var completed = false
        var capturedError: NSError?
        operation { error in
            capturedError = error
            completed = true
        }
        while !completed && Date() < deadline {
            _ = RunLoop.current.run(mode: .default, before: Date().addingTimeInterval(0.01))
        }
        guard completed else {
            mailSetError(errorOut, "MailKit operation timed out")
            return MAIL_STATUS_TIMED_OUT
        }
        if let capturedError {
            mailSetErrorJSON(errorOut, mailNSErrorPayload(capturedError))
            return MAIL_STATUS_OPERATION_FAILED
        }
        return MAIL_STATUS_OK
    }

    let semaphore = DispatchSemaphore(value: 0)
    var capturedError: NSError?
    operation { error in
        capturedError = error
        semaphore.signal()
    }
    if semaphore.wait(timeout: mailDispatchTime(timeoutMilliseconds: timeoutMilliseconds)) == .timedOut {
        mailSetError(errorOut, "MailKit operation timed out")
        return MAIL_STATUS_TIMED_OUT
    }
    if let capturedError {
        mailSetErrorJSON(errorOut, mailNSErrorPayload(capturedError))
        return MAIL_STATUS_OPERATION_FAILED
    }
    return MAIL_STATUS_OK
}

@_cdecl("mail_extension_manager_reload_visible_messages")
func mail_extension_manager_reload_visible_messages(
    _ timeoutMilliseconds: UInt64,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    mailWaitForNSErrorCompletion(timeoutMilliseconds: timeoutMilliseconds, errorOut: errorOut) {
        completion in
        MEExtensionManager.reloadVisibleMessages { error in
            completion(error as NSError?)
        }
    }
}

@_cdecl("mail_extension_manager_reload_content_blocker")
func mail_extension_manager_reload_content_blocker(
    _ identifier: UnsafePointer<CChar>?,
    _ timeoutMilliseconds: UInt64,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let identifier else {
        mailSetError(errorOut, "MailKit content blocker identifier was null")
        return MAIL_STATUS_INVALID_ARGUMENT
    }
    return mailWaitForNSErrorCompletion(
        timeoutMilliseconds: timeoutMilliseconds,
        errorOut: errorOut
    ) { completion in
        MEExtensionManager.reloadContentBlocker(
            withIdentifier: String(cString: identifier)
        ) { error in
            completion(error as NSError?)
        }
    }
}
