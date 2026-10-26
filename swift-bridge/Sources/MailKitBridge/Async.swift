import Foundation
import MailKit

@_cdecl("mail_message_action_decision_async")
public func mail_message_action_decision_async(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    defer { mail_object_release(messageBox) }
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageActionHandler.self), let messageBox else {
            throw MailBridgeError.invalidArgument("message-action async inputs were null")
        }
        if let box = try handler.decisionBox(from: messageBox) {
            cb(mailRetainedPointer(box), nil, ctx)
        } else {
            cb(nil, nil, ctx)
        }
    } catch {
        error.localizedDescription.withCString { errorCString in
            cb(nil, errorCString, ctx)
        }
    }
}

@_cdecl("mail_content_blocker_content_rules_async")
public func mail_content_blocker_content_rules_async(
    _ blocker: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    do {
        guard let blocker = mailTakeObject(blocker, as: RustContentBlocker.self) else {
            throw MailBridgeError.invalidArgument("content blocker pointer was null")
        }
        cb(try mailJSONBoxPointer(for: blocker.contentRulesBytes()), nil, ctx)
    } catch {
        error.localizedDescription.withCString { errorCString in
            cb(nil, errorCString, ctx)
        }
    }
}
