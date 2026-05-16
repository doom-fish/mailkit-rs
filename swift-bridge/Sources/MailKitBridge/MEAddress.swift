import MailKit

struct MailEmailAddressSnapshot: Codable {
    let rawString: String
    let addressString: String?
}

func mailSnapshot(_ address: MEEmailAddress) -> MailEmailAddressSnapshot {
    MailEmailAddressSnapshot(rawString: address.rawString, addressString: address.addressString)
}
