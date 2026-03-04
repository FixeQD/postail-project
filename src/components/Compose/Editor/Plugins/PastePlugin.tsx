import {
	$getSelection,
	$getRoot,
	$isRangeSelection,
	COMMAND_PRIORITY_CRITICAL,
	PASTE_COMMAND,
} from 'lexical'
import { useEffect } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useDraftStore } from '@/stores/draftStore'
import { $createImageNode } from '../Nodes/ImageNode'
import { fileToBytes } from '@/lib/fileUtils'
import { processFileAsAttachment } from '@/lib/attachmentUtils'
import type { EmailAttachment } from '@/types/compose'

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
			const bytes = await fileToBytes(file)

			const filename =
				file.name && file.name !== 'image.png'
					? file.name
					: `pasted_image_${Date.now()}.${file.type.split('/')[1] || 'png'}`

			const attachment = await processFileAsAttachment(bytes, filename, file.type, 'inline')

			addAttachment(attachment)

			insertInlineImage(attachment)
		} catch (err) {
			console.error('Failed to process pasted image:', err)
		}
	}

	const insertInlineImage = (attachment: EmailAttachment) => {
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
	}

	return null
}
