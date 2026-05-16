import Foundation
import MailKit

struct MailMessageSnapshot: Codable {
    let state: String
    let encryptionState: String
    let subject: String
    let fromAddress: MailEmailAddressSnapshot
    let toAddresses: [MailEmailAddressSnapshot]
    let ccAddresses: [MailEmailAddressSnapshot]
    let bccAddresses: [MailEmailAddressSnapshot]
    let replyToAddresses: [MailEmailAddressSnapshot]
    let allRecipientAddresses: [MailEmailAddressSnapshot]
    let dateSent: String?
    let dateReceived: String?
    let headers: [String: [String]]?
    let rawData: [UInt8]?
}

private let mailISO8601Formatter: ISO8601DateFormatter = {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return formatter
}()

private func mailDateString(_ date: Date?) -> String? {
    guard let date else {
        return nil
    }
    return mailISO8601Formatter.string(from: date)
}

private func mailRefinedDate(_ message: MEMessage, key: String) -> Date? {
    message.value(forKey: key) as? Date
}

func mailMessageStateName(_ state: MEMessageState) -> String {
    switch state {
    case .received:
        return "received"
    case .draft:
        return "draft"
    case .sending:
        return "sending"
    @unknown default:
        return "received"
    }
}

func mailMessageEncryptionStateName(_ state: MEMessageEncryptionState) -> String {
    switch state {
    case .unknown:
        return "unknown"
    case .notEncrypted:
        return "notEncrypted"
    case .encrypted:
        return "encrypted"
    @unknown default:
        return "unknown"
    }
}

func mailSnapshot(_ message: MEMessage) -> MailMessageSnapshot {
    MailMessageSnapshot(
        state: mailMessageStateName(message.state),
        encryptionState: mailMessageEncryptionStateName(message.encryptionState),
        subject: message.subject,
        fromAddress: mailSnapshot(message.fromAddress),
        toAddresses: message.toAddresses.map(mailSnapshot),
        ccAddresses: message.ccAddresses.map(mailSnapshot),
        bccAddresses: message.bccAddresses.map(mailSnapshot),
        replyToAddresses: message.replyToAddresses.map(mailSnapshot),
        allRecipientAddresses: message.allRecipientAddresses.map(mailSnapshot),
        dateSent: mailDateString(mailRefinedDate(message, key: "dateSent")),
        dateReceived: mailDateString(mailRefinedDate(message, key: "dateReceived")),
        headers: message.headers,
        rawData: message.rawData.map(Array.init)
    )
}
