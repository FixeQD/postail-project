import type { ComposeDraft } from '@/types/compose'

export interface TemplateVarContext {
	name?: string
	email?: string
	date?: string
	subject?: string
}

/**
 * Replaces variables in template text with actual values.
 */
export function resolveTemplateVariables(text: string, context: TemplateVarContext): string {
	if (!text) return ''

	return text
		.replace(/{{name}}/g, context.name || '')
		.replace(/{{email}}/g, context.email || '')
		.replace(/{{date}}/g, context.date || '')
		.replace(/{{subject}}/g, context.subject || '')
}

/**
 * Builds a TemplateVarContext from a draft.
 */
export function getTemplateContextFromDraft(draft: ComposeDraft | null): TemplateVarContext {
	const recipient = draft?.to?.[0]
	return {
		name: recipient?.name || recipient?.email || '',
		email: recipient?.email || '',
		date: new Date().toLocaleDateString(undefined, {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
		}),
		subject: draft?.replyContext?.subject || draft?.forwardContext?.subject || draft?.subject || '',
	}
}

/**
 * Returns placeholder context for previewing.
 */
export function getPlaceholderContext(): TemplateVarContext {
	return {
		name: 'John Doe',
		email: 'john@example.com',
		date: new Date().toLocaleDateString(undefined, {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
		}),
		subject: 'Re: Project Update',
	}
}
