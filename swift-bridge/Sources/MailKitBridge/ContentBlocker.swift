import Foundation
import MailKit

public typealias MailContentBlockerJSONCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

final class RustContentBlocker: NSObject, MEContentBlocker {
    fileprivate let contentRulesCallback: MailContentBlockerJSONCallback
    fileprivate let releaseCallback: MailReleaseUserDataCallback
    fileprivate let userData: UnsafeMutableRawPointer?

    init(
        contentRulesCallback: @escaping MailContentBlockerJSONCallback,
        releaseCallback: @escaping MailReleaseUserDataCallback,
        userData: UnsafeMutableRawPointer?
    ) {
        self.contentRulesCallback = contentRulesCallback
        self.releaseCallback = releaseCallback
        self.userData = userData
    }

    deinit {
        releaseCallback(userData)
    }

    func contentRulesBytes() throws -> [UInt8] {
        var errorPtr: UnsafeMutablePointer<CChar>?
        let result = contentRulesCallback(userData, &errorPtr)
        if let message = mailTakeCString(errorPtr) {
            throw MailBridgeError.frameworkError(message)
        }
        guard let result else {
            return []
        }
        return try mailDecodeConsumedJSONBox(result, as: [UInt8].self)
    }

    func contentRulesJSON() -> Data {
        (try? Data(contentRulesBytes())) ?? Data()
    }
}

@_cdecl("mail_content_blocker_new")
public func mail_content_blocker_new(
    _ contentRulesCallback: MailContentBlockerJSONCallback?,
    _ releaseCallback: MailReleaseUserDataCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ outBlocker: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let contentRulesCallback, let releaseCallback else {
        mailSetError(errorOut, "content blocker callbacks were incomplete")
        return MAIL_STATUS_INVALID_ARGUMENT
    }
    outBlocker?.pointee = mailRetainedPointer(
        RustContentBlocker(
            contentRulesCallback: contentRulesCallback,
            releaseCallback: releaseCallback,
            userData: userData
        )
    )
    return MAIL_STATUS_OK
}

@_cdecl("mail_content_blocker_test_content_rules_json")
public func mail_content_blocker_test_content_rules_json(
    _ blocker: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let blocker = mailTakeObject(blocker, as: RustContentBlocker.self) else {
            throw MailBridgeError.invalidArgument("content blocker pointer was null")
        }
        return try mailJSONBoxPointer(for: blocker.contentRulesBytes())
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}
