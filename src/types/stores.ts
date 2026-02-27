import type { AccountMeta } from './accounts'

export interface AccountState {
	accounts: AccountMeta[]
	activeAccount: AccountMeta | null
	activeMailbox: string
	lastMailboxes: Record<string, string>
	isLoading: boolean
	setAccounts: (accounts: AccountMeta[]) => void
	setActiveAccount: (account: AccountMeta | null) => void
	setActiveMailbox: (mailbox: string) => void
	fetchAccounts: () => Promise<AccountMeta[]>
	removeAccount: (id: string) => Promise<void>
	updateAccount: (account: AccountMeta) => void
	pendingReauthAccountId: string | null
	setPendingReauthAccountId: (id: string | null) => void
}

export interface DraftFromRust {
	id: string
	accountId: string
	to: string[]
	cc?: string[]
	bcc?: string[]
	subject?: string
	body?: string
	attachments?: import('./compose').EmailAttachment[]
	createdAt: number
	updatedAt: number
}

export interface MessageViewState {
	selectedMessage: {
		accountId: string
		mailbox: string
		uid: number
	} | null
	viewMode: 'html' | 'plain'
	isLoading: boolean
	titleMeta: {
		subject?: string
		from?: string
		date?: string
	}
	openMessage: (accountId: string, mailbox: string, uid: number) => void
	closeMessage: () => void
	setViewMode: (mode: 'html' | 'plain') => void
	setTitleMeta: (meta: MessageViewState['titleMeta']) => void
}

export type ToastType = 'success' | 'error' | 'info' | 'warning' | 'loading'

export interface Toast {
	id: string
	message: string
	description?: string
	type: ToastType
	duration?: number
}

export interface ToastOptions {
	id?: string
	description?: string
	duration?: number
}

export interface ToastStore {
	toasts: Toast[]
	addToast: (message: string, type: ToastType, options?: ToastOptions) => void
	removeToast: (id: string) => void
}

export interface AppSettings {
	'zen-mode': boolean
	'undo-send-delay': number
	'data-path': string
	'auto-lock-timeout': number
	'block-external-images': boolean
}

export interface SettingsState {
	settings: AppSettings
	isLoading: boolean
	setSetting: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => Promise<void>
	loadSettings: () => Promise<void>
}

export interface AccountSyncStatus {
	accountId: string
	accountEmail: string
	status: 'idle' | 'syncing' | 'error'
	mailbox?: string
	progress?: { current: number; total: number }
	mailboxProgress?: { currentMailbox: number; totalMailboxes: number }
	lastSync?: number
	error?: string
}

export interface SyncState {
	statuses: Map<string, AccountSyncStatus>
	isLoading: boolean
	setStatus: (accountId: string, status: AccountSyncStatus) => void
	updateStatus: (accountId: string, updates: Partial<AccountSyncStatus>) => void
	getStatus: (accountId: string) => AccountSyncStatus | undefined
	getAllStatuses: () => AccountSyncStatus[]
	getFormattedLastSync: (accountId: string) => string
	cancelSync: (accountId: string) => Promise<void>
	retrySync: (accountId: string) => Promise<void>
	loadInitialStatuses: (accounts: { id: string; email: string }[]) => Promise<void>
}

export interface OutboxItem {
	id: string
	subject?: string
	recipient?: string
	status: 'PENDING' | 'PROCESSING' | 'SENT' | 'RETRY' | 'FAILED'
	attempts: number
	lastError?: string
}

export interface OutboxState {
	items: OutboxItem[]
	isLoading: boolean
	selectedAccountId: string | null
	loadOutbox: (accountId: string) => Promise<void>
	retryMessage: (outboxId: string) => Promise<void>
	cancelMessage: (outboxId: string) => Promise<void>
	updateItemStatus: (outboxId: string, status: OutboxItem['status'], error?: string) => void
	setSelectedAccount: (accountId: string | null) => void
}

export interface ThemeState {
	accentColor: string
	backgroundId: string
	animationsEnabled: boolean
	isLoaded: boolean
	setAccentColor: (hex: string) => void
	setBackgroundId: (id: string) => void
	setAnimationsEnabled: (enabled: boolean) => void
	persistTheme: () => Promise<void>
	loadTheme: () => Promise<void>
	applyTheme: () => void
}
