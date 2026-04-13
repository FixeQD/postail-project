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
	isFocused: boolean
	zenMode: boolean
	accentColor: string
	animationsEnabled: boolean
	previewLines: number
	formatDate: (date: string) => string
	onMessageClick: (uid: number, mailbox: string) => void
	onDelete: (uid: number, mailbox: string) => void
	onToggleRead: (uid: number, isUnread: boolean, mailbox: string) => void
	onToggleStar: (uid: number, mailbox: string) => void
}

export interface DragMessagePayload {
	accountId: string
	mailbox: string
	uid: number
	message: import('../../types/mail').MailHeader
}
