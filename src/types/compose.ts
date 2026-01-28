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

export interface DraftState {
	// Current draft being edited
	currentDraft: ComposeDraft | null

	// All drafts for current account
	drafts: ComposeDraft[]

	// UI state
	isComposing: boolean
	isDirty: boolean
	isSaving: boolean
	lastSavedAt?: Date
	editorMode: 'rich-text' | 'source'

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
	loadDrafts: (accountId: string) => Promise<void>
	deleteDraft: (draftId: string) => Promise<void>
}
