import MailKit

@_cdecl("mail_message_action_move_to_trash")
func mail_message_action_move_to_trash() -> UnsafeMutableRawPointer? {
    mailRetainedPointer(MEMessageAction.moveToTrash)
}

@_cdecl("mail_message_action_move_to_archive")
func mail_message_action_move_to_archive() -> UnsafeMutableRawPointer? {
    mailRetainedPointer(MEMessageAction.moveToArchive)
}

@_cdecl("mail_message_action_move_to_junk")
func mail_message_action_move_to_junk() -> UnsafeMutableRawPointer? {
    mailRetainedPointer(MEMessageAction.moveToJunk)
}

@_cdecl("mail_message_action_mark_as_read")
func mail_message_action_mark_as_read() -> UnsafeMutableRawPointer? {
    mailRetainedPointer(MEMessageAction.markAsRead)
}

@_cdecl("mail_message_action_mark_as_unread")
func mail_message_action_mark_as_unread() -> UnsafeMutableRawPointer? {
    mailRetainedPointer(MEMessageAction.markAsUnread)
}

@_cdecl("mail_message_action_flag")
func mail_message_action_flag(_ flag: Int32) -> UnsafeMutableRawPointer? {
    guard let flag = MEMessageAction.Flag(rawValue: Int(flag)) else {
        return nil
    }
    return mailRetainedPointer(MEMessageAction.flag(flag))
}

@_cdecl("mail_message_action_set_background_color")
func mail_message_action_set_background_color(_ color: Int32) -> UnsafeMutableRawPointer? {
    guard let color = MEMessageAction.MessageColor(rawValue: Int(color)) else {
        return nil
    }
    return mailRetainedPointer(MEMessageAction.setBackgroundColor(color))
}

@_cdecl("mail_message_action_decision_invoke_again_with_body")
func mail_message_action_decision_invoke_again_with_body() -> UnsafeMutableRawPointer? {
    mailRetainedPointer(MEMessageActionDecision.invokeAgainWithBody)
}

@_cdecl("mail_message_action_decision_action")
func mail_message_action_decision_action(
    _ action: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer? {
    guard let action = mailTakeObject(action, as: MEMessageAction.self) else {
        return nil
    }
    return mailRetainedPointer(MEMessageActionDecision.action(action))
}

@_cdecl("mail_message_action_decision_actions")
func mail_message_action_decision_actions(
    _ actions: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ actionCount: Int
) -> UnsafeMutableRawPointer? {
    guard actionCount >= 0 else {
        return nil
    }
    let resolved: [MEMessageAction]
    if actionCount == 0 {
        resolved = []
    } else {
        guard let actions else {
            return nil
        }
        let buffer = UnsafeBufferPointer(start: actions, count: actionCount)
        resolved = buffer.compactMap { mailTakeObject($0, as: MEMessageAction.self) }
        guard resolved.count == actionCount else {
            return nil
        }
    }
    return mailRetainedPointer(MEMessageActionDecision.actions(resolved))
}
