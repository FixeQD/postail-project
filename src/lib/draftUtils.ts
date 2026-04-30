import { invoke } from '@tauri-apps/api/core'
import { parseAddress, parseAddresses } from '@/lib/parseAddress'
import i18n from '@/i18n'
import { useAccountStore } from '@/stores/accountStore'
import type { EmailAddress, DraftState } from '@/types/compose'
import type { Signature } from '@/types/signatures'

export const prepareReplyBase = (originalMessage: {
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

	return { fromParsed, originalSubject, subject, dateStr, quotedBody }
}

export const buildReplyAllRecipients = (
	originalMessage: {
		header: {
			from: string[]
			to?: string[]
			cc?: string[]
		}
	},
	fromParsed: ReturnType<typeof parseAddress>
): { toRecipients: EmailAddress[]; ccRecipients: EmailAddress[] } => {
	const userEmail = useAccountStore.getState().activeAccount?.email

	const toRecipients: EmailAddress[] = fromParsed.email
		? [{ email: fromParsed.email, name: fromParsed.name }]
		: []

	const originalTo = parseAddresses(originalMessage.header.to ?? [])
	const originalCc = parseAddresses(originalMessage.header.cc ?? [])
	const ccRecipients: EmailAddress[] = []

	originalTo.forEach((recp) => {
		if (recp.email === fromParsed.email || recp.email === userEmail) return
		if (!ccRecipients.some((r) => r.email === recp.email)) ccRecipients.push(recp)
	})

	originalCc.forEach((recp) => {
		if (recp.email === fromParsed.email || recp.email === userEmail) return
		if (!ccRecipients.some((r) => r.email === recp.email)) ccRecipients.push(recp)
	})

	return { toRecipients, ccRecipients }
}

export const buildReplyQuoteHtml = (
	date: string,
	fromName: string,
	fromEmail: string,
	quotedBody: string
) => `
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

export const buildForwardQuoteHtml = (
	subject: string,
	fromName: string,
	fromEmail: string,
	date: string,
	to: string,
	quotedBody: string
) => `
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

export const injectDefaultSignature = async (
	accountId: string,
	get: () => DraftState,
	set: (partial: Partial<DraftState>) => void
): Promise<void> => {
	try {
		const sig = await invoke<Signature | null>('get_default_signature', { accountId })
		if (!sig) return

		const { currentDraft } = get()
		if (!currentDraft) return

		const sigBlock = `<!-- SIGNATURE_START --><br><br><div class="signature-wrapper"><div class="signature">${sig.htmlContent}</div></div><!-- SIGNATURE_END -->`

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

export const buildForwardToString = (originalMessage: {
	header: { to?: string[] }
}): string => {
	const toRecipients = parseAddresses(originalMessage.header.to ?? [])
	return toRecipients
		.map((r) => (r.name ? `${r.name} <${r.email}>` : r.email))
		.join(', ')
}

export const escapePlainBodyAsHtml = (plain: string): string =>
	plain
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/\n/g, '<br>')

export const makeDraftId = (existing?: string): string => existing ?? crypto.randomUUID()

export const toUnixSeconds = (iso: string): number => {
	const v = Math.floor(Date.parse(iso) / 1000)
	return Number.isFinite(v) ? v : Math.floor(Date.now() / 1000)
}

export const prepareForwardBase = (originalMessage: {
	header: { from: string[]; subject?: string; internal_date: string; to?: string[] }
	body_html_safe?: string
	body_plain?: string
}) => {
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

	const hasHtml = !!originalMessage.body_html_safe?.trim()
	const rawBody = hasHtml
		? originalMessage.body_html_safe!.trim()
		: originalMessage.body_plain?.trim() || ''

	const quotedBody = hasHtml ? rawBody : escapePlainBodyAsHtml(rawBody)

	return { fromParsed, originalSubject, subject, dateStr, quotedBody }
}
