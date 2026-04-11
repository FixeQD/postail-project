import type { AdvancedSearchQuery } from '@/types/search'

export function serializeSearchQuery(query: AdvancedSearchQuery): string {
	const parts: string[] = []

	if (query.rawQuery?.trim()) parts.push(query.rawQuery.trim())
	if (query.from?.trim()) {
		const val = query.from.trim().replace(/"/g, '""')
		parts.push(`from:${val.includes(' ') ? `"${val}"` : val}`)
	}
	if (query.to?.trim()) {
		const val = query.to.trim().replace(/"/g, '""')
		parts.push(`to:${val.includes(' ') ? `"${val}"` : val}`)
	}
	if (query.subject?.trim()) {
		const val = query.subject.trim().replace(/"/g, '""')
		parts.push(`subject:${val.includes(' ') ? `"${val}"` : val}`)
	}
	if (query.body?.trim()) {
		const val = query.body.trim().replace(/"/g, '""')
		parts.push(`body:${val.includes(' ') ? `"${val}"` : val}`)
	}
	if (query.folder?.trim()) {
		const val = query.folder.trim().replace(/"/g, '""')
		parts.push(`folder:${val.includes(' ') ? `"${val}"` : val}`)
	}
	if (query.dateFrom?.trim()) parts.push(`after:${query.dateFrom.trim()}`)
	if (query.dateTo?.trim()) parts.push(`before:${query.dateTo.trim()}`)
	if (query.hasAttachment) parts.push('has:attachment')

	return parts.join(' ')
}

export const SEARCH_OPERATORS = [
	'from:',
	'to:',
	'subject:',
	'body:',
	'folder:',
	'before:',
	'after:',
	'has:',
]

export function parseSearchOperators(input: string): AdvancedSearchQuery {
	const query: AdvancedSearchQuery = {}
	let rawQuery = ''

	// Split by operators
	const regex = /(?:^|\s)(from:|to:|subject:|body:|folder:|before:|after:|has:)/i
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

		if (currentOp && SEARCH_OPERATORS.includes(currentOp)) {
			let val = trimmed

			if (val.startsWith('"') && val.endsWith('"') && val.length >= 2) {
				val = val.substring(1, val.length - 1).replace(/""/g, '"')
			} else {
				val = val.replace(/""/g, '"')
			}

			if (currentOp === 'from:') query.from = val
			else if (currentOp === 'to:') query.to = val
			else if (currentOp === 'subject:')
				query.subject = query.subject ? `${query.subject} ${val}` : val
			else if (currentOp === 'body:') query.body = query.body ? `${query.body} ${val}` : val
			else if (currentOp === 'folder:') query.folder = val
			else if (currentOp === 'before:') query.dateTo = val
			else if (currentOp === 'after:') query.dateFrom = val
			else if (currentOp === 'has:') {
				if (val.toLowerCase() === 'attachment') {
					query.hasAttachment = true
				} else {
					rawQuery += (rawQuery ? ' ' : '') + 'has:' + val
				}
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
