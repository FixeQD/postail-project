import { useRef, useEffect, useCallback, memo } from 'react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useDraftStore } from '@/stores/draftStore'
import { fileToBytes } from '@/lib/fileUtils'
import { processFileAsAttachment } from '@/lib/attachmentUtils'
import type { EmailAttachment } from '@/types/compose'

export interface WysiwygEditorProps {
	value: string
	onChange: (html: string) => void
	placeholder?: string
	className?: string
}

interface CaretPosition {
	node: Node
	offset: number
}

function saveCaretPosition(container: HTMLElement): CaretPosition | null {
	const sel = window.getSelection()
	if (!sel || sel.rangeCount === 0) return null

	const range = sel.getRangeAt(0)
	if (!container.contains(range.startContainer)) return null

	return { node: range.startContainer, offset: range.startOffset }
}

function restoreCaretPosition(container: HTMLElement, pos: CaretPosition | null) {
	if (!pos || !container.contains(pos.node)) return

	try {
		const sel = window.getSelection()
		if (!sel) return
		const range = document.createRange()
		range.setStart(pos.node, pos.offset)
		range.collapse(true)
		sel.removeAllRanges()
		sel.addRange(range)
	} catch {
		// node may have been removed during innerHTML swap
	}
}

export const WysiwygEditor = memo(
	({ value, onChange, placeholder, className }: WysiwygEditorProps) => {
		const editorRef = useRef<HTMLDivElement>(null)
		const isInternalChange = useRef(false)
		const lastExternalValue = useRef(value)
		const savedRangeRef = useRef<Range | null>(null)
		const { addAttachment } = useDraftStore()

		// Sync external value -> DOM (only when it actually changed externally)
		useEffect(() => {
			const el = editorRef.current
			if (!el) return
			if (isInternalChange.current) {
				isInternalChange.current = false
				return
			}

			// Skip if DOM already matches
			if (el.innerHTML === value) {
				lastExternalValue.current = value
				return
			}

			const caret = saveCaretPosition(el)
			el.innerHTML = value
			lastExternalValue.current = value
			restoreCaretPosition(el, caret)
		}, [value])

		const handleInput = useCallback(() => {
			const el = editorRef.current
			if (!el) return

			isInternalChange.current = true
			const html = el.innerHTML
			lastExternalValue.current = html
			onChange(html)
		}, [onChange])

		// Insert an <img> at the saved cursor position or at end if nothing was saved
		const insertImageAtCursor = useCallback(
			(attachment: EmailAttachment) => {
				const el = editorRef.current
				if (!el) return

				const assetUrl = convertFileSrc(attachment.path!)
				const img = document.createElement('img')
				img.src = assetUrl
				img.alt = attachment.filename
				img.setAttribute('data-attachment-id', attachment.id)
				if (attachment.cid) img.setAttribute('data-cid', attachment.cid)

				const sel = window.getSelection()
				let range: Range

				if (
					savedRangeRef.current &&
					el.contains(savedRangeRef.current.commonAncestorContainer)
				) {
					range = savedRangeRef.current
				} else {
					// Nothing saved
					range = document.createRange()
					range.selectNodeContents(el)
					range.collapse(false)
				}

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

				isInternalChange.current = true
				onChange(el.innerHTML)
			},
			[onChange]
		)

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

		const handlePaste = useCallback(
			async (e: React.ClipboardEvent<HTMLDivElement>) => {
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
									const file = new File(
										[blob],
										`pasted_image_${Date.now()}.${ext}`,
										{ type }
									)
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

		const isEmpty = !value || value === '<br>' || value === '<p><br></p>' || value.trim() === ''

		return (
			<div className={`wysiwyg-editor-wrapper relative flex-1 ${className ?? ''}`}>
				{isEmpty && placeholder && (
					<div className='pointer-events-none absolute top-4 left-4 text-sm text-[var(--compose-placeholder)] select-none'>
						{placeholder}
					</div>
				)}
				<div
					ref={editorRef}
					contentEditable
					suppressContentEditableWarning
					className='wysiwyg-editable h-full min-h-50 w-full p-4 text-sm text-[var(--compose-text)] outline-none focus:outline-none'
					onInput={handleInput}
					onPaste={handlePaste}
					spellCheck
				/>
				<style>{`
				.wysiwyg-editable {
					word-wrap: break-word;
					overflow-wrap: break-word;
					white-space: pre-wrap;
					line-height: 1.6;
					caret-color: var(--compose-text);
				}
				.wysiwyg-editable:empty::before {
					content: '';
				}
				.wysiwyg-editable a {
					color: #22d3ee;
					text-decoration: underline;
					cursor: pointer;
				}
				.wysiwyg-editable img {
					display: block;
					max-width: 100%;
					border-radius: 8px;
					margin: 8px 0;
				}
				.wysiwyg-editable ul,
				.wysiwyg-editable ol {
					margin-left: 1rem;
					padding-left: 0.5rem;
				}
				.wysiwyg-editable ul { list-style-type: disc; }
				.wysiwyg-editable ol { list-style-type: decimal; }
				.wysiwyg-editable ul ::marker,
				.wysiwyg-editable ol ::marker {
					color: #e4e4e7;
				}
				.wysiwyg-editable .signature {
					border-top: 1px solid #e4e4e7;
					color: #71717a;
					padding-top: 8px;
					margin-top: 8px;
				}
				.wysiwyg-editable blockquote {
					border-left: 3px solid var(--compose-separator);
					padding-left: 12px;
					margin: 8px 0;
					color: var(--compose-text-muted);
				}
				.wysiwyg-editable h1 { font-size: 1.5em; font-weight: 700; margin: 0.5em 0; }
				.wysiwyg-editable h2 { font-size: 1.25em; font-weight: 600; margin: 0.4em 0; }
				.wysiwyg-editable h3 { font-size: 1.1em; font-weight: 600; margin: 0.3em 0; }
				.wysiwyg-editable p { margin: 0; }
			`}</style>
			</div>
		)
	}
)

export default WysiwygEditor
