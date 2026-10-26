import Foundation
import MailKit

struct MailMessageSignerSnapshot: Codable {
    let emailAddresses: [MailEmailAddressSnapshot]
    let label: String
    let context: [UInt8]?
}

func mailSnapshot(_ signer: MEMessageSigner) -> MailMessageSignerSnapshot {
    let contextBytes = signer.context.isEmpty ? nil : Array(signer.context)
    return MailMessageSignerSnapshot(
        emailAddresses: signer.emailAddresses.map(mailSnapshot),
        label: signer.label,
        context: contextBytes
    )
}

public typealias MailMessageSecurityDecodeCallback = @convention(c) (
    UnsafePointer<UInt8>?,
    Int,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

public typealias MailMessageSecurityEncodeCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

public typealias MailMessageSecurityControllerCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

struct MailRemoteContentSnapshot: Encodable {
    let shouldBlock: Bool
    let localizedReason: String?
}

struct MailSecurityInformationSnapshot: Encodable {
    let signers: [MailMessageSignerSnapshot]
    let isEncrypted: Bool
    let signingError: NSErrorPayload?
    let encryptionError: NSErrorPayload?
    let remoteContent: MailRemoteContentSnapshot
}

struct MailFailedDecodedMessageSnapshot: Encodable {
    let securityInformation: MailSecurityInformationSnapshot
}

struct MailCapabilitiesSnapshot: Encodable {
    let canSign: Bool
    let canEncrypt: Bool
}

struct MailFailedEncodingStatusSnapshot: Encodable {
    let capabilities: MailCapabilitiesSnapshot
    let securityError: NSErrorPayload
    let addressesFailingEncryption: [MailEmailAddressSnapshot]
}

struct MailFailedEncodingResultSnapshot: Encodable {
    let signingError: NSErrorPayload?
    let encryptionError: NSErrorPayload?
}

func mailFailedDecodedMessageBox(_ description: String) -> MailDecodedMessageBox {
    let error = mailSecurityError(.decodingError, description)
    let payload = mailNSErrorPayload(error)
    let information = MEMessageSecurityInformation(
        signers: [],
        isEncrypted: false,
        signingError: error,
        encryptionError: error,
        shouldBlockRemoteContent: true,
        localizedRemoteContentBlockingReason: nil
    )
    let snapshot = MailFailedDecodedMessageSnapshot(
        securityInformation: MailSecurityInformationSnapshot(
            signers: [],
            isEncrypted: false,
            signingError: payload,
            encryptionError: payload,
            remoteContent: MailRemoteContentSnapshot(shouldBlock: true, localizedReason: nil)
        )
    )
    return MailDecodedMessageBox(
        message: MEDecodedMessage(data: nil, securityInformation: information, context: nil),
        json: (try? mailJSONString(from: snapshot)) ?? ""
    )
}

func mailFailedEncodingStatusBox(_ description: String) -> MailOutgoingMessageEncodingStatusBox {
    let error = mailSecurityError(.encodingError, description)
    let snapshot = MailFailedEncodingStatusSnapshot(
        capabilities: MailCapabilitiesSnapshot(canSign: false, canEncrypt: false),
        securityError: mailNSErrorPayload(error),
        addressesFailingEncryption: []
    )
    return MailOutgoingMessageEncodingStatusBox(
        status: MEOutgoingMessageEncodingStatus(
            canSign: false,
            canEncrypt: false,
            securityError: error,
            addressesFailingEncryption: []
        ),
        json: (try? mailJSONString(from: snapshot)) ?? ""
    )
}

func mailFailedEncodingResultBox(
    _ description: String,
    composeContext: MailComposeContextSnapshot
) -> MailMessageEncodingResultBox {
    let error = mailSecurityError(.encodingError, description)
    let signingError = composeContext.shouldSign || !composeContext.shouldEncrypt ? error : nil
    let encryptionError = composeContext.shouldEncrypt || !composeContext.shouldSign ? error : nil
    let snapshot = MailFailedEncodingResultSnapshot(
        signingError: signingError.map(mailNSErrorPayload),
        encryptionError: encryptionError.map(mailNSErrorPayload)
    )
    return MailMessageEncodingResultBox(
        result: MEMessageEncodingResult(
            encodedMessage: nil,
            signingError: signingError,
            encryptionError: encryptionError
        ),
        json: (try? mailJSONString(from: snapshot)) ?? ""
    )
}

func mailUnmetEncodingRequest(
    _ result: MEMessageEncodingResult,
    composeContext: MailComposeContextSnapshot
) -> String? {
    if composeContext.shouldSign, result.signingError == nil, result.encodedMessage?.isSigned != true {
        return "The message security extension did not sign a message that was meant to be signed."
    }
    if composeContext.shouldEncrypt, result.encryptionError == nil, result.encodedMessage?.isEncrypted != true {
        return "The message security extension did not encrypt a message that was meant to be encrypted."
    }
    return nil
}

final class RustMessageSecurityHandler: NSObject, MEMessageSecurityHandler {
    fileprivate let decodeCallback: MailMessageSecurityDecodeCallback
    fileprivate let encodingStatusCallback: MailMessageSecurityEncodeCallback
    fileprivate let encodeMessageCallback: MailMessageSecurityEncodeCallback
    fileprivate let signersViewControllerCallback: MailMessageSecurityControllerCallback?
    fileprivate let messageContextViewControllerCallback: MailMessageSecurityControllerCallback?
    fileprivate let primaryActionCallback: MailMessageSecurityControllerCallback?
    fileprivate let releaseCallback: MailReleaseUserDataCallback
    fileprivate let userData: UnsafeMutableRawPointer?

    init(
        decodeCallback: @escaping MailMessageSecurityDecodeCallback,
        encodingStatusCallback: @escaping MailMessageSecurityEncodeCallback,
        encodeMessageCallback: @escaping MailMessageSecurityEncodeCallback,
        signersViewControllerCallback: MailMessageSecurityControllerCallback?,
        messageContextViewControllerCallback: MailMessageSecurityControllerCallback?,
        primaryActionCallback: MailMessageSecurityControllerCallback?,
        releaseCallback: @escaping MailReleaseUserDataCallback,
        userData: UnsafeMutableRawPointer?
    ) {
        self.decodeCallback = decodeCallback
        self.encodingStatusCallback = encodingStatusCallback
        self.encodeMessageCallback = encodeMessageCallback
        self.signersViewControllerCallback = signersViewControllerCallback
        self.messageContextViewControllerCallback = messageContextViewControllerCallback
        self.primaryActionCallback = primaryActionCallback
        self.releaseCallback = releaseCallback
        self.userData = userData
    }

    deinit {
        releaseCallback(userData)
    }

    fileprivate func decodedMessageBox(for data: Data) -> MailDecodedMessageBox? {
        do {
            var errorPtr: UnsafeMutablePointer<CChar>?
            let result = data.withUnsafeBytes { raw in
                decodeCallback(raw.baseAddress?.assumingMemoryBound(to: UInt8.self), raw.count, userData, &errorPtr)
            }
            if let message = mailTakeCString(errorPtr) {
                mail_object_release(result)
                throw MailBridgeError.frameworkError(message)
            }
            guard let result else {
                return nil
            }
            guard let decoded = mailConsumeObject(result, as: MailDecodedMessageBox.self) else {
                throw MailBridgeError.frameworkError("message-security decode callback returned an invalid box")
            }
            return decoded
        } catch {
            return mailFailedDecodedMessageBox(error.localizedDescription)
        }
    }

    fileprivate func invokeEncodeCallback(
        _ callback: MailMessageSecurityEncodeCallback,
        message: MailMessageSnapshot,
        composeContext: MailComposeContextSnapshot
    ) throws -> UnsafeMutableRawPointer? {
        let messageBox = try mailSnapshotBoxPointer(for: message)
        defer { mail_object_release(messageBox) }
        let contextBox = try mailSnapshotBoxPointer(for: composeContext)
        defer { mail_object_release(contextBox) }
        var errorPtr: UnsafeMutablePointer<CChar>?
        let result = callback(messageBox, contextBox, userData, &errorPtr)
        if let message = mailTakeCString(errorPtr) {
            mail_object_release(result)
            throw MailBridgeError.frameworkError(message)
        }
        return result
    }

    fileprivate func encodingStatusBox(
        message: MailMessageSnapshot,
        composeContext: MailComposeContextSnapshot
    ) -> MailOutgoingMessageEncodingStatusBox {
        do {
            let result = try invokeEncodeCallback(
                encodingStatusCallback,
                message: message,
                composeContext: composeContext
            )
            guard let status = mailConsumeObject(result, as: MailOutgoingMessageEncodingStatusBox.self) else {
                throw MailBridgeError.frameworkError("message-security status callback returned null")
            }
            return status
        } catch {
            return mailFailedEncodingStatusBox(error.localizedDescription)
        }
    }

    fileprivate func encodingResultBox(
        message: MailMessageSnapshot,
        composeContext: MailComposeContextSnapshot
    ) -> MailMessageEncodingResultBox {
        do {
            let result = try invokeEncodeCallback(
                encodeMessageCallback,
                message: message,
                composeContext: composeContext
            )
            guard let encodingResult = mailConsumeObject(result, as: MailMessageEncodingResultBox.self) else {
                throw MailBridgeError.frameworkError("message-security encode callback returned null")
            }
            if let unmet = mailUnmetEncodingRequest(encodingResult.result, composeContext: composeContext) {
                return mailFailedEncodingResultBox(unmet, composeContext: composeContext)
            }
            return encodingResult
        } catch {
            return mailFailedEncodingResultBox(error.localizedDescription, composeContext: composeContext)
        }
    }

    fileprivate func controller(
        callback: MailMessageSecurityControllerCallback?,
        jsonValue: some Encodable
    ) throws -> MEExtensionViewController? {
        guard let callback else {
            return nil
        }
        let box = try mailJSONBoxPointer(for: jsonValue)
        defer { mail_object_release(box) }
        var errorPtr: UnsafeMutablePointer<CChar>?
        let result = callback(box, userData, &errorPtr)
        if let message = mailTakeCString(errorPtr) {
            mail_object_release(result)
            throw MailBridgeError.frameworkError(message)
        }
        guard let result else {
            return nil
        }
        guard let controller = mailConsumeObject(result, as: MEExtensionViewController.self) else {
            throw MailBridgeError.frameworkError("message-security controller callback returned an invalid controller")
        }
        return controller
    }

    func decodedMessage(forMessageData data: Data) -> MEDecodedMessage? {
        decodedMessageBox(for: data)?.message
    }

    func getEncodingStatus(
        for message: MEMessage,
        composeContext: MEComposeContext,
        completionHandler: @escaping (MEOutgoingMessageEncodingStatus) -> Void
    ) {
        completionHandler(
            encodingStatusBox(
                message: mailSnapshot(message),
                composeContext: mailSnapshot(composeContext)
            ).status
        )
    }

    func encode(
        _ message: MEMessage,
        composeContext: MEComposeContext,
        completionHandler: @escaping (MEMessageEncodingResult) -> Void
    ) {
        completionHandler(
            encodingResultBox(
                message: mailSnapshot(message),
                composeContext: mailSnapshot(composeContext)
            ).result
        )
    }

    func extensionViewController(signers: [MEMessageSigner]) -> MEExtensionViewController? {
        try? controller(
            callback: signersViewControllerCallback,
            jsonValue: signers.map(mailSnapshot)
        )
    }

    func extensionViewController(messageContext: Data) -> MEExtensionViewController? {
        try? controller(
            callback: messageContextViewControllerCallback,
            jsonValue: Array(messageContext)
        )
    }

    func primaryActionClicked(
        forMessageContext context: Data,
        completionHandler: @escaping (MEExtensionViewController?) -> Void
    ) {
        completionHandler(try? controller(callback: primaryActionCallback, jsonValue: Array(context)))
    }
}

@_cdecl("mail_message_security_handler_new")
public func mail_message_security_handler_new(
    _ decodeCallback: MailMessageSecurityDecodeCallback?,
    _ encodingStatusCallback: MailMessageSecurityEncodeCallback?,
    _ encodeMessageCallback: MailMessageSecurityEncodeCallback?,
    _ signersViewControllerCallback: MailMessageSecurityControllerCallback?,
    _ messageContextViewControllerCallback: MailMessageSecurityControllerCallback?,
    _ primaryActionCallback: MailMessageSecurityControllerCallback?,
    _ releaseCallback: MailReleaseUserDataCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ outHandler: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let decodeCallback,
          let encodingStatusCallback,
          let encodeMessageCallback,
          let releaseCallback
    else {
        mailSetError(errorOut, "message security callbacks were incomplete")
        return MAIL_STATUS_INVALID_ARGUMENT
    }
    outHandler?.pointee = mailRetainedPointer(
        RustMessageSecurityHandler(
            decodeCallback: decodeCallback,
            encodingStatusCallback: encodingStatusCallback,
            encodeMessageCallback: encodeMessageCallback,
            signersViewControllerCallback: signersViewControllerCallback,
            messageContextViewControllerCallback: messageContextViewControllerCallback,
            primaryActionCallback: primaryActionCallback,
            releaseCallback: releaseCallback,
            userData: userData
        )
    )
    return MAIL_STATUS_OK
}

@_cdecl("mail_message_security_handler_test_decode_message")
public func mail_message_security_handler_test_decode_message(
    _ handler: UnsafeMutableRawPointer?,
    _ bytes: UnsafePointer<UInt8>?,
    _ count: Int,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self), count >= 0 else {
        mailSetError(errorOut, "message-security decode inputs were invalid")
        return nil
    }
    let data: Data
    if count == 0 {
        data = Data()
    } else {
        guard let bytes else {
            mailSetError(errorOut, "message-security decode bytes were null")
            return nil
        }
        data = Data(bytes: bytes, count: count)
    }
    return handler.decodedMessageBox(for: data).map(mailRetainedPointer)
}

func mailSecuritySnapshots(
    _ messageBox: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?
) throws -> (MailMessageSnapshot, MailComposeContextSnapshot) {
    defer {
        mail_object_release(messageBox)
        mail_object_release(contextBox)
    }
    return (
        try mailDecodeSnapshotBox(messageBox, as: MailMessageSnapshot.self),
        try mailDecodeSnapshotBox(contextBox, as: MailComposeContextSnapshot.self)
    )
}

@_cdecl("mail_message_security_handler_test_encoding_status")
public func mail_message_security_handler_test_encoding_status(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let (message, context) = try mailSecuritySnapshots(messageBox, contextBox)
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self) else {
            throw MailBridgeError.invalidArgument("message-security handler was null")
        }
        return mailRetainedPointer(handler.encodingStatusBox(message: message, composeContext: context))
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_message_security_handler_test_encode_message")
public func mail_message_security_handler_test_encode_message(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let (message, context) = try mailSecuritySnapshots(messageBox, contextBox)
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self) else {
            throw MailBridgeError.invalidArgument("message-security handler was null")
        }
        return mailRetainedPointer(handler.encodingResultBox(message: message, composeContext: context))
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_message_security_handler_test_view_controller_for_signers")
public func mail_message_security_handler_test_view_controller_for_signers(
    _ handler: UnsafeMutableRawPointer?,
    _ signersBox: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self),
              let signersBox
        else {
            throw MailBridgeError.invalidArgument("message-security signers inputs were null")
        }
        defer { mail_object_release(signersBox) }
        let signers = try mailDecodeJSONBox(signersBox, as: [MailMessageSignerSnapshot].self)
        if let controller = try handler.controller(callback: handler.signersViewControllerCallback, jsonValue: signers) {
            return mailRetainedPointer(controller)
        }
        return nil
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_message_security_handler_test_view_controller_for_message_context")
public func mail_message_security_handler_test_view_controller_for_message_context(
    _ handler: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self),
              let contextBox
        else {
            throw MailBridgeError.invalidArgument("message-security context inputs were null")
        }
        defer { mail_object_release(contextBox) }
        let context = try mailDecodeJSONBox(contextBox, as: [UInt8].self)
        if let controller = try handler.controller(callback: handler.messageContextViewControllerCallback, jsonValue: context) {
            return mailRetainedPointer(controller)
        }
        return nil
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_message_security_handler_test_primary_action_clicked")
public func mail_message_security_handler_test_primary_action_clicked(
    _ handler: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self),
              let contextBox
        else {
            throw MailBridgeError.invalidArgument("message-security primary-action inputs were null")
        }
        defer { mail_object_release(contextBox) }
        let context = try mailDecodeJSONBox(contextBox, as: [UInt8].self)
        if let controller = try handler.controller(callback: handler.primaryActionCallback, jsonValue: context) {
            return mailRetainedPointer(controller)
        }
        return nil
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_message_security_handler_encoding_status_async")
public func mail_message_security_handler_encoding_status_async(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ cb: @escaping @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    do {
        let (message, context) = try mailSecuritySnapshots(messageBox, contextBox)
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self) else {
            throw MailBridgeError.invalidArgument("message-security async status handler was null")
        }
        cb(mailRetainedPointer(handler.encodingStatusBox(message: message, composeContext: context)), nil, ctx)
    } catch {
        error.localizedDescription.withCString { errorCString in
            cb(nil, errorCString, ctx)
        }
    }
}

@_cdecl("mail_message_security_handler_encode_message_async")
public func mail_message_security_handler_encode_message_async(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ cb: @escaping @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    do {
        let (message, context) = try mailSecuritySnapshots(messageBox, contextBox)
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self) else {
            throw MailBridgeError.invalidArgument("message-security async encode handler was null")
        }
        cb(mailRetainedPointer(handler.encodingResultBox(message: message, composeContext: context)), nil, ctx)
    } catch {
        error.localizedDescription.withCString { errorCString in
            cb(nil, errorCString, ctx)
        }
    }
}

@_cdecl("mail_message_security_handler_primary_action_clicked_async")
public func mail_message_security_handler_primary_action_clicked_async(
    _ handler: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ cb: @escaping @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self),
              let contextBox
        else {
            throw MailBridgeError.invalidArgument("message-security async primary-action inputs were null")
        }
        let context = try mailDecodeConsumedJSONBox(contextBox, as: [UInt8].self)
        do {
            if let controller = try handler.controller(
                callback: handler.primaryActionCallback,
                jsonValue: context
            ) {
                cb(mailRetainedPointer(controller), nil, ctx)
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
