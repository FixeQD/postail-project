export interface Mailbox {
	name: string
	display_name: string
	role: string
	uid_validity?: number
	highest_modseq?: number
	last_synced_uid?: number
}

// Keep in sync with src-tauri/src/db/mod.rs => MailHeader
export interface MailHeader {
	uid: number
	message_id?: string
	internal_date: string // DateTime<Utc> serialized to string
	subject?: string
	from: string[]
	to: string[]
	flags: string[]
	snippet?: string
	has_attachments: boolean
}

export interface AttachmentMeta {
	part_id: string
	filename?: string
	mime_type: string
	size: number
}

export interface MessageFull {
	header: MailHeader
	body_html_safe: string
	body_plain: string
	attachments: AttachmentMeta[]
	inline_images: AttachmentMeta[]
}

export interface ParsedAddress {
	name: string
	email: string
}

export interface MessageViewSelection {
	accountId: string
	mailbox: string
	uid: number
}
