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

typealias MailMessageSecurityDecodeCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

typealias MailMessageSecurityEncodeCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

typealias MailMessageSecurityControllerCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer?

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

    fileprivate func decodeBox(from bytes: [UInt8]) throws -> MailDecodedMessageBox? {
        let dataBox = try mailJSONBoxPointer(for: bytes)
        defer { mail_object_release(dataBox) }
        var errorPtr: UnsafeMutablePointer<CChar>?
        let result = decodeCallback(dataBox, userData, &errorPtr)
        if let message = mailTakeCString(errorPtr) {
            throw MailBridgeError.frameworkError(message)
        }
        guard let result else {
            return nil
        }
        guard let decoded = mailConsumeObject(result, as: MailDecodedMessageBox.self) else {
            throw MailBridgeError.frameworkError("message-security decode callback returned an invalid box")
        }
        return decoded
    }

    fileprivate func encodingStatusBox(
        message: MailMessageSnapshot,
        composeContext: MailComposeContextSnapshot
    ) throws -> MailOutgoingMessageEncodingStatusBox {
        let messageBox = try mailJSONBoxPointer(for: message)
        let contextBox = try mailJSONBoxPointer(for: composeContext)
        defer {
            mail_object_release(messageBox)
            mail_object_release(contextBox)
        }
        var errorPtr: UnsafeMutablePointer<CChar>?
        let result = encodingStatusCallback(messageBox, contextBox, userData, &errorPtr)
        if let message = mailTakeCString(errorPtr) {
            throw MailBridgeError.frameworkError(message)
        }
        guard let result,
              let status = mailConsumeObject(result, as: MailOutgoingMessageEncodingStatusBox.self)
        else {
            throw MailBridgeError.frameworkError("message-security status callback returned null")
        }
        return status
    }

    fileprivate func encodingResultBox(
        message: MailMessageSnapshot,
        composeContext: MailComposeContextSnapshot
    ) throws -> MailMessageEncodingResultBox {
        let messageBox = try mailJSONBoxPointer(for: message)
        let contextBox = try mailJSONBoxPointer(for: composeContext)
        defer {
            mail_object_release(messageBox)
            mail_object_release(contextBox)
        }
        var errorPtr: UnsafeMutablePointer<CChar>?
        let result = encodeMessageCallback(messageBox, contextBox, userData, &errorPtr)
        if let message = mailTakeCString(errorPtr) {
            throw MailBridgeError.frameworkError(message)
        }
        guard let result,
              let encodingResult = mailConsumeObject(result, as: MailMessageEncodingResultBox.self)
        else {
            throw MailBridgeError.frameworkError("message-security encode callback returned null")
        }
        return encodingResult
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
        try? decodeBox(from: Array(data))?.message
    }

    func getEncodingStatus(
        for message: MEMessage,
        composeContext: MEComposeContext,
        completionHandler: @escaping (MEOutgoingMessageEncodingStatus) -> Void
    ) {
        let status = (try? encodingStatusBox(
            message: mailSnapshot(message),
            composeContext: mailSnapshot(composeContext)
        ).status)
        completionHandler(status ?? MEOutgoingMessageEncodingStatus(
            canSign: false,
            canEncrypt: false,
            securityError: nil,
            addressesFailingEncryption: []
        ))
    }

    func encode(
        _ message: MEMessage,
        composeContext: MEComposeContext,
        completionHandler: @escaping (MEMessageEncodingResult) -> Void
    ) {
        let result = (try? encodingResultBox(
            message: mailSnapshot(message),
            composeContext: mailSnapshot(composeContext)
        ).result)
        completionHandler(result ?? MEMessageEncodingResult(encodedMessage: nil, signingError: nil, encryptionError: nil))
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
func mail_message_security_handler_new(
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
func mail_message_security_handler_test_decode_message(
    _ handler: UnsafeMutableRawPointer?,
    _ dataBox: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self),
              let dataBox
        else {
            throw MailBridgeError.invalidArgument("message-security decode inputs were null")
        }
        defer { mail_object_release(dataBox) }
        let bytes = try mailDecodeJSONBox(dataBox, as: [UInt8].self)
        if let box = try handler.decodeBox(from: bytes) {
            return mailRetainedPointer(box)
        }
        return nil
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_message_security_handler_test_encoding_status")
func mail_message_security_handler_test_encoding_status(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self),
              let messageBox,
              let contextBox
        else {
            throw MailBridgeError.invalidArgument("message-security status inputs were null")
        }
        defer {
            mail_object_release(messageBox)
            mail_object_release(contextBox)
        }
        let message = try mailDecodeJSONBox(messageBox, as: MailMessageSnapshot.self)
        let context = try mailDecodeJSONBox(contextBox, as: MailComposeContextSnapshot.self)
        return mailRetainedPointer(try handler.encodingStatusBox(message: message, composeContext: context))
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_message_security_handler_test_encode_message")
func mail_message_security_handler_test_encode_message(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self),
              let messageBox,
              let contextBox
        else {
            throw MailBridgeError.invalidArgument("message-security encode inputs were null")
        }
        defer {
            mail_object_release(messageBox)
            mail_object_release(contextBox)
        }
        let message = try mailDecodeJSONBox(messageBox, as: MailMessageSnapshot.self)
        let context = try mailDecodeJSONBox(contextBox, as: MailComposeContextSnapshot.self)
        return mailRetainedPointer(try handler.encodingResultBox(message: message, composeContext: context))
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return nil
    }
}

@_cdecl("mail_message_security_handler_test_view_controller_for_signers")
func mail_message_security_handler_test_view_controller_for_signers(
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
func mail_message_security_handler_test_view_controller_for_message_context(
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
func mail_message_security_handler_test_primary_action_clicked(
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
func mail_message_security_handler_encoding_status_async(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ cb: @escaping @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self),
              let messageBox,
              let contextBox
        else {
            throw MailBridgeError.invalidArgument("message-security async status inputs were null")
        }
        let message = try mailDecodeConsumedJSONBox(messageBox, as: MailMessageSnapshot.self)
        let context = try mailDecodeConsumedJSONBox(contextBox, as: MailComposeContextSnapshot.self)
        do {
            cb(mailRetainedPointer(try handler.encodingStatusBox(message: message, composeContext: context)), nil, ctx)
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

@_cdecl("mail_message_security_handler_encode_message_async")
func mail_message_security_handler_encode_message_async(
    _ handler: UnsafeMutableRawPointer?,
    _ messageBox: UnsafeMutableRawPointer?,
    _ contextBox: UnsafeMutableRawPointer?,
    _ cb: @escaping @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    do {
        guard let handler = mailTakeObject(handler, as: RustMessageSecurityHandler.self),
              let messageBox,
              let contextBox
        else {
            throw MailBridgeError.invalidArgument("message-security async encode inputs were null")
        }
        let message = try mailDecodeConsumedJSONBox(messageBox, as: MailMessageSnapshot.self)
        let context = try mailDecodeConsumedJSONBox(contextBox, as: MailComposeContextSnapshot.self)
        do {
            cb(mailRetainedPointer(try handler.encodingResultBox(message: message, composeContext: context)), nil, ctx)
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

@_cdecl("mail_message_security_handler_primary_action_clicked_async")
func mail_message_security_handler_primary_action_clicked_async(
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
