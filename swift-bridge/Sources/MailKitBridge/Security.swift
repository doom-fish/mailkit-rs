import Foundation
import MailKit

@_cdecl("mail_message_signer_new")
public func mail_message_signer_new(
    _ addresses: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ addressCount: Int,
    _ label: UnsafePointer<CChar>?,
    _ contextBytes: UnsafePointer<UInt8>?,
    _ contextLen: Int,
    _ hasContext: Bool,
    _ outSigner: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard addressCount >= 0 else {
            throw MailBridgeError.invalidArgument("MEMessageSigner address count was negative")
        }
        guard let label else {
            throw MailBridgeError.invalidArgument("MEMessageSigner label was null")
        }
        let resolvedAddresses: [MEEmailAddress]
        if addressCount == 0 {
            resolvedAddresses = []
        } else {
            guard let addresses else {
                throw MailBridgeError.invalidArgument("MEMessageSigner addresses pointer was null")
            }
            let buffer = UnsafeBufferPointer(start: addresses, count: addressCount)
            resolvedAddresses = buffer.compactMap { mailTakeObject($0, as: MEEmailAddress.self) }
            guard resolvedAddresses.count == addressCount else {
                throw MailBridgeError.invalidArgument("MEMessageSigner addresses contained null entries")
            }
        }
        let context = try mailData(
            bytes: contextBytes,
            count: contextLen,
            hasValue: hasContext,
            label: "MEMessageSigner context"
        )
        outSigner?.pointee = mailRetainedPointer(
            MEMessageSigner(
                emailAddresses: resolvedAddresses,
                signatureLabel: String(cString: label),
                context: context
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

@_cdecl("mail_message_security_information_new")
public func mail_message_security_information_new(
    _ signers: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ signerCount: Int,
    _ isEncrypted: Bool,
    _ signingErrorJSON: UnsafePointer<CChar>?,
    _ encryptionErrorJSON: UnsafePointer<CChar>?,
    _ shouldBlockRemoteContent: Bool,
    _ localizedReason: UnsafePointer<CChar>?,
    _ outInfo: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard signerCount >= 0 else {
            throw MailBridgeError.invalidArgument("MEMessageSecurityInformation signer count was negative")
        }
        let resolvedSigners: [MEMessageSigner]
        if signerCount == 0 {
            resolvedSigners = []
        } else {
            guard let signers else {
                throw MailBridgeError.invalidArgument("MEMessageSecurityInformation signers pointer was null")
            }
            let buffer = UnsafeBufferPointer(start: signers, count: signerCount)
            resolvedSigners = buffer.compactMap { mailTakeObject($0, as: MEMessageSigner.self) }
            guard resolvedSigners.count == signerCount else {
                throw MailBridgeError.invalidArgument("MEMessageSecurityInformation signers contained null entries")
            }
        }
        let signingError = try mailDecodeNSError(signingErrorJSON)
        let encryptionError = try mailDecodeNSError(encryptionErrorJSON)
        let reason = localizedReason.map { String(cString: $0) }
        outInfo?.pointee = mailRetainedPointer(
            MEMessageSecurityInformation(
                signers: resolvedSigners,
                isEncrypted: isEncrypted,
                signingError: signingError,
                encryptionError: encryptionError,
                shouldBlockRemoteContent: shouldBlockRemoteContent,
                localizedRemoteContentBlockingReason: reason
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
