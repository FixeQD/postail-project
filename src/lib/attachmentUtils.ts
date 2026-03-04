import { invoke } from '@tauri-apps/api/core'
import type { EmailAttachment } from '@/types/compose'

export async function processFileAsAttachment(
	bytes: Uint8Array,
	filename: string,
	contentType: string,
	mode: 'inline' | 'attachment'
): Promise<EmailAttachment> {
	const isInlineImage = mode === 'inline' && contentType.startsWith('image/')
	return invoke<EmailAttachment>(
		isInlineImage ? 'add_inline_attachment' : 'add_attachment_bytes',
		{ bytes, filename, contentType }
	)
}
