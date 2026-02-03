import {
	$getSelection,
	$getRoot,
	$isRangeSelection,
	COMMAND_PRIORITY_CRITICAL,
	PASTE_COMMAND,
} from 'lexical'
import { useEffect } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { useDraftStore } from '@/stores/draftStore'
import { $createImageNode } from '../Nodes/ImageNode'
import type { EmailAttachment } from '@/types/compose'

/**
 * Intercepts paste events to detect image data, upload it as an inline attachment, and insert a corresponding image node into the editor.
 *
 * Registers both a Lexical paste command override and a native root-level paste listener, and cleans up those listeners when the plugin is unmounted.
 *
 * @returns `null` — this plugin does not render any UI
 */
export default function PastePlugin(): null {
	const [editor] = useLexicalComposerContext()
	const { addAttachment } = useDraftStore()

	useEffect(() => {
		const onPaste = async (event: ClipboardEvent) => {
			const data = event.clipboardData
			if (!data) return

			let handled = false

			if (data.items.length > 0) {
				for (const item of Array.from(data.items)) {
					if (item.type.startsWith('image/')) {
						const file = item.getAsFile()
						if (file) {
							await handleImagePaste(file)
							handled = true
						}
					}
				}
			}

			if (!handled && data.files.length > 0) {
				for (const file of Array.from(data.files)) {
					if (file.type.startsWith('image/')) {
						await handleImagePaste(file)
						handled = true
					}
				}
			}

			if (!handled && data.types.length === 0) {
				try {
					const items = await navigator.clipboard.read()
					for (const item of items) {
						for (const type of item.types) {
							if (type.startsWith('image/')) {
								const blob = await item.getType(type)
								const file = new File(
									[blob],
									`pasted_image_${Date.now()}.${type.split('/')[1] || 'png'}`,
									{ type }
								)
								await handleImagePaste(file)
								handled = true
								break
							}
						}
						if (handled) break
					}
				} catch {
					// Fallback failed
				}
			}

			if (handled) {
				event.preventDefault()
				event.stopPropagation()
			}
		}

		const unregisterLexical = editor.registerCommand(
			PASTE_COMMAND,
			() => false,
			COMMAND_PRIORITY_CRITICAL
		)

		const registerOnRoot = (root: HTMLElement) => {
			root.addEventListener('paste', onPaste, true)
			return () => root.removeEventListener('paste', onPaste, true)
		}

		const unregisterRoot = editor.registerRootListener((newRoot) => {
			if (newRoot) registerOnRoot(newRoot)
		})

		const currentRoot = editor.getRootElement()
		let cleanupNative: (() => void) | undefined
		if (currentRoot) cleanupNative = registerOnRoot(currentRoot)

		return () => {
			unregisterLexical()
			unregisterRoot()
			if (cleanupNative) cleanupNative()
		}
	}, [editor, addAttachment])

	const handleImagePaste = async (file: File) => {
		try {
			const buffer = await file.arrayBuffer()
			const bytes = new Uint8Array(buffer)

			const filename =
				file.name && file.name !== 'image.png'
					? file.name
					: `pasted_image_${Date.now()}.${file.type.split('/')[1] || 'png'}`

			const attachment = await invoke<EmailAttachment>('add_inline_attachment', {
				bytes,
				filename,
				contentType: file.type,
			})

			addAttachment(attachment)

			editor.update(() => {
				let selection = $getSelection()
				if (!$isRangeSelection(selection)) {
					const root = $getRoot()
					root.selectEnd()
					selection = $getSelection()
				}

				if ($isRangeSelection(selection)) {
					const assetUrl = convertFileSrc(attachment.path!)
					const node = $createImageNode({
						altText: 'Pasted image',
						attachmentId: attachment.id,
						cid: attachment.cid,
						src: assetUrl,
					})
					selection.insertNodes([node])
				}
			})
		} catch (err) {
			console.error('Failed to process pasted image:', err)
		}
	}

	return null
}