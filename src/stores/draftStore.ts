import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import type { ComposeDraft, EmailAddress, EmailAttachment, DraftState } from '@/types/compose'

export const useDraftStore = create<DraftState>((set, get) => ({
	// Initial state
	currentDraft: null,
	drafts: [],
	isComposing: false,
	isDirty: false,
	isSaving: false,
	lastSavedAt: undefined,

	// Actions
	setCurrentDraft: (draft: ComposeDraft | null) => {
		set({ currentDraft: draft })
	},

	updateCurrentDraft: (updates: Partial<ComposeDraft>) => {
		const { currentDraft } = get()
		if (!currentDraft) return

		const updatedDraft = {
			...currentDraft,
			...updates,
			updatedAt: new Date().toISOString(),
		}

		set({
			currentDraft: updatedDraft,
			isDirty: true,
		})
	},

	addRecipient: (type: 'to' | 'cc' | 'bcc', recipient: EmailAddress) => {
		const { currentDraft } = get()
		if (!currentDraft) return

		const recipients = currentDraft[type] || []

		// Check if recipient already exists
		if (recipients.some((r) => r.email === recipient.email)) return

		const updatedDraft = {
			...currentDraft,
			[type]: [...recipients, recipient],
			updatedAt: new Date().toISOString(),
		}

		set({
			currentDraft: updatedDraft,
			isDirty: true,
		})
	},

	removeRecipient: (type: 'to' | 'cc' | 'bcc', email: string) => {
		const { currentDraft } = get()
		if (!currentDraft) return

		const recipients = currentDraft[type] || []
		const updatedDraft = {
			...currentDraft,
			[type]: recipients.filter((r) => r.email !== email),
			updatedAt: new Date().toISOString(),
		}

		set({
			currentDraft: updatedDraft,
			isDirty: true,
		})
	},

	setSubject: (subject: string) => {
		get().updateCurrentDraft({ subject })
	},

	setBody: (body: string) => {
		get().updateCurrentDraft({ body })
	},

	addAttachment: (attachment: EmailAttachment) => {
		const { currentDraft } = get()
		if (!currentDraft) return

		const attachments = currentDraft.attachments || []

		// Check if attachment already exists
		if (attachments.some((a) => a.id === attachment.id)) return

		const updatedDraft = {
			...currentDraft,
			attachments: [...attachments, attachment],
			updatedAt: new Date().toISOString(),
		}

		set({
			currentDraft: updatedDraft,
			isDirty: true,
		})
	},

	removeAttachment: (attachmentId: string) => {
		const { currentDraft } = get()
		if (!currentDraft) return

		const attachments = currentDraft.attachments || []
		const updatedDraft = {
			...currentDraft,
			attachments: attachments.filter((a) => a.id !== attachmentId),
			updatedAt: new Date().toISOString(),
		}

		set({
			currentDraft: updatedDraft,
			isDirty: true,
		})
	},

	startComposing: (accountId: string, draftOverrides?: Partial<ComposeDraft>) => {
		const newDraft: ComposeDraft = {
			id: crypto.randomUUID(),
			accountId,
			to: [],
			cc: [],
			bcc: [],
			subject: '',
			body: '',
			bodyType: 'html',
			attachments: [],
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString(),
			...draftOverrides,
		}

		set({
			currentDraft: newDraft,
			isComposing: true,
			isDirty: false,
		})
	},

	loadDraft: (draft: ComposeDraft) => {
		set({
			currentDraft: draft,
			isComposing: true,
			isDirty: false,
		})
	},

	stopComposing: () => {
		set({
			currentDraft: null,
			isComposing: false,
			isDirty: false,
		})
	},

	markDirty: () => {
		set({ isDirty: true })
	},

	markClean: () => {
		set({
			isDirty: false,
			lastSavedAt: new Date(),
		})
	},

	saveDraft: async () => {
		const { currentDraft, isDirty } = get()

		if (!currentDraft || !isDirty) return

		set({ isSaving: true })
		console.log('Zapisuję draft do bazy...', currentDraft)

		try {
			const draftForRust = {
				id: currentDraft.id!,
				account_id: currentDraft.accountId,
				subject: currentDraft.subject || null,
				body: currentDraft.body || null,
				to: currentDraft.to.map((r) => r.email),
				created_at: Math.floor(Date.parse(currentDraft.createdAt) / 1000),
				updated_at: Math.floor(Date.parse(currentDraft.updatedAt) / 1000),
			}

			await invoke('save_draft', { draft: draftForRust })

			set({
				isSaving: false,
				isDirty: false,
				lastSavedAt: new Date(),
			})
		} catch (error) {
			console.error('Failed to save draft:', error)
			set({ isSaving: false })
		}
	},

	loadDrafts: async (accountId: string) => {
		try {
			const draftsFromRust = await invoke<any[]>('list_drafts', { accountId })
			const drafts: ComposeDraft[] = draftsFromRust.map((d) => ({
				id: d.id,
				accountId: d.account_id,
				to: d.to.map((email: string) => ({ email })),
				cc: [],
				bcc: [],
				subject: d.subject || '',
				body: d.body || '',
				bodyType: 'html',
				attachments: [],
				createdAt: new Date(d.created_at * 1000).toISOString(),
				updatedAt: new Date(d.updated_at * 1000).toISOString(),
			}))
			set({ drafts })
		} catch (error) {
			console.error('Failed to load drafts:', error)
			set({ drafts: [] })
		}
	},

	deleteDraft: async (draftId: string) => {
		try {
			await invoke('delete_draft', { id: draftId })

			const { drafts, currentDraft } = get()
			const updatedDrafts = drafts.filter((d) => d.id !== draftId)

			set({
				drafts: updatedDrafts,
				currentDraft: currentDraft?.id === draftId ? null : currentDraft,
			})
		} catch (error) {
			console.error('Failed to delete draft:', error)
		}
	},
}))
