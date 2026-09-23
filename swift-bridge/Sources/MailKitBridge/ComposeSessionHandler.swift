import Foundation
import MailKit

struct MailComposeContextSnapshot: Codable {
    let contextId: String
    var originalMessage: MailMessageSnapshot?
    let action: String
    let isEncrypted: Bool
    let shouldEncrypt: Bool
    let isSigned: Bool
    let shouldSign: Bool
}

extension MailComposeContextSnapshot: MailPayloadSnapshot {
    func payloads(prefix: String) -> [String: Data] {
        originalMessage?.payloads(prefix: prefix + "originalMessage.") ?? [:]
    }

    mutating func adoptPayloads(from carrier: MailPayloadCarrier, prefix: String) {
        originalMessage?.adoptPayloads(from: carrier, prefix: prefix + "originalMessage.")
    }
}

struct MailComposeSessionSnapshot: Codable {
    let sessionId: String
    var mailMessage: MailMessageSnapshot
    var composeContext: MailComposeContextSnapshot
}

extension MailComposeSessionSnapshot: MailPayloadSnapshot {
    func payloads(prefix: String) -> [String: Data] {
        mailMessage.payloads(prefix: prefix + "mailMessage.").merging(
            composeContext.payloads(prefix: prefix + "composeContext.")
        ) { current, _ in current }
    }

    mutating func adoptPayloads(from carrier: MailPayloadCarrier, prefix: String) {
        mailMessage.adoptPayloads(from: carrier, prefix: prefix + "mailMessage.")
        composeContext.adoptPayloads(from: carrier, prefix: prefix + "composeContext.")
    }
}

struct MailAddressAnnotationEntry: Codable {
    let rawString: String
    let kind: String
    let localizedDescription: String
}

final class MailComposeSessionBox: NSObject, MailPayloadCarrier {
    let snapshot: MailComposeSessionSnapshot
    let session: MEComposeSession?
    private let payloads: [String: Data]
    private(set) var reloadCount = 0

    init(session: MEComposeSession) {
        let snapshot = mailSnapshot(session)
        self.snapshot = snapshot
        self.session = session
        self.payloads = snapshot.payloads(prefix: "")
    }

    init(snapshot: MailComposeSessionSnapshot) {
        self.snapshot = snapshot
        self.session = nil
        self.payloads = snapshot.payloads(prefix: "")
    }

    func mailPayload(forKey key: String) -> Data? {
        payloads[key]
    }

    func reload() {
        if let session {
            session.reload()
        } else {
            reloadCount += 1
        }
    }

    func candidateAddresses() -> [MEEmailAddress] {
        if let session {
            return session.mailMessage.allRecipientAddresses
        }
        return snapshot.mailMessage.allRecipientAddresses.map {
            MEEmailAddress(rawString: $0.rawString)
        }
    }
}

func mailComposeUserActionName(_ action: MEComposeUserAction) -> String {
    switch action {
    case .newMessage:
        return "newMessage"
    case .reply:
        return "reply"
    case .replyAll:
        return "replyAll"
    case .forward:
        return "forward"
    @unknown default:
        return "newMessage"
    }
}

func mailSnapshot(_ context: MEComposeContext) -> MailComposeContextSnapshot {
    MailComposeContextSnapshot(
        contextId: context.contextID.uuidString,
        originalMessage: context.originalMessage.map(mailSnapshot),
        action: mailComposeUserActionName(context.action),
        isEncrypted: context.isEncrypted,
        shouldEncrypt: context.shouldEncrypt,
        isSigned: context.isSigned,
        shouldSign: context.shouldSign
    )
}

func mailSnapshot(_ session: MEComposeSession) -> MailComposeSessionSnapshot {
    MailComposeSessionSnapshot(
        sessionId: session.sessionID.uuidString,
        mailMessage: mailSnapshot(session.mailMessage),
        composeContext: mailSnapshot(session.composeContext)
    )
}

private func mailAddressAnnotation(from entry: MailAddressAnnotationEntry) -> MEAddressAnnotation {
    switch entry.kind {
    case "error":
        return MEAddressAnnotation.error(withLocalizedDescription: entry.localizedDescription)
    case "success":
        return MEAddressAnnotation.success(withLocalizedDescription: entry.localizedDescription)
    default:
        return MEAddressAnnotation.warning(withLocalizedDescription: entry.localizedDescription)
    }
}

private func mailAnnotationDictionary(
    entries: [MailAddressAnnotationEntry],
    sessionBox: MailComposeSessionBox
) -> [MEEmailAddress: MEAddressAnnotation] {
    let candidates = Dictionary(uniqueKeysWithValues: sessionBox.candidateAddresses().map {
        ($0.rawString, $0)
    })
    return Dictionary(uniqueKeysWithValues: entries.map { entry in
        (
            candidates[entry.rawString] ?? MEEmailAddress(rawString: entry.rawString),
            mailAddressAnnotation(from: entry)
        )
    })
}

typealias MailComposeSessionVoidCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32

typealias MailComposeSessionControllerCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

typealias MailComposeSessionJSONCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

typealias MailReleaseUserDataCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void

final class RustComposeSessionHandler: NSObject, MEComposeSessionHandler {
    fileprivate let didBeginCallback: MailComposeSessionVoidCallback
    fileprivate let didEndCallback: MailComposeSessionVoidCallback
    fileprivate let viewControllerCallback: MailComposeSessionControllerCallback
    fileprivate let annotateCallback: MailComposeSessionJSONCallback?
    fileprivate let canSendCallback: MailComposeSessionJSONCallback?
    fileprivate let additionalHeadersCallback: MailComposeSessionJSONCallback?
    fileprivate let releaseCallback: MailReleaseUserDataCallback
    fileprivate let userData: UnsafeMutableRawPointer?

    init(
        didBeginCallback: @escaping MailComposeSessionVoidCallback,
        didEndCallback: @escaping MailComposeSessionVoidCallback,
        viewControllerCallback: @escaping MailComposeSessionControllerCallback,
        annotateCallback: MailComposeSessionJSONCallback?,
        canSendCallback: MailComposeSessionJSONCallback?,
        additionalHeadersCallback: MailComposeSessionJSONCallback?,
        releaseCallback: @escaping MailReleaseUserDataCallback,
        userData: UnsafeMutableRawPointer?
    ) {
        self.didBeginCallback = didBeginCallback
        self.didEndCallback = didEndCallback
        self.viewControllerCallback = viewControllerCallback
        self.annotateCallback = annotateCallback
        self.canSendCallback = canSendCallback
        self.additionalHeadersCallback = additionalHeadersCallback
        self.releaseCallback = releaseCallback
        self.userData = userData
    }

    deinit {
        releaseCallback(userData)
    }

    fileprivate func invokeVoid(
        callback: MailComposeSessionVoidCallback,
        sessionBox: MailComposeSessionBox
    ) throws {
        var errorPtr: UnsafeMutablePointer<CChar>?
        let status = mailWithRetainedPointer(sessionBox) { ptr in
            callback(ptr, userData, &errorPtr)
        }
        guard status == MAIL_STATUS_OK else {
            throw MailBridgeError.frameworkError(
                mailTakeCString(errorPtr) ?? "MailKit compose-session callback failed with status \(status)"
            )
        }
    }

    fileprivate func invokeJSON<T: Decodable>(
        callback: MailComposeSessionJSONCallback?,
        sessionBox: MailComposeSessionBox,
        as type: T.Type
    ) throws -> T? {
        guard let callback else {
            return nil
        }
        var errorPtr: UnsafeMutablePointer<CChar>?
        let result = mailWithRetainedPointer(sessionBox) { ptr in
            callback(ptr, userData, &errorPtr)
        }
        if let message = mailTakeCString(errorPtr) {
            throw MailBridgeError.frameworkError(message)
        }
        guard let result else {
            return nil
        }
        return try mailDecodeConsumedJSONBox(result, as: type)
    }

    fileprivate func invokeViewController(sessionBox: MailComposeSessionBox) throws -> MEExtensionViewController {
        var errorPtr: UnsafeMutablePointer<CChar>?
        let controllerRaw = mailWithRetainedPointer(sessionBox) { ptr in
            viewControllerCallback(ptr, userData, &errorPtr)
        }
        if let message = mailTakeCString(errorPtr) {
            throw MailBridgeError.frameworkError(message)
        }
        guard let controller = mailConsumeObject(controllerRaw, as: MEExtensionViewController.self) else {
            throw MailBridgeError.frameworkError("compose-session view controller callback returned null")
        }
        return controller
    }

    func annotationEntries(for sessionBox: MailComposeSessionBox) throws -> [MailAddressAnnotationEntry] {
        try invokeJSON(callback: annotateCallback, sessionBox: sessionBox, as: [MailAddressAnnotationEntry].self) ?? []
    }

    func canSendError(for sessionBox: MailComposeSessionBox) throws -> NSError? {
        guard let payload = try invokeJSON(callback: canSendCallback, sessionBox: sessionBox, as: NSErrorPayload.self) else {
            return nil
        }
        return NSError(
            domain: payload.domain,
            code: Int(payload.code),
            userInfo: [NSLocalizedDescriptionKey: payload.localizedDescription]
        )
    }

    func additionalHeaders(for sessionBox: MailComposeSessionBox) throws -> [String: [String]] {
        try invokeJSON(callback: additionalHeadersCallback, sessionBox: sessionBox, as: [String: [String]].self) ?? [:]
    }

    func mailComposeSessionDidBegin(_ session: MEComposeSession) {
        try? invokeVoid(callback: didBeginCallback, sessionBox: MailComposeSessionBox(session: session))
    }

    func mailComposeSessionDidEnd(_ session: MEComposeSession) {
        try? invokeVoid(callback: didEndCallback, sessionBox: MailComposeSessionBox(session: session))
    }

    func viewController(for session: MEComposeSession) -> MEExtensionViewController {
        (try? invokeViewController(sessionBox: MailComposeSessionBox(session: session))) ?? MEExtensionViewController()
    }

    func annotateAddressesForSession(
        _ session: MEComposeSession,
        completion: @escaping ([MEEmailAddress: MEAddressAnnotation]) -> Void
    ) {
        let sessionBox = MailComposeSessionBox(session: session)
        let entries = (try? annotationEntries(for: sessionBox)) ?? []
        completion(mailAnnotationDictionary(entries: entries, sessionBox: sessionBox))
    }

    func allowMessageSendForSession(
        _ session: MEComposeSession,
        completion: @escaping (Error?) -> Void
    ) {
        let sessionBox = MailComposeSessionBox(session: session)
        do {
            completion(try canSendError(for: sessionBox))
        } catch {
            completion(error)
        }
    }

    func additionalHeaders(for session: MEComposeSession) -> [String: [String]] {
        (try? additionalHeaders(for: MailComposeSessionBox(session: session))) ?? [:]
    }
}

@_cdecl("mail_compose_session_new")
func mail_compose_session_new(
    _ sessionBox: UnsafeMutableRawPointer?,
    _ outSession: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let snapshot = try mailDecodeSnapshotBox(sessionBox, as: MailComposeSessionSnapshot.self)
        outSession?.pointee = mailRetainedPointer(MailComposeSessionBox(snapshot: snapshot))
        return MAIL_STATUS_OK
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return MAIL_STATUS_INVALID_ARGUMENT
    }
}

@_cdecl("mail_compose_session_copy_json")
func mail_compose_session_copy_json(_ session: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let session = mailTakeObject(session, as: MailComposeSessionBox.self) else {
        return nil
    }
    do {
        return mailCopyString(try mailJSONString(from: session.snapshot))
    } catch {
        return nil
    }
}

@_cdecl("mail_compose_session_reload")
func mail_compose_session_reload(
    _ session: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let session = mailTakeObject(session, as: MailComposeSessionBox.self) else {
        mailSetError(errorOut, "compose session box was null")
        return MAIL_STATUS_INVALID_ARGUMENT
    }
    session.reload()
    return MAIL_STATUS_OK
}

@_cdecl("mail_compose_session_reload_count")
func mail_compose_session_reload_count(_ session: UnsafeMutableRawPointer?) -> Int {
    mailTakeObject(session, as: MailComposeSessionBox.self)?.reloadCount ?? 0
}

@_cdecl("mail_compose_session_handler_new")
func mail_compose_session_handler_new(
    _ didBeginCallback: MailComposeSessionVoidCallback?,
    _ didEndCallback: MailComposeSessionVoidCallback?,
    _ viewControllerCallback: MailComposeSessionControllerCallback?,
    _ annotateCallback: MailComposeSessionJSONCallback?,
    _ canSendCallback: MailComposeSessionJSONCallback?,
    _ additionalHeadersCallback: MailComposeSessionJSONCallback?,
    _ releaseCallback: MailReleaseUserDataCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ outHandler: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let didBeginCallback, let didEndCallback, let viewControllerCallback, let releaseCallback else {
        mailSetError(errorOut, "compose session handler callbacks were incomplete")
        return MAIL_STATUS_INVALID_ARGUMENT
    }
    outHandler?.pointee = mailRetainedPointer(
        RustComposeSessionHandler(
            didBeginCallback: didBeginCallback,
            didEndCallback: didEndCallback,
            viewControllerCallback: viewControllerCallback,
            annotateCallback: annotateCallback,
            canSendCallback: canSendCallback,
            additionalHeadersCallback: additionalHeadersCallback,
            releaseCallback: releaseCallback,
            userData: userData
        )
    )
    return MAIL_STATUS_OK
}

@_cdecl("mail_compose_session_handler_test_did_begin")
func mail_compose_session_handler_test_did_begin(
    _ handler: UnsafeMutableRawPointer?,
    _ session: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let handler = mailTakeObject(handler, as: RustComposeSessionHandler.self),
              let session = mailConsumeObject(session, as: MailComposeSessionBox.self)
        else {
            throw MailBridgeError.invalidArgument("compose session test inputs were null")
        }
        try handler.invokeVoid(callback: handler.didBeginCallback, sessionBox: session)
        return MAIL_STATUS_OK
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return MAIL_STATUS_FRAMEWORK_ERROR
    }
}

@_cdecl("mail_compose_session_handler_test_did_end")
func mail_compose_session_handler_test_did_end(
    _ handler: UnsafeMutableRawPointer?,
    _ session: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let handler = mailTakeObject(handler, as: RustComposeSessionHandler.self),
              let session = mailConsumeObject(session, as: MailComposeSessionBox.self)
        else {
            throw MailBridgeError.invalidArgument("compose session test inputs were null")
        }
        try handler.invokeVoid(callback: handler.didEndCallback, sessionBox: session)
        return MAIL_STATUS_OK
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return MAIL_STATUS_FRAMEWORK_ERROR
    }
}

@_cdecl("mail_compose_session_handler_test_view_controller")
func mail_compose_session_handler_test_view_controller(
    _ handler: UnsafeMutableRawPointer?,
    _ session: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustComposeSessionHandler.self),
              let session = mailConsumeObject(session, as: MailComposeSessionBox.self)
        else {
            throw MailBridgeError.invalidArgument("compose session test inputs were null")
        }
        let controller = try handler.invokeViewController(sessionBox: session)
        return mailRetainedPointer(controller)
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_compose_session_handler_test_annotate_addresses")
func mail_compose_session_handler_test_annotate_addresses(
    _ handler: UnsafeMutableRawPointer?,
    _ session: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustComposeSessionHandler.self),
              let session = mailConsumeObject(session, as: MailComposeSessionBox.self)
        else {
            throw MailBridgeError.invalidArgument("compose session test inputs were null")
        }
        return try mailJSONBoxPointer(for: handler.annotationEntries(for: session))
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_compose_session_handler_test_can_send_message")
func mail_compose_session_handler_test_can_send_message(
    _ handler: UnsafeMutableRawPointer?,
    _ session: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustComposeSessionHandler.self),
              let session = mailConsumeObject(session, as: MailComposeSessionBox.self)
        else {
            throw MailBridgeError.invalidArgument("compose session test inputs were null")
        }
        guard let error = try handler.canSendError(for: session) else {
            return nil
        }
        return try mailJSONBoxPointer(for: mailNSErrorPayload(error))
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_compose_session_handler_test_additional_headers")
func mail_compose_session_handler_test_additional_headers(
    _ handler: UnsafeMutableRawPointer?,
    _ session: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustComposeSessionHandler.self),
              let session = mailConsumeObject(session, as: MailComposeSessionBox.self)
        else {
            throw MailBridgeError.invalidArgument("compose session test inputs were null")
        }
        return try mailJSONBoxPointer(for: handler.additionalHeaders(for: session))
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_compose_session_handler_annotate_addresses_async")
func mail_compose_session_handler_annotate_addresses_async(
    _ handler: UnsafeMutableRawPointer?,
    _ session: UnsafeMutableRawPointer?,
    _ cb: @escaping @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    do {
        guard let handler = mailTakeObject(handler, as: RustComposeSessionHandler.self),
              let session = mailConsumeObject(session, as: MailComposeSessionBox.self)
        else {
            throw MailBridgeError.invalidArgument("compose session async annotate inputs were null")
        }
        do {
            cb(try mailJSONBoxPointer(for: handler.annotationEntries(for: session)), nil, ctx)
        } catch {
            error.localizedDescription.withCString { errorCString in
                cb(nil, errorCString, ctx)
            }
        }
    } catch {
        error.localizedDescription.withCString { errorCString in
            cb(nil, errorCString, ctx)
        }
    }
}

@_cdecl("mail_compose_session_handler_allow_message_send_async")
func mail_compose_session_handler_allow_message_send_async(
    _ handler: UnsafeMutableRawPointer?,
    _ session: UnsafeMutableRawPointer?,
    _ cb: @escaping @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    do {
        guard let handler = mailTakeObject(handler, as: RustComposeSessionHandler.self),
              let session = mailConsumeObject(session, as: MailComposeSessionBox.self)
        else {
            throw MailBridgeError.invalidArgument("compose session async send inputs were null")
        }
        do {
            if let sendError = try handler.canSendError(for: session) {
                do {
                    let payload = try mailJSONString(from: mailNSErrorPayload(sendError))
                    payload.withCString { errorCString in
                        cb(nil, errorCString, ctx)
                    }
                } catch {
                    sendError.localizedDescription.withCString { errorCString in
                        cb(nil, errorCString, ctx)
                    }
                }
            } else {
                cb(nil, nil, ctx)
            }
        } catch {
            error.localizedDescription.withCString { errorCString in
                cb(nil, errorCString, ctx)
            }
        }
    } catch {
        error.localizedDescription.withCString { errorCString in
            cb(nil, errorCString, ctx)
        }
    }
}
