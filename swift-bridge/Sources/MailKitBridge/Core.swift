import Foundation
import MailKit

let MAIL_STATUS_OK: Int32 = 0
let MAIL_STATUS_INVALID_ARGUMENT: Int32 = -1
let MAIL_STATUS_FRAMEWORK_ERROR: Int32 = -2
let MAIL_STATUS_OPERATION_FAILED: Int32 = -3
let MAIL_STATUS_TIMED_OUT: Int32 = -4

struct NSErrorPayload: Codable {
    let domain: String
    let code: Int64
    let localizedDescription: String
}

enum MailBridgeError: LocalizedError {
    case invalidArgument(String)
    case frameworkError(String)

    var errorDescription: String? {
        switch self {
        case let .invalidArgument(message), let .frameworkError(message):
            return message
        }
    }
}

final class MailJSONBox: NSObject {
    let json: String

    init(json: String) {
        self.json = json
    }
}

final class MailMessageActionDecisionBox: NSObject {
    let decision: MEMessageActionDecision
    let json: String

    init(decision: MEMessageActionDecision, json: String) {
        self.decision = decision
        self.json = json
    }
}

final class MailDecodedMessageBox: NSObject {
    let message: MEDecodedMessage
    let json: String

    init(message: MEDecodedMessage, json: String) {
        self.message = message
        self.json = json
    }
}

final class MailOutgoingMessageEncodingStatusBox: NSObject {
    let status: MEOutgoingMessageEncodingStatus
    let json: String

    init(status: MEOutgoingMessageEncodingStatus, json: String) {
        self.status = status
        self.json = json
    }
}

final class MailMessageEncodingResultBox: NSObject {
    let result: MEMessageEncodingResult
    let json: String

    init(result: MEMessageEncodingResult, json: String) {
        self.result = result
        self.json = json
    }
}

func mailRetainedPointer(_ object: AnyObject) -> UnsafeMutableRawPointer {
    UnsafeMutableRawPointer(Unmanaged.passRetained(object).toOpaque())
}

func mailTakeObject<T: AnyObject>(_ ptr: UnsafeMutableRawPointer?, as _: T.Type) -> T? {
    guard let ptr else {
        return nil
    }
    return Unmanaged<T>.fromOpaque(ptr).takeUnretainedValue()
}

func mailConsumeObject<T: AnyObject>(_ ptr: UnsafeMutableRawPointer?, as _: T.Type) -> T? {
    guard let ptr else {
        return nil
    }
    return Unmanaged<T>.fromOpaque(ptr).takeRetainedValue()
}

func mailCopyString(_ value: String?) -> UnsafeMutablePointer<CChar>? {
    guard let value else {
        return nil
    }
    return strdup(value)
}

func mailTakeCString(_ ptr: UnsafeMutablePointer<CChar>?) -> String? {
    guard let ptr else {
        return nil
    }
    defer { free(ptr) }
    return String(cString: ptr)
}

func mailWithRetainedPointer<T: AnyObject, R>(
    _ object: T,
    _ body: (UnsafeMutableRawPointer) throws -> R
) rethrows -> R {
    let ptr = mailRetainedPointer(object)
    defer { mail_object_release(ptr) }
    return try body(ptr)
}

func mailJSONString<T: Encodable>(from value: T) throws -> String {
    let data = try JSONEncoder().encode(value)
    guard let string = String(data: data, encoding: .utf8) else {
        throw MailBridgeError.frameworkError("failed to encode MailKit JSON payload as UTF-8")
    }
    return string
}

func mailJSONBoxPointer<T: Encodable>(for value: T) throws -> UnsafeMutableRawPointer {
    mailRetainedPointer(MailJSONBox(json: try mailJSONString(from: value)))
}

func mailSetError(_ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?, _ value: String) {
    guard let errorOut else {
        return
    }
    errorOut.pointee = strdup(value)
}

func mailSetErrorJSON<T: Encodable>(
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ payload: T
) {
    do {
        mailSetError(errorOut, try mailJSONString(from: payload))
    } catch {
        mailSetError(errorOut, (error as NSError).localizedDescription)
    }
}

func mailDecodeJSON<T: Decodable>(_ json: UnsafePointer<CChar>?, as _: T.Type) throws -> T {
    guard let json else {
        throw MailBridgeError.invalidArgument("MailKit JSON payload was null")
    }
    let data = Data(String(cString: json).utf8)
    return try JSONDecoder().decode(T.self, from: data)
}

func mailDecodeJSONBox<T: Decodable>(_ ptr: UnsafeMutableRawPointer?, as _: T.Type) throws -> T {
    guard let box = mailTakeObject(ptr, as: MailJSONBox.self) else {
        throw MailBridgeError.invalidArgument("MailKit JSON box was null")
    }
    return try JSONDecoder().decode(T.self, from: Data(box.json.utf8))
}

func mailDecodeConsumedJSONBox<T: Decodable>(_ ptr: UnsafeMutableRawPointer?, as _: T.Type) throws -> T {
    guard let box = mailConsumeObject(ptr, as: MailJSONBox.self) else {
        throw MailBridgeError.invalidArgument("MailKit JSON box was null")
    }
    return try JSONDecoder().decode(T.self, from: Data(box.json.utf8))
}

func mailDecodeNSError(_ json: UnsafePointer<CChar>?) throws -> NSError? {
    guard let json else {
        return nil
    }
    let payload = try mailDecodeJSON(json, as: NSErrorPayload.self)
    return NSError(
        domain: payload.domain,
        code: Int(payload.code),
        userInfo: [NSLocalizedDescriptionKey: payload.localizedDescription]
    )
}

func mailNSErrorPayload(_ error: NSError) -> NSErrorPayload {
    NSErrorPayload(
        domain: error.domain,
        code: Int64(error.code),
        localizedDescription: error.localizedDescription
    )
}

func mailData(
    bytes: UnsafePointer<UInt8>?,
    count: Int,
    hasValue: Bool,
    label: String
) throws -> Data? {
    guard hasValue else {
        return nil
    }
    guard count >= 0 else {
        throw MailBridgeError.invalidArgument("\(label) length was negative")
    }
    if count == 0 {
        return Data()
    }
    guard let bytes else {
        throw MailBridgeError.invalidArgument("\(label) bytes were null")
    }
    return Data(bytes: bytes, count: count)
}

@_cdecl("mail_json_box_new")
func mail_json_box_new(_ json: UnsafePointer<CChar>?) -> UnsafeMutableRawPointer? {
    guard let json else {
        return nil
    }
    return mailRetainedPointer(MailJSONBox(json: String(cString: json)))
}

@_cdecl("mail_json_box_copy")
func mail_json_box_copy(_ box: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    mailCopyString(mailTakeObject(box, as: MailJSONBox.self)?.json)
}

@_cdecl("mail_message_action_decision_box_new")
func mail_message_action_decision_box_new(
    _ decision: UnsafeMutableRawPointer?,
    _ json: UnsafePointer<CChar>?
) -> UnsafeMutableRawPointer? {
    guard let decision = mailConsumeObject(decision, as: MEMessageActionDecision.self), let json else {
        return nil
    }
    return mailRetainedPointer(MailMessageActionDecisionBox(decision: decision, json: String(cString: json)))
}

@_cdecl("mail_message_action_decision_box_copy_json")
func mail_message_action_decision_box_copy_json(_ box: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    mailCopyString(mailTakeObject(box, as: MailMessageActionDecisionBox.self)?.json)
}

@_cdecl("mail_message_action_decision_box_copy_value")
func mail_message_action_decision_box_copy_value(_ box: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let box = mailTakeObject(box, as: MailMessageActionDecisionBox.self) else {
        return nil
    }
    return mailRetainedPointer(box.decision)
}

@_cdecl("mail_decoded_message_box_new")
func mail_decoded_message_box_new(
    _ message: UnsafeMutableRawPointer?,
    _ json: UnsafePointer<CChar>?
) -> UnsafeMutableRawPointer? {
    guard let message = mailConsumeObject(message, as: MEDecodedMessage.self), let json else {
        return nil
    }
    return mailRetainedPointer(MailDecodedMessageBox(message: message, json: String(cString: json)))
}

@_cdecl("mail_decoded_message_box_copy_json")
func mail_decoded_message_box_copy_json(_ box: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    mailCopyString(mailTakeObject(box, as: MailDecodedMessageBox.self)?.json)
}

@_cdecl("mail_decoded_message_box_copy_value")
func mail_decoded_message_box_copy_value(_ box: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let box = mailTakeObject(box, as: MailDecodedMessageBox.self) else {
        return nil
    }
    return mailRetainedPointer(box.message)
}

@_cdecl("mail_outgoing_message_encoding_status_box_new")
func mail_outgoing_message_encoding_status_box_new(
    _ status: UnsafeMutableRawPointer?,
    _ json: UnsafePointer<CChar>?
) -> UnsafeMutableRawPointer? {
    guard let status = mailConsumeObject(status, as: MEOutgoingMessageEncodingStatus.self), let json else {
        return nil
    }
    return mailRetainedPointer(MailOutgoingMessageEncodingStatusBox(status: status, json: String(cString: json)))
}

@_cdecl("mail_outgoing_message_encoding_status_box_copy_json")
func mail_outgoing_message_encoding_status_box_copy_json(_ box: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    mailCopyString(mailTakeObject(box, as: MailOutgoingMessageEncodingStatusBox.self)?.json)
}

@_cdecl("mail_outgoing_message_encoding_status_box_copy_value")
func mail_outgoing_message_encoding_status_box_copy_value(
    _ box: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer? {
    guard let box = mailTakeObject(box, as: MailOutgoingMessageEncodingStatusBox.self) else {
        return nil
    }
    return mailRetainedPointer(box.status)
}

@_cdecl("mail_message_encoding_result_box_new")
func mail_message_encoding_result_box_new(
    _ result: UnsafeMutableRawPointer?,
    _ json: UnsafePointer<CChar>?
) -> UnsafeMutableRawPointer? {
    guard let result = mailConsumeObject(result, as: MEMessageEncodingResult.self), let json else {
        return nil
    }
    return mailRetainedPointer(MailMessageEncodingResultBox(result: result, json: String(cString: json)))
}

@_cdecl("mail_message_encoding_result_box_copy_json")
func mail_message_encoding_result_box_copy_json(_ box: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    mailCopyString(mailTakeObject(box, as: MailMessageEncodingResultBox.self)?.json)
}

@_cdecl("mail_message_encoding_result_box_copy_value")
func mail_message_encoding_result_box_copy_value(_ box: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let box = mailTakeObject(box, as: MailMessageEncodingResultBox.self) else {
        return nil
    }
    return mailRetainedPointer(box.result)
}

@_cdecl("mail_compose_session_error_domain_copy")
func mail_compose_session_error_domain_copy() -> UnsafeMutablePointer<CChar>? {
    mailCopyString(MEComposeSessionErrorDomain)
}

@_cdecl("mail_message_security_error_domain_copy")
func mail_message_security_error_domain_copy() -> UnsafeMutablePointer<CChar>? {
    mailCopyString(MEMessageSecurityErrorDomain)
}

@_cdecl("mail_extension_view_controller_new")
func mail_extension_view_controller_new() -> UnsafeMutableRawPointer? {
    mailRetainedPointer(MEExtensionViewController())
}

@_cdecl("mail_object_release")
func mail_object_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else {
        return
    }
    Unmanaged<AnyObject>.fromOpaque(ptr).release()
}
