export interface UseInboxShortcutsProps {
	onNextMessage: () => void
	onPrevMessage: () => void
	onOpenMessage: () => void
	onDeleteMessage: () => void
	onReply: () => void
	onReplyAll: () => void
	onForward: () => void
	onNewMessage: () => void
	onToggleRead: () => void
	onMarkUnread: () => void
	onToggleStar: () => void
	onFocusSearch: () => void
}

export interface UseComposeShortcutsProps {
	onSend: () => void
	onSaveDraft: () => void
	onClose: () => void
	onAttachFile: () => void
	onInsertLink: () => void
	onToggleCc: () => void
	onToggleBcc: () => void
	enabled?: boolean
}

export interface UseGlobalShortcutsProps {
	onNewMessage: () => void
	onFocusSearch: () => void
	onRefresh: () => void
	onGoToInbox: () => void
	onGoToOutbox: () => void
	onGoToDrafts: () => void
	onGoToAccounts: () => void
	onOpenSettings: () => void
	enabled?: boolean
}
