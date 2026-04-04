export interface Mailbox {
	name: string
	display_name: string
	role: string
	uid_validity?: number
	highest_modseq?: number
	last_synced_uid?: number
	hidden?: boolean
}

// Keep in sync with src-tauri/src/db/mod.rs => MailHeader
export interface MailHeader {
	uid: number
	mailbox: string
	message_id?: string
	internal_date: string // DateTime<Utc> serialized to string
	subject?: string
	from: string[]
	to: string[]
	cc: string[]
	flags: string[]
	snippet?: string
	has_attachments: boolean
	starred: boolean
	tags: string[]
}

export interface ParsedAddress {
	name: string
	email: string
}

export interface AttachmentMeta {
	part_id: string
	filename?: string
	mime_type: string
	size: number
	cached_path?: string
	cid?: string
}

export interface MessageFull {
	header: MailHeader
	body_html_safe: string
	body_plain: string
	attachments: AttachmentMeta[]
	inline_images: AttachmentMeta[]
	read_receipt_to?: string | null
}

export interface ThreadMessage {
	header: MailHeader
	body_html_safe: string
	body_plain: string
	is_current: boolean
}

export interface ThreadView {
	messages: ThreadMessage[]
}
