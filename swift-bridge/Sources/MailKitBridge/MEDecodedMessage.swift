import Foundation
import MailKit

@_cdecl("mail_decoded_message_banner_new")
func mail_decoded_message_banner_new(
    _ title: UnsafePointer<CChar>?,
    _ primaryActionTitle: UnsafePointer<CChar>?,
    _ dismissable: Bool,
    _ outBanner: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let title, let primaryActionTitle else {
        mailSetError(errorOut, "MEDecodedMessageBanner title arguments were null")
        return MAIL_STATUS_INVALID_ARGUMENT
    }
    outBanner?.pointee = mailRetainedPointer(
        MEDecodedMessageBanner(
            title: String(cString: title),
            primaryActionTitle: String(cString: primaryActionTitle),
            dismissable: dismissable
        )
    )
    return MAIL_STATUS_OK
}

@_cdecl("mail_decoded_message_new")
func mail_decoded_message_new(
    _ rawDataBytes: UnsafePointer<UInt8>?,
    _ rawDataLen: Int,
    _ hasRawData: Bool,
    _ securityInformation: UnsafeMutableRawPointer?,
    _ contextBytes: UnsafePointer<UInt8>?,
    _ contextLen: Int,
    _ hasContext: Bool,
    _ banner: UnsafeMutableRawPointer?,
    _ outMessage: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let securityInformation = mailTakeObject(
            securityInformation,
            as: MEMessageSecurityInformation.self
        ) else {
            throw MailBridgeError.invalidArgument("MEDecodedMessage security information was null")
        }
        let rawData = try mailData(
            bytes: rawDataBytes,
            count: rawDataLen,
            hasValue: hasRawData,
            label: "MEDecodedMessage raw data"
        )
        let context = try mailData(
            bytes: contextBytes,
            count: contextLen,
            hasValue: hasContext,
            label: "MEDecodedMessage context"
        )
        let banner = mailTakeObject(banner, as: MEDecodedMessageBanner.self)
        outMessage?.pointee = mailRetainedPointer(
            MEDecodedMessage(
                data: rawData,
                securityInformation: securityInformation,
                context: context,
                banner: banner
            )
        )
        return MAIL_STATUS_OK
    } catch let error as MailBridgeError {
        mailSetError(errorOut, error.localizedDescription)
        return MAIL_STATUS_INVALID_ARGUMENT
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
        return MAIL_STATUS_FRAMEWORK_ERROR
    }
}
