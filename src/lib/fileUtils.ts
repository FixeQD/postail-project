import type { LucideIcon } from 'lucide-react'
import { File, FileImage, FileText } from 'lucide-react'

export async function fileToBytes(file: File): Promise<Uint8Array> {
	return new Uint8Array(await file.arrayBuffer())
}

export function getFileIcon(contentType: string | undefined): LucideIcon {
	if (!contentType) return File
	if (contentType.startsWith('image/')) return FileImage
	if (contentType.startsWith('text/')) return FileText
	return File
}
