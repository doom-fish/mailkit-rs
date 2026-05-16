import Foundation
import MailKit

@_cdecl("mail_email_address_new")
func mail_email_address_new(
    _ rawString: UnsafePointer<CChar>?,
    _ outAddress: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let rawString else {
        mailSetError(errorOut, "MEEmailAddress raw string was null")
        return MAIL_STATUS_INVALID_ARGUMENT
    }
    outAddress?.pointee = mailRetainedPointer(MEEmailAddress(rawString: String(cString: rawString)))
    return MAIL_STATUS_OK
}

@_cdecl("mail_email_address_raw_string")
func mail_email_address_raw_string(_ address: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    mailCopyString(mailTakeObject(address, as: MEEmailAddress.self)?.rawString)
}

@_cdecl("mail_email_address_address_string")
func mail_email_address_address_string(_ address: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    mailCopyString(mailTakeObject(address, as: MEEmailAddress.self)?.addressString)
}

@_cdecl("mail_address_annotation_error")
func mail_address_annotation_error(_ localizedDescription: UnsafePointer<CChar>?) -> UnsafeMutableRawPointer? {
    guard let localizedDescription else {
        return nil
    }
    return mailRetainedPointer(
        MEAddressAnnotation.error(withLocalizedDescription: String(cString: localizedDescription))
    )
}

@_cdecl("mail_address_annotation_warning")
func mail_address_annotation_warning(_ localizedDescription: UnsafePointer<CChar>?) -> UnsafeMutableRawPointer? {
    guard let localizedDescription else {
        return nil
    }
    return mailRetainedPointer(
        MEAddressAnnotation.warning(withLocalizedDescription: String(cString: localizedDescription))
    )
}

@_cdecl("mail_address_annotation_success")
func mail_address_annotation_success(_ localizedDescription: UnsafePointer<CChar>?) -> UnsafeMutableRawPointer? {
    guard let localizedDescription else {
        return nil
    }
    return mailRetainedPointer(
        MEAddressAnnotation.success(withLocalizedDescription: String(cString: localizedDescription))
    )
}
