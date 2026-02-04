import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { html_beautify } from 'js-beautify'
import type {
	ComposeDraft,
	EmailAddress,
	EmailAttachment,
	DraftState,
	SanitizeIssue,
} from '@/types/compose'

const formatOptions: import('js-beautify').HTMLBeautifyOptions = {
	indent_size: 1,
	indent_char: '\t',
	max_preserve_newlines: 1,
	preserve_newlines: true,
	wrap_line_length: 0,
	wrap_attributes: 'auto',
	wrap_attributes_indent_size: 1,
	end_with_newline: false,
	indent_inner_html: true,
	extra_liners: [],
}

let validationTimer: ReturnType<typeof setTimeout> | null = null

export const useDraftStore = create<DraftState>((set, get) => ({
	// Initial state
	currentDraft: null,
	drafts: [],
	isComposing: false,
	isDirty: false,
	isSaving: false,
	isSending: false,
	lastSavedAt: undefined,
	editorMode: 'rich-text',

	compatibilityPanelOpen: false,
	compatibilityPanelWidth: 280,
	compatibilityIssues: [],
	isValidating: false,
	validationDismissed: false,
	showSendWarning: false,

	// Actions
	setCurrentDraft: (draft: ComposeDraft | null) => {
		set({ currentDraft: draft })
	},

	setEditorMode: (mode: 'rich-text' | 'source') => {
		set({ editorMode: mode })
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

	saveDraft: async (html?: string) => {
		const { currentDraft, isDirty, isSaving } = get()

		// Allow saving if dirty or if html override is provided
		if (!currentDraft || isSaving) return
		if (!isDirty && !html) return

		set({ isSaving: true })

		// If html override provided, sync it to currentDraft first
		if (html && html !== currentDraft.body) {
			set({
				currentDraft: {
					...currentDraft,
					body: html,
					updatedAt: new Date().toISOString(),
				},
				isDirty: true,
			})
		}

		const bodyToSave = html || currentDraft.body

		try {
			// Ensure we have an id before saving
			let draftId = currentDraft.id
			if (!draftId) {
				draftId = crypto.randomUUID()
				set({
					currentDraft: { ...currentDraft, id: draftId },
				})
			}

			const draftForRust = {
				id: draftId,
				accountId: currentDraft.accountId,
				subject: currentDraft.subject || null,
				body: bodyToSave || null,
				to: currentDraft.to.map((r) => r.email),
				cc: currentDraft.cc?.map((r) => r.email) || [],
				bcc: currentDraft.bcc?.map((r) => r.email) || [],
				attachments: currentDraft.attachments || [],
				createdAt: Math.floor(Date.parse(currentDraft.createdAt) / 1000),
				updatedAt: Math.floor(Date.parse(currentDraft.updatedAt) / 1000),
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

	loadDrafts: async (accountId: string, signal?: AbortSignal) => {
		try {
			const draftsFromRust = await invoke<any[]>('list_drafts', { accountId })
			// Check for cancellation before updating state
			if (signal?.aborted) return accountId
			const drafts: ComposeDraft[] = draftsFromRust.map((d) => ({
				id: d.id,
				accountId: d.accountId,
				to: d.to.map((email: string) => ({ email })),
				cc: d.cc?.map((email: string) => ({ email })) || [],
				bcc: d.bcc?.map((email: string) => ({ email })) || [],
				subject: d.subject || '',
				body: d.body || '',
				bodyType: 'html',
				attachments: d.attachments || [],
				createdAt: new Date(d.createdAt * 1000).toISOString(),
				updatedAt: new Date(d.updatedAt * 1000).toISOString(),
			}))
			set({ drafts })
			return accountId
		} catch (error) {
			console.error('Failed to load drafts:', error)
			set({ drafts: [] })
			return accountId
		}
	},

	deleteDraft: async (draftId: string) => {
		try {
			await invoke('delete_draft', { id: draftId })

			const { drafts, currentDraft } = get()
			const updatedDrafts = drafts.filter((d) => d.id !== draftId)
			const isRemovingCurrent = currentDraft?.id === draftId

			set({
				drafts: updatedDrafts,
				currentDraft: isRemovingCurrent ? null : currentDraft,
				isComposing: isRemovingCurrent ? false : get().isComposing,
				isDirty: isRemovingCurrent ? false : get().isDirty,
			})
		} catch (error) {
			console.error('Failed to delete draft:', error)
		}
	},

	sendDraft: async (html?: string) => {
		const {
			currentDraft,
			compatibilityIssues,
			validationDismissed,
			isValidating,
			validateCompatibility,
		} = get()
		if (!currentDraft || !currentDraft.id) {
			throw new Error('No draft to send')
		}

		// Wait for any in-flight validation or run validation immediately
		if (isValidating) {
			// Validation is in progress, run immediate validation to wait for results
			await validateCompatibility(html || currentDraft.body, true)
		} else {
			// No validation running, check if we need to validate
			const hasErrors = compatibilityIssues.some((i) => i.severity === 'Error')
			const hasWarnings = compatibilityIssues.some(
				(i) => i.severity === 'Warning' || i.severity === 'Info'
			)

			if (
				compatibilityIssues.length === 0 ||
				hasErrors ||
				(hasWarnings && !validationDismissed)
			) {
				// Need to validate first
				await validateCompatibility(html || currentDraft.body, true)
			}
		}

		// Re-check issues after potential validation
		const { compatibilityIssues: updatedIssues, validationDismissed: updatedDismissed } = get()
		const hasErrors = updatedIssues.some((i) => i.severity === 'Error')
		const hasWarnings = updatedIssues.some(
			(i) => i.severity === 'Warning' || i.severity === 'Info'
		)

		if (hasErrors || (hasWarnings && !updatedDismissed)) {
			set({ showSendWarning: true })
			throw new Error('Compatibility issues found')
		}

		set({ isSending: true })

		try {
			await get().saveDraft(html)

			const result = await invoke<{ eml_bytes: number[]; html_with_cids: string }>(
				'build_email_from_draft',
				{ draftId: currentDraft.id }
			)

			const emlBytes = new Uint8Array(result.eml_bytes)

			const outboxId = await invoke<string>('enqueue_message', {
				accountId: currentDraft.accountId,
				rawEml: Array.from(emlBytes),
			})

			console.log('Draft sent, outbox ID:', outboxId)

			set({
				currentDraft: null,
				isComposing: false,
				isDirty: false,
				isSending: false,
				compatibilityPanelOpen: false,
				compatibilityIssues: [],
				validationDismissed: false,
			})

			return outboxId
		} catch (error) {
			console.error('Failed to send draft:', error)
			set({ isSending: false })
			throw error
		}
	},

	toggleCompatibilityPanel: () => {
		const { compatibilityPanelOpen, editorMode } = get()
		if (!compatibilityPanelOpen && editorMode !== 'source') return
		set({ compatibilityPanelOpen: !compatibilityPanelOpen })
	},

	setCompatibilityPanelWidth: (width: number) => {
		const clampedWidth = Math.max(200, Math.min(500, width))
		const current = get().compatibilityPanelWidth
		if (Math.abs(current - clampedWidth) > 5) {
			set({ compatibilityPanelWidth: clampedWidth })
		}
	},

	validateCompatibility: async (html: string, immediate = false) => {
		if (validationTimer) {
			clearTimeout(validationTimer)
		}

		const runValidation = async () => {
			set({ isValidating: true })
			try {
				const result = await invoke<{
					html: string
					issues: SanitizeIssue[]
				}>('process_email_content', { html })

				set({
					compatibilityIssues: result.issues,
					isValidating: false,
					validationDismissed:
						result.issues.length === 0 ? false : get().validationDismissed,
				})
			} catch (error) {
				console.error('Failed to validate compatibility:', error)
				set({ isValidating: false })
			}
		}

		set({ compatibilityIssues: [] })

		if (immediate) {
			await runValidation()
		} else {
			validationTimer = setTimeout(runValidation, 800)
		}
	},

	applyAutoFix: async (html: string) => {
		set({ isValidating: true })
		try {
			const fixedHtml = await invoke<string>('auto_fix_email_html', { html })

			const formattedHtml = html_beautify(fixedHtml, formatOptions)

			// Update draft body with fixed HTML
			const { currentDraft } = get()
			if (currentDraft) {
				set({
					currentDraft: {
						...currentDraft,
						body: formattedHtml,
						updatedAt: new Date().toISOString(),
					},
					isDirty: true,
				})
			}

			// Re-run validation to show if there are still issues
			const validationResult = await invoke<{
				html: string
				issues: SanitizeIssue[]
			}>('process_email_content', { html: formattedHtml })

			set({
				compatibilityIssues: validationResult.issues,
				isValidating: false,
			})

			return formattedHtml
		} catch (error) {
			console.error('Failed to apply auto-fix:', error)
			set({ isValidating: false })
			throw error
		}
	},

	dismissValidationWarning: () => {
		set({ validationDismissed: true, showSendWarning: false })
	},

	resetValidationDismissed: () => {
		set({ validationDismissed: false })
	},

	setShowSendWarning: (show: boolean) => {
		set({ showSendWarning: show })
	},

	triggerAttachFile: () => {
		window.dispatchEvent(new CustomEvent('compose:attach-file'))
	},

	triggerInsertLink: () => {
		window.dispatchEvent(new CustomEvent('compose:insert-link'))
	},
}))
