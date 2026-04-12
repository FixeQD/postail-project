export interface AdvancedSearchQuery {
	from?: string
	to?: string
	subject?: string
	body?: string
	dateFrom?: string
	dateTo?: string
	hasAttachment?: boolean
	folder?: string
	rawQuery?: string
}

export interface SavedSearch {
	id: string
	account_id: string
	name: string
	query_json: string
	icon: string
	position: number
	created_at: number
}

export interface SearchResult {
	message_id: number
	account_id: string
	mailbox: string
	uid: number
	subject?: string
	from_addr?: string
	snippet?: string
	rank: number
	has_attachments: boolean
	date: number
}
