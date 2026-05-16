// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "MailKitBridge",
    platforms: [
        .macOS(.v13),
    ],
    products: [
        .library(
            name: "MailKitBridge",
            type: .static,
            targets: ["MailKitBridge"]
        )
    ],
    targets: [
        .target(
            name: "MailKitBridge",
            path: "Sources/MailKitBridge",
            publicHeadersPath: "include"
        )
    ]
)
