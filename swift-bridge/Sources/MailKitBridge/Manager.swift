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
public func mail_extension_manager_reload_visible_messages(
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
public func mail_extension_manager_reload_content_blocker(
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

@_cdecl("mail_extension_manager_reload_visible_messages_async")
public func mail_extension_manager_reload_visible_messages_async(
    _ cb: @escaping @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    MEExtensionManager.reloadVisibleMessages { error in
        guard let error = error as NSError? else {
            cb(nil, nil, ctx)
            return
        }
        do {
            let payload = try mailJSONString(from: mailNSErrorPayload(error))
            payload.withCString { errorCString in
                cb(nil, errorCString, ctx)
            }
        } catch {
            error.localizedDescription.withCString { errorCString in
                cb(nil, errorCString, ctx)
            }
        }
    }
}

@_cdecl("mail_extension_manager_reload_content_blocker_async")
public func mail_extension_manager_reload_content_blocker_async(
    _ identifier: UnsafePointer<CChar>?,
    _ cb: @escaping @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    guard let identifier else {
        "MailKit content blocker identifier was null".withCString { errorCString in
            cb(nil, errorCString, ctx)
        }
        return
    }
    let contentBlockerIdentifier = String(cString: identifier)
    MEExtensionManager.reloadContentBlocker(withIdentifier: contentBlockerIdentifier) { error in
        guard let error = error as NSError? else {
            cb(nil, nil, ctx)
            return
        }
        do {
            let payload = try mailJSONString(from: mailNSErrorPayload(error))
            payload.withCString { errorCString in
                cb(nil, errorCString, ctx)
            }
        } catch {
            error.localizedDescription.withCString { errorCString in
                cb(nil, errorCString, ctx)
            }
        }
    }
}
