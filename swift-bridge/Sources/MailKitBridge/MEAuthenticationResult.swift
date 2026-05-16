func mailAuthenticationResultsHeaderValues(from snapshot: MailMessageSnapshot) -> [String] {
    snapshot.headers?
        .filter { $0.key.caseInsensitiveCompare("Authentication-Results") == .orderedSame }
        .flatMap(\.value) ?? []
}
