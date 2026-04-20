import { useEffect, useCallback, useRef } from 'react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useDraftStore } from '@/stores/draftStore'
import { fileToBytes } from '@/lib/fileUtils'
import { processFileAsAttachment } from '@/lib/attachmentUtils'
import type { EmailAttachment } from '@/types/compose'

export default function PastePlugin() {
	const { addAttachment } = useDraftStore()
	const savedRangeRef = useRef<Range | null>(null)

	// Insert an <img> at the saved cursor position or at end if nothing was saved
	const insertImageAtCursor = useCallback((attachment: EmailAttachment) => {
		const el = document.querySelector('.wysiwyg-editable') as HTMLDivElement | null
		if (!el) return

		const assetUrl = convertFileSrc(attachment.path!)
		const img = document.createElement('img')
		img.src = assetUrl
		img.alt = attachment.filename
		img.setAttribute('data-attachment-id', attachment.id)
		if (attachment.cid) img.setAttribute('data-cid', attachment.cid)

		const sel = window.getSelection()
		let range: Range

		if (savedRangeRef.current && el.contains(savedRangeRef.current.commonAncestorContainer)) {
			range = savedRangeRef.current
		} else {
			// Nothing saved or invalid
			range = document.createRange()
			range.selectNodeContents(el)
			range.collapse(false)
		}

		// --- Signature Protection ---
		const sigWrapper = el.querySelector('.signature-wrapper')
		if (sigWrapper) {
			// If range is inside sigWrapper or after it, move before it
			if (
				sigWrapper.contains(range.commonAncestorContainer) ||
				(range.startContainer === el &&
					range.startOffset > Array.from(el.childNodes).indexOf(sigWrapper))
			) {
				range.setStartBefore(sigWrapper)
				range.collapse(true)
			}
		}
		// ----------------------------

		sel?.removeAllRanges()
		sel?.addRange(range)
		range.deleteContents()
		range.insertNode(img)

		// Move cursor after the image
		const afterRange = document.createRange()
		afterRange.setStartAfter(img)
		afterRange.collapse(true)
		sel?.removeAllRanges()
		sel?.addRange(afterRange)
		savedRangeRef.current = afterRange

		// Trigger change event for editor to pick up
		el.dispatchEvent(new Event('compose:editor-change', { bubbles: true }))
	}, [])

	// Handle insert-inline-image custom event
	useEffect(() => {
		const handler = (e: Event) => {
			const attachment = (e as CustomEvent).detail as EmailAttachment
			insertImageAtCursor(attachment)
		}
		window.addEventListener('compose:insert-inline-image', handler)
		return () => window.removeEventListener('compose:insert-inline-image', handler)
	}, [insertImageAtCursor])

	// Process pasted image file
	const handleImageFile = useCallback(
		async (file: File) => {
			try {
				const bytes = await fileToBytes(file)
				const filename =
					file.name && file.name !== 'image.png'
						? file.name
						: `pasted_image_${Date.now()}.${file.type.split('/')[1] || 'png'}`

				const attachment = await processFileAsAttachment(
					bytes,
					filename,
					file.type,
					'inline'
				)
				addAttachment(attachment)
				insertImageAtCursor(attachment)
			} catch (err) {
				console.error('Failed to process pasted image:', err)
			}
		},
		[addAttachment, insertImageAtCursor]
	)

	// Main paste handler
	const handlePaste = useCallback(
		async (e: ClipboardEvent) => {
			const data = e.clipboardData
			if (!data) return

			// Save the cursor position right as paste fires
			const sel = window.getSelection()
			if (sel && sel.rangeCount > 0) {
				savedRangeRef.current = sel.getRangeAt(0).cloneRange()
			}

			let handled = false

			// Strategy 1: clipboardData.items
			for (const item of Array.from(data.items)) {
				if (item.type.startsWith('image/')) {
					const file = item.getAsFile()
					if (file) {
						e.preventDefault()
						await handleImageFile(file)
						handled = true
						break
					}
				}
			}

			if (handled) return

			// Strategy 2: clipboardData.files
			for (const file of Array.from(data.files)) {
				if (file.type.startsWith('image/')) {
					e.preventDefault()
					await handleImageFile(file)
					handled = true
					break
				}
			}

			if (handled) return

			// Strategy 3: navigator.clipboard API fallback
			if (data.types.length === 0) {
				try {
					const items = await navigator.clipboard.read()
					for (const item of items) {
						for (const type of item.types) {
							if (type.startsWith('image/')) {
								const blob = await item.getType(type)
								const ext = type.split('/')[1] || 'png'
								const file = new File([blob], `pasted_image_${Date.now()}.${ext}`, {
									type,
								})
								e.preventDefault()
								await handleImageFile(file)
								handled = true
								break
							}
						}
						if (handled) break
					}
				} catch {
					// Clipboard API unavailable or permission denied
				}
			}
		},
		[handleImageFile]
	)

	// Attach paste listener to document
	useEffect(() => {
		const el = document.querySelector('.wysiwyg-editable') as HTMLDivElement | null
		if (!el) return

		el.addEventListener('paste', handlePaste)
		return () => el.removeEventListener('paste', handlePaste)
	}, [handlePaste])

	return null
}
