import Foundation
import MailKit

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
func mail_extension_new(
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

@_cdecl("mail_extension_test_has_compose_session_handler")
func mail_extension_test_has_compose_session_handler(_ extensionPtr: UnsafeMutableRawPointer?) -> Bool {
    guard let mailExtension = mailTakeObject(extensionPtr, as: RustMailExtension.self) else {
        return false
    }
    return mailExtension.responds(to: #selector(RustMailExtension.handler(for:)))
}

@_cdecl("mail_extension_test_has_message_action_handler")
func mail_extension_test_has_message_action_handler(_ extensionPtr: UnsafeMutableRawPointer?) -> Bool {
    guard let mailExtension = mailTakeObject(extensionPtr, as: RustMailExtension.self) else {
        return false
    }
    return mailExtension.responds(to: #selector(RustMailExtension.handlerForMessageActions))
}

@_cdecl("mail_extension_test_has_content_blocker")
func mail_extension_test_has_content_blocker(_ extensionPtr: UnsafeMutableRawPointer?) -> Bool {
    guard let mailExtension = mailTakeObject(extensionPtr, as: RustMailExtension.self) else {
        return false
    }
    return mailExtension.responds(to: #selector(RustMailExtension.handlerForContentBlocker))
}

@_cdecl("mail_extension_test_has_message_security_handler")
func mail_extension_test_has_message_security_handler(_ extensionPtr: UnsafeMutableRawPointer?) -> Bool {
    guard let mailExtension = mailTakeObject(extensionPtr, as: RustMailExtension.self) else {
        return false
    }
    return mailExtension.responds(to: #selector(RustMailExtension.handlerForMessageSecurity))
}
