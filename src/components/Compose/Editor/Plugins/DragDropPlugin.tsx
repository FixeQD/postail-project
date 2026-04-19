import React, { useEffect, useState, useCallback, useRef } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useDraftStore } from '@/stores/draftStore'
import { invoke } from '@tauri-apps/api/core'
import { useAsyncState } from '@/hooks/useAsyncState'
import type { EmailAttachment } from '@/types/compose'
import { Image as ImageIcon, Paperclip } from 'lucide-react'

export default function DragDropPlugin(): React.ReactNode {
	const addAttachment = useDraftStore((s) => s.addAttachment)
	const [isDragging, setIsDragging] = useState(false)
	const { isLoading: isProcessing, run: runProcessing } = useAsyncState()
	const [activeZone, setActiveZone] = useState<'inline' | 'attachment' | null>(null)

	const dragCounter = useRef(0)
	const zonesRef = useRef<{ inline: DOMRect | null; attachment: DOMRect | null }>({
		inline: null,
		attachment: null,
	})
	const isProcessingRef = useRef(false)
	// Track current zone in a ref so the Tauri drop handler can read it without stale closure
	const activeZoneRef = useRef<'inline' | 'attachment' | null>(null)

	const insertInlineImage = useCallback((attachment: EmailAttachment) => {
		window.dispatchEvent(new CustomEvent('compose:insert-inline-image', { detail: attachment }))
	}, [])

	const handlePaths = useCallback(
		async (paths: string[], mode: 'inline' | 'attachment') => {
			if (isProcessingRef.current) return
			isProcessingRef.current = true

			try {
				await runProcessing(async () => {
					for (const path of paths) {
						if (mode === 'inline') {
							const attachment = await invoke<EmailAttachment>(
								'add_inline_attachment_path',
								{ path }
							)
							addAttachment(attachment)
							if (attachment.contentType?.startsWith('image/') || isImagePath(path)) {
								insertInlineImage(attachment)
							}
						} else {
							const attachment = await invoke<EmailAttachment>('add_attachment', { path })
							addAttachment(attachment)
						}
					}
				})
			} catch (err) {
				console.error('[DragDrop] Processing failure:', err)
			} finally {
				isProcessingRef.current = false
			}
		},
		[addAttachment, insertInlineImage, runProcessing]
	)

	const handlePathsRef = useRef(handlePaths)
	useEffect(() => {
		handlePathsRef.current = handlePaths
	}, [handlePaths])

	useEffect(() => {
		const handleDragEnter = (e: DragEvent) => {
			e.preventDefault()
			e.stopPropagation()
			dragCounter.current++
			if (dragCounter.current === 1) setIsDragging(true)
		}

		const handleDragOver = (e: DragEvent) => {
			e.preventDefault()
			e.stopPropagation()

			const inlineRect = zonesRef.current.inline
			const attachRect = zonesRef.current.attachment
			let zone: 'inline' | 'attachment' | null = null

			if (inlineRect && e.clientX >= inlineRect.left && e.clientX <= inlineRect.right) {
				zone = 'inline'
			} else if (
				attachRect &&
				e.clientX >= attachRect.left &&
				e.clientX <= attachRect.right
			) {
				zone = 'attachment'
			}

			activeZoneRef.current = zone
			setActiveZone(zone)
		}

		const handleDragLeave = (e: DragEvent) => {
			e.preventDefault()
			e.stopPropagation()
			dragCounter.current--
			if (dragCounter.current <= 0) {
				dragCounter.current = 0
				setIsDragging(false)
				setActiveZone(null)
				activeZoneRef.current = null
			}
		}

		window.addEventListener('dragenter', handleDragEnter, true)
		window.addEventListener('dragover', handleDragOver, true)
		window.addEventListener('dragleave', handleDragLeave, true)

		return () => {
			window.removeEventListener('dragenter', handleDragEnter, true)
			window.removeEventListener('dragover', handleDragOver, true)
			window.removeEventListener('dragleave', handleDragLeave, true)
		}
	}, [])

	useEffect(() => {
		let unlisten: (() => void) | undefined
		let isMounted = true

		getCurrentWindow()
			.onDragDropEvent((event) => {
				if (!isMounted || event.payload.type !== 'drop') return

				const { paths, position } = event.payload
				if (!paths.length) return

				const dpr = window.devicePixelRatio || 1
				const logicalX = position.x / dpr

				let mode: 'inline' | 'attachment' = activeZoneRef.current ?? 'attachment'
				const inlineRect = zonesRef.current.inline
				if (!activeZoneRef.current && inlineRect) {
					mode =
						logicalX >= inlineRect.left && logicalX <= inlineRect.right
							? 'inline'
							: 'attachment'
				}

				setIsDragging(false)
				setActiveZone(null)
				activeZoneRef.current = null
				dragCounter.current = 0

				handlePathsRef.current(paths, mode)
			})
			.then((fn) => {
				if (isMounted) {
					unlisten = fn
				} else {
					fn()
				}
			})

		return () => {
			isMounted = false
			unlisten?.()
		}
	}, [])

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
				</div>
			)}
		</div>
	)
}

function isImagePath(path: string): boolean {
	return /\.(png|jpe?g|gif|webp|bmp|svg|avif)$/i.test(path)
}
