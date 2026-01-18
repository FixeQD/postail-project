export interface AccountMeta {
	id: string
	name: string
	email: string
	provider_type: string
	auth_type: string
	imap_host: string
	imap_port: number
	imap_tls: boolean
	smtp_host: string
	smtp_port: number
	smtp_tls: boolean
	encryption_mode: string
	created_at: string
}
