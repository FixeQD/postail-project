import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { parseAddress, parseAddresses } from '@/lib/parseAddress'
import i18n from '@/i18n'
import { useAccountStore } from '@/stores/accountStore'
import type {
	ComposeDraft,
	EmailAddress,
	EmailAttachment,
	DraftState,
	SanitizeIssue,
} from '@/types/compose'
import type { DraftFromRust } from '@/types/stores'
import type { Signature } from '@/types/signatures'
import type { Template } from '@/types/templates'

let validationTimer: ReturnType<typeof setTimeout> | null = null

const prepareReplyBase = (originalMessage: {
	header: { from: string[]; subject?: string; internal_date: string }
	body_html_safe?: string
	body_plain?: string
}) => {
	const fromRaw = originalMessage.header.from[0] || ''
	const fromParsed = parseAddress(fromRaw)
	const originalSubject = originalMessage.header.subject || ''
	const subject = originalSubject.toLowerCase().startsWith('re:')
		? originalSubject
		: `Re: ${originalSubject}`

	const date = new Date(originalMessage.header.internal_date)
	const dateStr = date.toLocaleString(i18n.t('app.languageCode'), {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
	})

	const hasHtml = !!originalMessage.body_html_safe?.trim()
	const rawBody = hasHtml
		? originalMessage.body_html_safe!.trim()
		: originalMessage.body_plain?.trim() || ''

	const quotedBody = hasHtml
		? rawBody
		: rawBody
				.replace(/&/g, '&amp;')
				.replace(/</g, '&lt;')
				.replace(/>/g, '&gt;')
				.replace(/\n/g, '<br>')

	return {
		fromParsed,
		originalSubject,
		subject,
		dateStr,
		quotedBody,
	}
}

/**
 * Fetches the default signature for an account and appends it to the current draft body
 */
const injectDefaultSignature = async (
	accountId: string,
	get: () => DraftState,
	set: (partial: Partial<DraftState>) => void
): Promise<void> => {
	try {
		const sig = await invoke<Signature | null>('get_default_signature', { accountId })
		if (!sig) return

		const { currentDraft } = get()
		if (!currentDraft) return

		const sigBlock = `<br><br><div class="signature">${sig.htmlContent}</div>`

		set({
			currentDraft: {
				...currentDraft,
				body: currentDraft.body ? currentDraft.body + sigBlock : sigBlock,
			},
		})
	} catch (e) {
		console.error('[draftStore] Failed to fetch default signature:', e)
	}
}

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

		injectDefaultSignature(accountId, get, set)
	},

	startReply: (accountId, originalMessage) => {
		const { fromParsed, originalSubject, subject, dateStr, quotedBody } =
			prepareReplyBase(originalMessage)

		const replyDraft: ComposeDraft = {
			id: crypto.randomUUID(),
			accountId,
			to: fromParsed.email ? [{ email: fromParsed.email, name: fromParsed.name }] : [],
			cc: [],
			bcc: [],
			subject,
			body: '', // Start with empty body
			bodyType: 'html',
			attachments: [],
			replyContext: {
				subject: originalSubject,
				fromName: fromParsed.name,
				fromEmail: fromParsed.email || '',
				date: dateStr,
				body: quotedBody,
			},
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString(),
		}

		set({
			currentDraft: replyDraft,
			isComposing: true,
			isDirty: false,
		})

		injectDefaultSignature(accountId, get, set)
	},

	startReplyAll: (accountId, originalMessage) => {
		const userEmail = useAccountStore.getState().activeAccount?.email
		const { fromParsed, originalSubject, subject, dateStr, quotedBody } =
			prepareReplyBase(originalMessage)

		// To: always contains the sender
		const toRecipients: EmailAddress[] = fromParsed.email
			? [{ email: fromParsed.email, name: fromParsed.name }]
			: []

		// To & CC from original message
		const originalTo = parseAddresses(originalMessage.header.to)
		const originalCc = parseAddresses(originalMessage.header.cc)

		const ccRecipients: EmailAddress[] = []

		// Add original 'to' recipients to 'cc' of reply
		originalTo.forEach((recp) => {
			if (recp.email === fromParsed.email || recp.email === userEmail) return
			if (!ccRecipients.some((r) => r.email === recp.email)) ccRecipients.push(recp)
		})

		// Add original 'cc' recipients
		originalCc.forEach((recp) => {
			if (recp.email === fromParsed.email || recp.email === userEmail) return
			if (!ccRecipients.some((r) => r.email === recp.email)) ccRecipients.push(recp)
		})

		const replyDraft: ComposeDraft = {
			id: crypto.randomUUID(),
			accountId,
			to: toRecipients,
			cc: ccRecipients,
			bcc: [],
			subject,
			body: '',
			bodyType: 'html',
			attachments: [],
			replyContext: {
				subject: originalSubject,
				fromName: fromParsed.name,
				fromEmail: fromParsed.email || '',
				date: dateStr,
				body: quotedBody,
			},
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString(),
		}

		set({
			currentDraft: replyDraft,
			isComposing: true,
			isDirty: false,
		})

		injectDefaultSignature(accountId, get, set)
	},

	startForward: (accountId, originalMessage) => {
		const fromRaw = originalMessage.header.from[0] || ''
		const fromParsed = parseAddress(fromRaw)
		const originalSubject = originalMessage.header.subject || ''
		const subject = originalSubject.toLowerCase().startsWith('fwd:')
			? originalSubject
			: `Fwd: ${originalSubject}`

		const date = new Date(originalMessage.header.internal_date)
		const dateStr = date.toLocaleString(i18n.t('app.languageCode'), {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
		})

		const toRecipients = parseAddresses(originalMessage.header.to)
		const toStr = toRecipients
			.map((r) => (r.name ? `${r.name} <${r.email}>` : r.email))
			.join(', ')

		const hasHtml = !!originalMessage.body_html_safe?.trim()
		const rawBody = hasHtml
			? originalMessage.body_html_safe!.trim()
			: originalMessage.body_plain?.trim() || ''

		const quotedBody = hasHtml
			? rawBody
			: rawBody
					.replace(/&/g, '&amp;')
					.replace(/</g, '&lt;')
					.replace(/>/g, '&gt;')
					.replace(/\n/g, '<br>')

		const forwardDraft: ComposeDraft = {
			id: crypto.randomUUID(),
			accountId,
			to: [],
			cc: [],
			bcc: [],
			subject,
			body: '',
			bodyType: 'html',
			attachments: [],
			forwardContext: {
				subject: originalSubject,
				fromName: fromParsed.name,
				fromEmail: fromParsed.email || '',
				date: dateStr,
				to: toStr,
				body: quotedBody,
			},
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString(),
		}

		set({
			currentDraft: forwardDraft,
			isComposing: true,
			isDirty: false,
		})

		injectDefaultSignature(accountId, get, set)
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
				createdAt: (() => {
					const v = Math.floor(Date.parse(currentDraft.createdAt) / 1000)
					return Number.isFinite(v) ? v : Math.floor(Date.now() / 1000)
				})(),
				updatedAt: (() => {
					const v = Math.floor(Date.parse(currentDraft.updatedAt) / 1000)
					return Number.isFinite(v) ? v : Math.floor(Date.now() / 1000)
				})(),
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
		// Check for cancellation before starting the async operation
		if (signal?.aborted) return accountId
		try {
			const draftsFromRust = await invoke<unknown[]>('list_drafts', { accountId })
			// Check for cancellation after the async operation completes
			if (signal?.aborted) return accountId
			const validDrafts: ComposeDraft[] = []

			const isValidDraft = (x: unknown): x is DraftFromRust => {
				if (!x || typeof x !== 'object') return false
				const obj = x as Record<string, unknown>
				if (typeof obj.id !== 'string') return false
				if (typeof obj.accountId !== 'string') return false
				if (!Array.isArray(obj.to) || !obj.to.every((e) => typeof e === 'string'))
					return false
				if (obj.cc && !Array.isArray(obj.cc)) return false
				if (obj.bcc && !Array.isArray(obj.bcc)) return false
				if (typeof obj.createdAt !== 'number' || !Number.isFinite(obj.createdAt))
					return false
				if (typeof obj.updatedAt !== 'number' || !Number.isFinite(obj.updatedAt))
					return false
				if (obj.attachments && !Array.isArray(obj.attachments)) return false
				return true
			}

			for (const item of draftsFromRust) {
				if (!isValidDraft(item)) {
					console.warn('Skipping invalid draft from backend', item)
					continue
				}

				validDrafts.push({
					id: item.id,
					accountId: item.accountId,
					to: item.to.map((email: string) => ({ email })),
					cc: item.cc?.map((email: string) => ({ email })) || [],
					bcc: item.bcc?.map((email: string) => ({ email })) || [],
					subject: item.subject || '',
					body: item.body || '',
					bodyType: 'html',
					attachments: item.attachments || [],
					createdAt: new Date(item.createdAt * 1000).toISOString(),
					updatedAt: new Date(item.updatedAt * 1000).toISOString(),
				})
			}

			const drafts: ComposeDraft[] = validDrafts
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
			// Append reply context if exists
			let finalBody = html || currentDraft.body
			if (currentDraft.replyContext) {
				const { date, fromName, fromEmail, body: quotedBody } = currentDraft.replyContext
				const quoteHtml = `
<br>
<div class="gmail_quote gmail_quote_container">
	<div dir="ltr" class="gmail_attr">${date} ${fromName} &lt;<a href="mailto:${fromEmail}">${fromEmail}</a>&gt; napisał(a):<br></div>
	<details style="margin-top: 4px;">
		<summary style="cursor:pointer; color:#888; font-size:12px; margin:4px 0; border:1px solid #eee; padding:2px 8px; display:inline-block; border-radius:4px; list-style:none;">...</summary>
		<blockquote class="gmail_quote" style="margin:0px 0px 0px 0.8ex;border-left:1px solid rgb(204,204,204);padding-left:1ex">
			${quotedBody}
		</blockquote>
	</details>
</div>`
				finalBody += quoteHtml
			}

			if (currentDraft.forwardContext) {
				const {
					subject,
					fromName,
					fromEmail,
					date,
					to,
					body: quotedBody,
				} = currentDraft.forwardContext
				const forwardHtml = `
<br>
<div class="gmail_quote gmail_quote_container">
	<div dir="ltr" class="gmail_attr" style="color:#555; font-size:12px; margin-bottom:4px;">
		---------- Forwarded message ---------<br>
		<b>From:</b> ${fromName} &lt;<a href="mailto:${fromEmail}">${fromEmail}</a>&gt;<br>
		<b>Date:</b> ${date}<br>
		<b>Subject:</b> ${subject}<br>
		<b>To:</b> ${to}
	</div>
	<blockquote class="gmail_quote" style="margin:0px 0px 0px 0.8ex;border-left:1px solid rgb(204,204,204);padding-left:1ex">
		${quotedBody}
	</blockquote>
</div>`
				finalBody += forwardHtml
			}

			await get().saveDraft(finalBody)

			const result = await invoke<{ eml_bytes: number[]; html_with_cids: string }>(
				'build_email_from_draft',
				{
					draftId: currentDraft.id,
					requestReadReceipt: currentDraft.requestReadReceipt ?? false,
				}
			)

			const emlBytes = new Uint8Array(result.eml_bytes)

			const outboxId = await invoke<string>('enqueue_message', {
				accountId: currentDraft.accountId,
				rawEml: Array.from(emlBytes),
			})

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

			const { html_beautify } = await import('js-beautify')
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

	replaceSignature: (html: string | null) => {
		const { currentDraft } = get()
		if (!currentDraft) return

		const signatureBlockRegex = /<br\s*\/?><br\s*\/?><div class="signature">[\s\S]*?<\/div>/gi
		const currentBody = currentDraft.body || ''

		let updatedBody: string
		if (html === null) {
			updatedBody = currentBody.replace(signatureBlockRegex, '')
		} else {
			const sigBlock = `<br><br><div class="signature">${html}</div>`
			if (signatureBlockRegex.test(currentBody)) {
				updatedBody = currentBody.replace(signatureBlockRegex, sigBlock)
			} else {
				updatedBody = currentBody ? currentBody + sigBlock : sigBlock
			}
		}

		set({
			currentDraft: {
				...currentDraft,
				body: updatedBody,
				updatedAt: new Date().toISOString(),
			},
			isDirty: true,
		})
	},

	applyTemplate: (template: Template) => {
		const { currentDraft } = get()
		if (!currentDraft) return

		// If subject is empty, use template subject
		let subject = currentDraft.subject
		if (!subject.trim() && template.subject) {
			subject = template.subject
		}

		set({
			currentDraft: {
				...currentDraft,
				subject,
				body: template.htmlBody,
				updatedAt: new Date().toISOString(),
			},
			isDirty: true,
		})
	},
}))
