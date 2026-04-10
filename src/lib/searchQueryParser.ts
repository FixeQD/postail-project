import type { AdvancedSearchQuery } from '@/types/search'

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
		} else if (
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
