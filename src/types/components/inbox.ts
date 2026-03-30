export interface InboxScreenProps {
	onOpenSettings: () => void
}

export interface MessageListProps {
	account: import('../../types/accounts').AccountMeta
	mailbox: string
	focusedUid?: number | null
	onMessageClick: (uid: number, mailbox: string) => void
}

export interface MessageRowProps {
	message: import('../../types/mail').MailHeader
	isUnread: boolean
	isHovered: boolean
	isFocused: boolean
	zenMode: boolean
	accentColor: string
	animationsEnabled: boolean
	previewLines: number
	formatDate: (date: string) => string
	onMessageClick: (uid: number, mailbox: string) => void
	onMouseEnter: () => void
	onMouseLeave: () => void
	onDelete: () => void
	onToggleRead: () => void
	onToggleStar: () => void
}
