import MailKit

public typealias MailMessageActionDecisionCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

public typealias MailMessageActionHeadersCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

final class RustMessageActionHandler: NSObject, MEMessageActionHandler {
    fileprivate let decideCallback: MailMessageActionDecisionCallback
    fileprivate let requiredHeadersCallback: MailMessageActionHeadersCallback?
    fileprivate let releaseCallback: MailReleaseUserDataCallback
    fileprivate let userData: UnsafeMutableRawPointer?

    init(
        decideCallback: @escaping MailMessageActionDecisionCallback,
        requiredHeadersCallback: MailMessageActionHeadersCallback?,
        releaseCallback: @escaping MailReleaseUserDataCallback,
        userData: UnsafeMutableRawPointer?
    ) {
        self.decideCallback = decideCallback
        self.requiredHeadersCallback = requiredHeadersCallback
        self.releaseCallback = releaseCallback
        self.userData = userData
    }

    deinit {
        releaseCallback(userData)
    }

    func decisionBox(from messageBox: UnsafeMutableRawPointer) throws -> MailMessageActionDecisionBox? {
        var errorPtr: UnsafeMutablePointer<CChar>?
        let result = decideCallback(messageBox, userData, &errorPtr)
        if let message = mailTakeCString(errorPtr) {
            mail_object_release(result)
            throw MailBridgeError.frameworkError(message)
        }
        guard let result else {
            return nil
        }
        guard let decision = mailConsumeObject(result, as: MailMessageActionDecisionBox.self) else {
            throw MailBridgeError.frameworkError("message-action callback returned an invalid decision box")
        }
        return decision
    }

    fileprivate func decisionBox(for snapshot: MailMessageSnapshot) throws -> MailMessageActionDecisionBox? {
        let messageBox = try mailSnapshotBoxPointer(for: snapshot)
        defer { mail_object_release(messageBox) }
        return try decisionBox(from: messageBox)
    }

    fileprivate func requiredHeadersList() throws -> [String] {
        guard let requiredHeadersCallback else {
            return []
        }
        var errorPtr: UnsafeMutablePointer<CChar>?
        let result = requiredHeadersCallback(userData, &errorPtr)
        if let message = mailTakeCString(errorPtr) {
            throw MailBridgeError.frameworkError(message)
        }
        guard let result else {
            return []
        }
        return try mailDecodeConsumedJSONBox(result, as: [String].self)
    }

    func decideAction(
        for message: MEMessage,
        completionHandler: @escaping (MEMessageActionDecision?) -> Void
    ) {
        completionHandler(try? decisionBox(for: mailSnapshot(message))?.decision)
    }

    var requiredHeaders: [String] {
        (try? requiredHeadersList()) ?? []
    }
}

@_cdecl("mail_message_action_extension_new")
public func mail_message_action_extension_new(
    _ decideCallback: MailMessageActionDecisionCallback?,
    _ requiredHeadersCallback: MailMessageActionHeadersCallback?,
    _ releaseCallback: MailReleaseUserDataCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ outHandler: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let decideCallback, let releaseCallback else {
        mailSetError(errorOut, "message action callbacks were incomplete")
        return MAIL_STATUS_INVALID_ARGUMENT
    }
    outHandler?.pointee = mailRetainedPointer(
        RustMessageActionHandler(
            decideCallback: decideCallback,
            requiredHeadersCallback: requiredHeadersCallback,
            releaseCallback: releaseCallback,
            userData: userData
        )
    )
    return MAIL_STATUS_OK
}

@_cdecl("mail_message_action_extension_test_decide_action")
public func mail_message_action_extension_test_decide_action(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageActionHandler.self),
              let messageBox
        else {
            throw MailBridgeError.invalidArgument("message-action test inputs were null")
        }
        defer { mail_object_release(messageBox) }
        if let box = try handler.decisionBox(from: messageBox) {
            return mailRetainedPointer(box)
        }
        return nil
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_message_action_extension_test_required_headers")
public func mail_message_action_extension_test_required_headers(
    _ handler: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageActionHandler.self) else {
            throw MailBridgeError.invalidArgument("message-action handler pointer was null")
        }
        return try mailJSONBoxPointer(for: handler.requiredHeadersList())
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}
