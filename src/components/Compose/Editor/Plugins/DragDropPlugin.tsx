import React, { useEffect, useState, useCallback, useRef } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { $getSelection, $isRangeSelection, $createParagraphNode, $getRoot } from 'lexical'
import { useDraftStore } from '@/stores/draftStore'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { $createImageNode } from '../Nodes/ImageNode'
import type { EmailAttachment } from '@/types/compose'
import { UploadCloud, Image as ImageIcon, Paperclip } from 'lucide-react'

export default function DragDropPlugin(): React.ReactNode {
	const [editor] = useLexicalComposerContext()
	const { addAttachment } = useDraftStore()
	const [isDragging, setIsDragging] = useState(false)
	const [isProcessing, setIsProcessing] = useState(false)
	const [dragType, setDragType] = useState<'media' | 'file'>('file')
	const [activeZone, setActiveZone] = useState<'inline' | 'attachment' | null>(null)

	const dragCounter = useRef(0)
	const zonesRef = useRef<{ inline: DOMRect | null; attachment: DOMRect | null }>({
		inline: null,
		attachment: null,
	})
	const handleFileProcessingRef =
		useRef<(files: File[], uris: string[], mode: 'inline' | 'attachment') => Promise<void>>(
			undefined
		)

	const checkDragType = useCallback((dataTransfer: DataTransfer) => {
		const types = Array.from(dataTransfer.types)
		if (types.some((t) => t.startsWith('image/'))) return 'media'
		if (types.includes('Files') || types.includes('text/uri-list')) return 'media'
		return 'file'
	}, [])

	useEffect(() => {
		console.log('[DragDrop] Plugin mounted permanently, registering listeners')

		const handleDragEnter = (e: DragEvent) => {
			e.preventDefault()
			e.stopPropagation()
			dragCounter.current++

			if (e.dataTransfer && dragCounter.current === 1) {
				const type = checkDragType(e.dataTransfer)
				setDragType(type)
				setIsDragging(true)
			}
		}

		const handleDragOver = (e: DragEvent) => {
			e.preventDefault()
			e.stopPropagation()

			if (dragCounter.current > 0) {
				const inlineRect = zonesRef.current.inline
				const attachRect = zonesRef.current.attachment

				if (inlineRect && e.clientX >= inlineRect.left && e.clientX <= inlineRect.right) {
					setActiveZone('inline')
				} else if (
					attachRect &&
					e.clientX >= attachRect.left &&
					e.clientX <= attachRect.right
				) {
					setActiveZone('attachment')
				} else {
					setActiveZone(null)
				}
			}
		}

		const handleDragLeave = (e: DragEvent) => {
			e.preventDefault()
			e.stopPropagation()
			dragCounter.current--

			if (dragCounter.current <= 0) {
				setIsDragging(false)
				setActiveZone(null)
				dragCounter.current = 0
			}
		}

		const handleDrop = (e: DragEvent) => {
			console.log('[DragDrop] Native Window Drop detected!')
			e.preventDefault()
			e.stopPropagation()

			setIsDragging(false)
			dragCounter.current = 0

			if (!e.dataTransfer) return

			// Extract data SYNCHRONOUSLY
			const files = e.dataTransfer.files ? Array.from(e.dataTransfer.files) : []
			const uris = e.dataTransfer.getData('text/uri-list')
				? e.dataTransfer
						.getData('text/uri-list')
						.split('\n')
						.filter((u) => u.trim())
				: []

			// Mode detection based on pointer position at drop
			let mode: 'inline' | 'attachment' = 'attachment'
			const inlineRect = zonesRef.current.inline
			if (inlineRect && e.clientX >= inlineRect.left && e.clientX <= inlineRect.right) {
				mode = 'inline'
			}

			console.log(
				`[DragDrop] Processing. Mode: ${mode}, Files: ${files.length}, URIs: ${uris.length}`
			)
			handleFileProcessingRef.current?.(files, uris, mode)
			setActiveZone(null)
		}

		window.addEventListener('dragenter', handleDragEnter, true)
		window.addEventListener('dragover', handleDragOver, true)
		window.addEventListener('dragleave', handleDragLeave, true)
		window.addEventListener('drop', handleDrop, true)

		return () => {
			console.log('[DragDrop] Unmounting plugin, cleaning listeners')
			window.removeEventListener('dragenter', handleDragEnter, true)
			window.removeEventListener('dragover', handleDragOver, true)
			window.removeEventListener('dragleave', handleDragLeave, true)
			window.removeEventListener('drop', handleDrop, true)
		}
	}, [checkDragType])

	const insertInlineImage = useCallback(
		(attachment: EmailAttachment) => {
			editor.update(() => {
				const selection = $getSelection() || $getRoot().selectEnd()

				if ($isRangeSelection(selection)) {
					const assetUrl = convertFileSrc(attachment.path!)
					const node = $createImageNode({
						altText: attachment.filename,
						attachmentId: attachment.id,
						cid: attachment.cid,
						src: assetUrl,
					})
					selection.insertNodes([node])
					selection.insertNodes([$createParagraphNode()])
				}
			})
		},
		[editor]
	)

	const handleFileProcessing = useCallback(
		async (files: File[], uris: string[], mode: 'inline' | 'attachment') => {
			setIsProcessing(true)

			try {
				for (const file of files) {
					const buffer = await file.arrayBuffer()
					const bytes = new Uint8Array(buffer)

					const isInlineImage = mode === 'inline' && file.type.startsWith('image/')

					const attachment = await invoke<EmailAttachment>(
						isInlineImage ? 'add_inline_attachment' : 'add_attachment_bytes',
						{
							bytes,
							filename: file.name,
							contentType: file.type,
						}
					)

					addAttachment(attachment)
					if (isInlineImage) {
						insertInlineImage(attachment)
					}
				}

				for (const uri of uris) {
					const path = uri.startsWith('file://') ? decodeURIComponent(uri.slice(7)) : uri
					if (!path) continue

					let attachment: EmailAttachment

					if (mode === 'inline') {
						const response = await fetch(uri)
						const buffer = await response.arrayBuffer()
						const bytes = new Uint8Array(buffer)
						const contentType =
							response.headers.get('content-type') || 'application/octet-stream'
						const filename = path.split('/').pop() || 'attachment'

						const isInlineImage = contentType.startsWith('image/')
						attachment = await invoke<EmailAttachment>(
							isInlineImage ? 'add_inline_attachment' : 'add_attachment_bytes',
							{ bytes, filename, contentType }
						)
						addAttachment(attachment)
						if (isInlineImage) {
							insertInlineImage(attachment)
						}
					} else {
						attachment = await invoke<EmailAttachment>('add_attachment', { path })
						addAttachment(attachment)
					}
				}
			} catch (err) {
				console.error('[DragDrop] Processing failure:', err)
			} finally {
				setIsProcessing(false)
			}
		},
		[addAttachment, insertInlineImage]
	)
	handleFileProcessingRef.current = handleFileProcessing

	// ALWAYS return the container div, but hide/show the content
	return (
		<div
			className={`pointer-events-none fixed inset-0 z-[100] flex items-center justify-center bg-zinc-950/80 p-12 backdrop-blur-sm transition-all duration-200 ${!isDragging && !isProcessing ? 'scale-95 opacity-0' : 'scale-100 opacity-100'}`}
			style={{ visibility: !isDragging && !isProcessing ? 'hidden' : 'visible' }}>
			{isProcessing ? (
				<div className='pointer-events-auto flex flex-col items-center gap-4 text-white'>
					<div className='h-12 w-12 animate-spin rounded-full border-b-2 border-blue-500'></div>
					<p className='text-xl font-medium'>Processing items...</p>
				</div>
			) : (
				<div className='pointer-events-none flex h-96 w-full max-w-4xl gap-8'>
					{dragType === 'media' ? (
						<>
							<div
								ref={(el) => {
									if (el) zonesRef.current.inline = el.getBoundingClientRect()
								}}
								className={`flex flex-1 flex-col items-center justify-center gap-4 rounded-2xl border-4 border-dashed transition-all duration-200 ${
									activeZone === 'inline'
										? 'scale-105 border-blue-500 bg-blue-500/20 shadow-[0_0_30px_rgba(59,130,246,0.3)]'
										: 'border-zinc-700 bg-zinc-900/40'
								}`}>
								<div
									className={`rounded-full p-4 ${activeZone === 'inline' ? 'bg-blue-500 text-white' : 'bg-zinc-800 text-zinc-400'}`}>
									<ImageIcon size={48} />
								</div>
								<div className='text-center'>
									<h3
										className={`text-xl font-bold ${activeZone === 'inline' ? 'text-blue-200' : 'text-zinc-300'}`}>
										Insert Inline
									</h3>
									<p className='mt-1 text-zinc-500'>Embed directly in email</p>
								</div>
							</div>

							<div
								ref={(el) => {
									if (el) zonesRef.current.attachment = el.getBoundingClientRect()
								}}
								className={`flex flex-1 flex-col items-center justify-center gap-4 rounded-2xl border-4 border-dashed transition-all duration-200 ${
									activeZone === 'attachment'
										? 'scale-105 border-green-500 bg-green-500/20 shadow-[0_0_30px_rgba(34,197,94,0.3)]'
										: 'border-zinc-700 bg-zinc-900/40'
								}`}>
								<div
									className={`rounded-full p-4 ${activeZone === 'attachment' ? 'bg-green-500 text-white' : 'bg-zinc-800 text-zinc-400'}`}>
									<Paperclip size={48} />
								</div>
								<div className='text-center'>
									<h3
										className={`text-xl font-bold ${activeZone === 'attachment' ? 'text-green-200' : 'text-zinc-300'}`}>
										Add as Attachment
									</h3>
									<p className='mt-1 text-zinc-500'>Add to file list</p>
								</div>
							</div>
						</>
					) : (
						<div
							ref={(el) => {
								if (el) zonesRef.current.attachment = el.getBoundingClientRect()
							}}
							className={`flex flex-1 flex-col items-center justify-center gap-6 rounded-2xl border-4 border-dashed transition-all duration-200 ${
								activeZone === 'attachment'
									? 'scale-105 border-blue-500 bg-blue-500/20 shadow-[0_0_30px_rgba(59,130,246,0.3)]'
									: 'border-zinc-700 bg-zinc-900/40'
							}`}>
							<div
								className={`rounded-full p-6 ${activeZone === 'attachment' ? 'bg-blue-500 text-white' : 'bg-zinc-800 text-zinc-400'}`}>
								<UploadCloud size={64} />
							</div>
							<div className='text-center'>
								<h3
									className={`text-2xl font-bold ${activeZone === 'attachment' ? 'text-blue-200' : 'text-zinc-300'}`}>
									Add Attachment
								</h3>
								<p className='mt-2 text-lg text-zinc-500'>Drop files to attach</p>
							</div>
						</div>
					)}
				</div>
			)}
		</div>
	)
}
