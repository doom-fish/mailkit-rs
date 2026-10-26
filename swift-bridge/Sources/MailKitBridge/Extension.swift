import Darwin
import Foundation
import MailKit

private let mailPrincipalLock = NSLock()
private var mailPrincipal: RustMailExtension?

private func mailRegisteredPrincipal() -> RustMailExtension? {
    mailPrincipalLock.lock()
    defer { mailPrincipalLock.unlock() }
    return mailPrincipal
}

private func mailInstallPrincipal() -> RustMailExtension? {
    if let principal = mailRegisteredPrincipal() {
        return principal
    }
    guard let symbol = dlsym(UnsafeMutableRawPointer(bitPattern: -2), "mailkit_install_extension") else {
        return nil
    }
    unsafeBitCast(symbol, to: (@convention(c) () -> Void).self)()
    return mailRegisteredPrincipal()
}

@objc(RustMailExtension)
final class RustMailExtension: NSObject, MEExtension {
    private let composeSessionHandler: RustComposeSessionHandler?
    private let messageActionHandler: RustMessageActionHandler?
    private let contentBlocker: RustContentBlocker?
    private let messageSecurityHandler: RustMessageSecurityHandler?

    init(
        composeSessionHandler: RustComposeSessionHandler?,
        messageActionHandler: RustMessageActionHandler?,
        contentBlocker: RustContentBlocker?,
        messageSecurityHandler: RustMessageSecurityHandler?
    ) {
        self.composeSessionHandler = composeSessionHandler
        self.messageActionHandler = messageActionHandler
        self.contentBlocker = contentBlocker
        self.messageSecurityHandler = messageSecurityHandler
    }

    override convenience init() {
        let principal = mailInstallPrincipal()
        self.init(
            composeSessionHandler: principal?.composeSessionHandler,
            messageActionHandler: principal?.messageActionHandler,
            contentBlocker: principal?.contentBlocker,
            messageSecurityHandler: principal?.messageSecurityHandler
        )
    }

    override func responds(to selector: Selector!) -> Bool {
        switch selector {
        case #selector(handler(for:)):
            return composeSessionHandler != nil
        case #selector(handlerForMessageActions):
            return messageActionHandler != nil
        case #selector(handlerForContentBlocker):
            return contentBlocker != nil
        case #selector(handlerForMessageSecurity):
            return messageSecurityHandler != nil
        default:
            return super.responds(to: selector)
        }
    }

    func handler(for _: MEComposeSession) -> any MEComposeSessionHandler {
        composeSessionHandler!
    }

    func handlerForMessageActions() -> any MEMessageActionHandler {
        messageActionHandler!
    }

    func handlerForContentBlocker() -> any MEContentBlocker {
        contentBlocker!
    }

    func handlerForMessageSecurity() -> any MEMessageSecurityHandler {
        messageSecurityHandler!
    }
}

@_cdecl("mail_extension_new")
public func mail_extension_new(
    _ composeSessionHandler: UnsafeMutableRawPointer?,
    _ messageActionHandler: UnsafeMutableRawPointer?,
    _ contentBlocker: UnsafeMutableRawPointer?,
    _ messageSecurityHandler: UnsafeMutableRawPointer?,
    _ outExtension: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outExtension?.pointee = mailRetainedPointer(
        RustMailExtension(
            composeSessionHandler: mailTakeObject(composeSessionHandler, as: RustComposeSessionHandler.self),
            messageActionHandler: mailTakeObject(messageActionHandler, as: RustMessageActionHandler.self),
            contentBlocker: mailTakeObject(contentBlocker, as: RustContentBlocker.self),
            messageSecurityHandler: mailTakeObject(messageSecurityHandler, as: RustMessageSecurityHandler.self)
        )
    )
    return MAIL_STATUS_OK
}

@_cdecl("mail_extension_register_principal")
public func mail_extension_register_principal(_ extensionPtr: UnsafeMutableRawPointer?) -> Bool {
    let mailExtension = mailTakeObject(extensionPtr, as: RustMailExtension.self)
    mailPrincipalLock.lock()
    mailPrincipal = mailExtension
    mailPrincipalLock.unlock()
    return mailExtension != nil
}

@_cdecl("mail_extension_test_new_principal")
public func mail_extension_test_new_principal() -> UnsafeMutableRawPointer? {
    guard let principalClass = NSClassFromString("RustMailExtension") as? NSObject.Type else {
        return nil
    }
    return mailRetainedPointer(principalClass.init())
}

@_cdecl("mail_extension_test_has_compose_session_handler")
public func mail_extension_test_has_compose_session_handler(_ extensionPtr: UnsafeMutableRawPointer?) -> Bool {
    guard let mailExtension = mailTakeObject(extensionPtr, as: RustMailExtension.self) else {
        return false
    }
    return mailExtension.responds(to: #selector(RustMailExtension.handler(for:)))
}

@_cdecl("mail_extension_test_has_message_action_handler")
public func mail_extension_test_has_message_action_handler(_ extensionPtr: UnsafeMutableRawPointer?) -> Bool {
    guard let mailExtension = mailTakeObject(extensionPtr, as: RustMailExtension.self) else {
        return false
    }
    return mailExtension.responds(to: #selector(RustMailExtension.handlerForMessageActions))
}

@_cdecl("mail_extension_test_has_content_blocker")
public func mail_extension_test_has_content_blocker(_ extensionPtr: UnsafeMutableRawPointer?) -> Bool {
    guard let mailExtension = mailTakeObject(extensionPtr, as: RustMailExtension.self) else {
        return false
    }
    return mailExtension.responds(to: #selector(RustMailExtension.handlerForContentBlocker))
}

@_cdecl("mail_extension_test_has_message_security_handler")
public func mail_extension_test_has_message_security_handler(_ extensionPtr: UnsafeMutableRawPointer?) -> Bool {
    guard let mailExtension = mailTakeObject(extensionPtr, as: RustMailExtension.self) else {
        return false
    }
    return mailExtension.responds(to: #selector(RustMailExtension.handlerForMessageSecurity))
}
