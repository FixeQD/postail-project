import type { AccountMeta } from '../accounts'
import type { Mailbox } from '../mail'

export interface TitleBarProps {
	isDashboard?: boolean
	onSearch?: (query: string) => void
	onOpenSettings?: () => void
	onOpenOutbox?: () => void
}

export interface SidebarProps {
	activeAccount: AccountMeta | null
	activeMailbox: string
	onMailboxSelect: (mailbox: string) => void
	onCompose: () => void
}

export interface MailboxItemProps {
	mailbox: Mailbox
	isActive: boolean
	isCollapsed?: boolean
	unreadCount?: number
	accentColor?: string
	animationsEnabled?: boolean
	onClick?: () => void
	onSelect?: (name: string) => void
}

export interface StatusBarProps {
	onOpenOutbox: () => void
}

export interface LockScreenProps {
	isLocked: boolean
	onUnlock: () => void
	useEncryptionPassword: boolean
}

export interface DecryptedTextProps {
	text: string
	speed?: number
	maxIterations?: number
	sequential?: boolean
	revealDirection?: 'start' | 'end' | 'center'
	useOriginalCharsOnly?: boolean
	characters?: string
	className?: string
	encryptedClassName?: string
	parentClassName?: string
	animateOn?: 'view' | 'hover' | 'both'
}

export interface AccountSwitcherProps {
	onOpenSettings: () => void
}

export interface RecoveryVerifyDialogProps {
	open: boolean
	onClose: () => void
	onVerified: () => void | Promise<void>
}

export interface OutboxPanelProps {
	accountId: string
	isOpen: boolean
	onClose: () => void
}

export interface DraftsListProps {
	accountId: string
	onDraftClick: (draft: import('../compose').ComposeDraft) => void
}

export interface MessageViewProps {
	accountId: string
	mailbox: string
	uid: number
	onBack: () => void
	onNext?: () => void
	onPrev?: () => void
}

export interface MessageViewHeaderProps {
	onBack: () => void
	onReply: () => void
	onReplyAll: () => void
	onForward: () => void
	onDelete: () => void
	onMarkUnread: () => void
	onViewSource?: () => void
	hasHtml?: boolean
	isDeleting?: boolean
}

export interface MessageViewAttachmentsProps {
	attachments: import('../mail').AttachmentMeta[]
	accountId: string
	mailbox: string
	uid: number
}

export interface MessageViewBodyProps {
	htmlContent: string
	plainContent: string
	viewMode: 'html' | 'plain'
	allowExternalResources?: boolean
	inline_images?: import('../mail').AttachmentMeta[]
	onExternalDetected?: () => void
	onLoadingChange?: (loading: boolean) => void
}

export interface MessageViewMetaProps {
	header: import('../mail').MailHeader
}

export interface SettingCardProps {
	label: string
	description: string
	icon: React.ComponentType<{ className?: string }>
	children: React.ReactNode
	disabled?: boolean
}

export interface AddAccountDialogProps {
	onAccountAdded?: (accountId?: string) => void
	children?: React.ReactNode
}

export interface EditAccountDialogProps {
	account: AccountMeta
	open: boolean
	onOpenChange: (open: boolean) => void
}

export interface ManualAccountFormProps {
	onSuccess: (accountId: string) => void
	onCancel: () => void
	editAccount?: AccountMeta
}

export interface FormData {
	accountName: string
	email: string
	useSeparateUsername: boolean
	username?: string
	password?: string
	imapHost: string
	imapPort: string
	smtpHost: string
	smtpPort: string
	useSsl: boolean
}

export interface AccountCardProps {
	account: AccountMeta
	onRemove: (id: string) => void
	onSync: (id: string) => void
}

export interface BuildInfo {
	version: string
	build_timestamp: string
	git_hash: string
	git_branch: string
	profile: string
	rustc: string
}
