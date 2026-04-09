import type { AdvancedSearchQuery } from '@/components/TitleBar/SearchBar'

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

		if (currentOp === 'subject:') {
			query.subject = query.subject ? `${query.subject} ${trimmed}` : trimmed
			currentOp = null
		} else if (currentOp === 'from:') {
			const words = trimmed.split(/\s+/)
			query.from = words[0]
			if (words.length > 1) {
				rawQuery += (rawQuery ? ' ' : '') + words.slice(1).join(' ')
			}
			currentOp = null
		} else if (currentOp === 'to:') {
			const words = trimmed.split(/\s+/)
			query.to = words[0]
			if (words.length > 1) {
				rawQuery += (rawQuery ? ' ' : '') + words.slice(1).join(' ')
			}
			currentOp = null
		} else if (currentOp === 'before:') {
			const words = trimmed.split(/\s+/)
			query.dateTo = words[0]
			if (words.length > 1) {
				rawQuery += (rawQuery ? ' ' : '') + words.slice(1).join(' ')
			}
			currentOp = null
		} else if (currentOp === 'after:') {
			const words = trimmed.split(/\s+/)
			query.dateFrom = words[0]
			if (words.length > 1) {
				rawQuery += (rawQuery ? ' ' : '') + words.slice(1).join(' ')
			}
			currentOp = null
		} else if (currentOp === 'has:') {
			const words = trimmed.split(/\s+/)
			if (words[0].toLowerCase() === 'attachment') {
				query.hasAttachment = true
			} else {
				rawQuery += (rawQuery ? ' ' : '') + 'has:' + words[0]
			}
			if (words.length > 1) {
				rawQuery += (rawQuery ? ' ' : '') + words.slice(1).join(' ')
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
