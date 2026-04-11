import type { AdvancedSearchQuery } from '@/types/search'

export function serializeSearchQuery(query: AdvancedSearchQuery): string {
	const parts: string[] = []

	if (query.rawQuery?.trim()) parts.push(query.rawQuery.trim())
	if (query.from?.trim()) {
		const val = query.from.trim()
		parts.push(`from:${val.includes(' ') ? `"${val}"` : val}`)
	}
	if (query.to?.trim()) {
		const val = query.to.trim()
		parts.push(`to:${val.includes(' ') ? `"${val}"` : val}`)
	}
	if (query.subject?.trim()) {
		const val = query.subject.trim()
		parts.push(`subject:${val.includes(' ') ? `"${val}"` : val}`)
	}
	if (query.dateFrom?.trim()) parts.push(`after:${query.dateFrom.trim()}`)
	if (query.dateTo?.trim()) parts.push(`before:${query.dateTo.trim()}`)
	if (query.hasAttachment) parts.push('has:attachment')

	return parts.join(' ')
}

export const SEARCH_OPERATORS = ['from:', 'to:', 'subject:', 'before:', 'after:', 'has:']

export function parseSearchOperators(input: string): AdvancedSearchQuery {
	const query: AdvancedSearchQuery = {}
	let rawQuery = ''

	// Split by operators
	const regex = /(?:^|\s)(from:|to:|subject:|before:|after:|has:)/i
	const parts = input.split(regex).filter(Boolean)

	let currentOp: string | null = null

	for (const part of parts) {
		const trimmed = part.trim()
		if (!trimmed) continue

		const lowerPart = trimmed.toLowerCase()

		if (SEARCH_OPERATORS.includes(lowerPart)) {
			currentOp = lowerPart
			continue
		}

		if (
			currentOp === 'subject:' ||
			currentOp === 'from:' ||
			currentOp === 'to:' ||
			currentOp === 'before:' ||
			currentOp === 'after:' ||
			currentOp === 'has:'
		) {
			let val = ''
			let rem = ''

			if (trimmed.startsWith('"')) {
				const endQuote = trimmed.indexOf('"', 1)
				if (endQuote !== -1) {
					val = trimmed.substring(1, endQuote)
					rem = trimmed.substring(endQuote + 1).trim()
				} else {
					const words = trimmed.split(/\s+/)
					val = words[0]
					rem = words.slice(1).join(' ')
				}
			} else {
				const words = trimmed.split(/\s+/)
				val = words[0]
				rem = words.slice(1).join(' ')
			}

			if (currentOp === 'from:') query.from = val
			else if (currentOp === 'to:') query.to = val
			else if (currentOp === 'subject:')
				query.subject = query.subject ? `${query.subject} ${val}` : val
			else if (currentOp === 'before:') query.dateTo = val
			else if (currentOp === 'after:') query.dateFrom = val
			else if (currentOp === 'has:') {
				if (val.toLowerCase() === 'attachment') {
					query.hasAttachment = true
				} else {
					rem = val + (rem ? ' ' + rem : '')
					rawQuery += (rawQuery ? ' ' : '') + 'has:'
				}
			}

			if (rem) {
				rawQuery += (rawQuery ? ' ' : '') + rem
			}
			currentOp = null
		} else {
			rawQuery += (rawQuery ? ' ' : '') + trimmed
		}
	}

	if (rawQuery.trim()) {
		query.rawQuery = rawQuery.trim()
	}

	return query
}
