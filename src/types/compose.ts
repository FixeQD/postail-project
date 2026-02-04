export interface EmailAddress {
	email: string
	name?: string
}

export interface EmailAttachment {
	id: string
	filename: string
	contentType: string
	size: number
	cid?: string
	inline: boolean
	path?: string
	hash: string
}

export interface ComposeDraft {
	id?: string
	accountId: string
	to: EmailAddress[]
	cc?: EmailAddress[]
	bcc?: EmailAddress[]
	subject: string
	body: string
	bodyType: 'plain' | 'html'
	attachments: EmailAttachment[]
	isReplyTo?: string
	isForwardOf?: string
	createdAt: string
	updatedAt: string
}

export type IssueSeverity = 'Info' | 'Warning' | 'Error'

export interface SanitizeIssue {
	property: string
	reason: string
	severity: IssueSeverity
	count: number
}

export interface DraftState {
	// Current draft being edited
	currentDraft: ComposeDraft | null

	// All drafts for current account
	drafts: ComposeDraft[]

	// UI state
	isComposing: boolean
	isDirty: boolean
	isSaving: boolean
	isSending: boolean
	lastSavedAt?: Date
	editorMode: 'rich-text' | 'source'

	compatibilityPanelOpen: boolean
	compatibilityPanelWidth: number
	compatibilityIssues: SanitizeIssue[]
	isValidating: boolean
	validationDismissed: boolean
	showSendWarning: boolean

	// Actions
	setCurrentDraft: (draft: ComposeDraft | null) => void
	updateCurrentDraft: (updates: Partial<ComposeDraft>) => void
	setEditorMode: (mode: 'rich-text' | 'source') => void
	addRecipient: (type: 'to' | 'cc' | 'bcc', recipient: EmailAddress) => void
	removeRecipient: (type: 'to' | 'cc' | 'bcc', email: string) => void
	setSubject: (subject: string) => void
	setBody: (body: string) => void
	addAttachment: (attachment: EmailAttachment) => void
	removeAttachment: (attachmentId: string) => void
	startComposing: (accountId: string, draft?: Partial<ComposeDraft>) => void
	stopComposing: () => void
	markDirty: () => void
	markClean: () => void
	saveDraft: (html?: string) => Promise<void>
	loadDraft: (draft: ComposeDraft) => void
	loadDrafts: (accountId: string, signal?: AbortSignal) => Promise<string>
	deleteDraft: (draftId: string) => Promise<void>
	sendDraft: (html?: string) => Promise<string>

	toggleCompatibilityPanel: () => void
	setCompatibilityPanelWidth: (width: number) => void
	validateCompatibility: (html: string, immediate?: boolean) => Promise<void>
	applyAutoFix: (html: string) => Promise<string>
	dismissValidationWarning: () => void
	resetValidationDismissed: () => void
	setShowSendWarning: (show: boolean) => void

	// Keyboard shortcut triggers
	triggerAttachFile: () => void
	triggerInsertLink: () => void
}
