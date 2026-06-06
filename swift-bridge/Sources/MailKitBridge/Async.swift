import Foundation
import MailKit

// MARK: - Message Action Decision Async

@_cdecl("mail_message_action_decision_async")
func mail_message_action_decision_async(
    messageBox: UnsafeMutableRawPointer,
    cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    ctx: UnsafeMutableRawPointer
) {
    Task {
        // Note: This is a stub implementation. In a real extension, you would:
        // decode the message from messageBox
        // let decision = try await MEMessageActionHandler.decide(...)
        // Then return the decision via callback
        //
        // The error C-string must only be produced and consumed *inside* the
        // `withCString` closure: the pointer it yields is deallocated as soon
        // as the closure returns, so escaping it (e.g. `let e = ....withCString { $0 }`)
        // and passing it to `cb` afterwards is a use-after-free across FFI.
        "Not implemented in non-extension context".withCString { errorCStr in
            cb(nil, errorCStr, ctx)
        }
    }
}

// MARK: - Content Blocker Async

@_cdecl("mail_content_blocker_content_rules_async")
func mail_content_blocker_content_rules_async(
    blocker: UnsafeMutableRawPointer,
    cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    ctx: UnsafeMutableRawPointer
) {
    Task {
        do {
            // Note: MailKit async APIs for content blockers are extension-only
            // This is a stub implementation. In a real extension, you would:
            // let rules = try await MEContentBlocker.contentRulesURL()
            
            let emptyRules: [UInt8] = []
            let rulesData = Data(emptyRules)
            let jsonData = try JSONSerialization.data(
                withJSONObject: ["rules": []] as [String: Any],
                options: []
            )
            
            let box = MailJSONBox(json: String(data: jsonData, encoding: .utf8) ?? "[]")
            let result = Unmanaged.passRetained(box).toOpaque()
            cb(result, nil, ctx)
        } catch {
            error.localizedDescription.withCString { errorCStr in
                cb(nil, errorCStr, ctx)
            }
        }
    }
}

