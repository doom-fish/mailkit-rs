import Foundation
import MailKit

@_cdecl("mail_encoded_outgoing_message_new")
func mail_encoded_outgoing_message_new(
    _ rawDataBytes: UnsafePointer<UInt8>?,
    _ rawDataLen: Int,
    _ isSigned: Bool,
    _ isEncrypted: Bool,
    _ outMessage: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let rawData = try mailData(
            bytes: rawDataBytes,
            count: rawDataLen,
            hasValue: true,
            label: "MEEncodedOutgoingMessage raw data"
        ) ?? Data()
        outMessage?.pointee = mailRetainedPointer(
            MEEncodedOutgoingMessage(
                rawData: rawData,
                isSigned: isSigned,
                isEncrypted: isEncrypted
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

@_cdecl("mail_outgoing_message_encoding_status_new")
func mail_outgoing_message_encoding_status_new(
    _ canSign: Bool,
    _ canEncrypt: Bool,
    _ securityErrorJSON: UnsafePointer<CChar>?,
    _ addressesFailingEncryption: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ failingAddressCount: Int,
    _ outStatus: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard failingAddressCount >= 0 else {
            throw MailBridgeError.invalidArgument("MEOutgoingMessageEncodingStatus address count was negative")
        }
        let resolvedAddresses: [MEEmailAddress]
        if failingAddressCount == 0 {
            resolvedAddresses = []
        } else {
            guard let addressesFailingEncryption else {
                throw MailBridgeError.invalidArgument("MEOutgoingMessageEncodingStatus addresses pointer was null")
            }
            let buffer = UnsafeBufferPointer(start: addressesFailingEncryption, count: failingAddressCount)
            resolvedAddresses = buffer.compactMap { mailTakeObject($0, as: MEEmailAddress.self) }
            guard resolvedAddresses.count == failingAddressCount else {
                throw MailBridgeError.invalidArgument("MEOutgoingMessageEncodingStatus addresses contained null entries")
            }
        }
        let securityError = try mailDecodeNSError(securityErrorJSON)
        outStatus?.pointee = mailRetainedPointer(
            MEOutgoingMessageEncodingStatus(
                canSign: canSign,
                canEncrypt: canEncrypt,
                securityError: securityError,
                addressesFailingEncryption: resolvedAddresses
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

@_cdecl("mail_message_encoding_result_new")
func mail_message_encoding_result_new(
    _ encodedMessage: UnsafeMutableRawPointer?,
    _ signingErrorJSON: UnsafePointer<CChar>?,
    _ encryptionErrorJSON: UnsafePointer<CChar>?,
    _ outResult: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let encodedMessage = mailTakeObject(encodedMessage, as: MEEncodedOutgoingMessage.self)
        let signingError = try mailDecodeNSError(signingErrorJSON)
        let encryptionError = try mailDecodeNSError(encryptionErrorJSON)
        outResult?.pointee = mailRetainedPointer(
            MEMessageEncodingResult(
                encodedMessage: encodedMessage,
                signingError: signingError,
                encryptionError: encryptionError
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
