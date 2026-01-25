import { useState, useEffect, useRef, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import {
	X,
	Paperclip,
	Bold,
	Italic,
	Underline,
	Link,
	List,
	ListOrdered,
	AlignLeft,
	MoreVertical,
	Trash2,
	Minimize2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useDraftStore } from '@/stores/draftStore'

interface ComposeScreenProps {
	open: boolean
	onOpenChange: (open: boolean) => void
}

export function ComposeScreen({ open, onOpenChange }: ComposeScreenProps) {
	const { t } = useTranslation()
	const {
		currentDraft,
		isComposing,
		isDirty,
		isSaving,
		setSubject,
		setBody,
		updateCurrentDraft,
		startComposing,
		stopComposing,
		saveDraft,
	} = useDraftStore()

	const [isDragging, setIsDragging] = useState(false)
	const [dragStart, setDragStart] = useState({ x: 0, y: 0 })
	const [isResizing, setIsResizing] = useState(false)
	const [resizeStart, setResizeStart] = useState({ mouseX: 0, mouseY: 0, width: 0, height: 0 })

	const [position, setPosition] = useState({
		x: window.innerWidth - 720,
		y: window.innerHeight - 650,
	})
	const [size, setSize] = useState({ width: 672, height: 600 })

	const editorRef = useRef<HTMLDivElement>(null)

	const handleResizeMouseDown = (e: React.MouseEvent) => {
		e.preventDefault()
		setIsResizing(true)
		setResizeStart({
			mouseX: e.clientX,
			mouseY: e.clientY,
			width: size.width,
			height: size.height,
		})
	}

	const handleMouseMove = useCallback(
		(e: MouseEvent) => {
			if (isDragging) {
				const newX = Math.max(
					0,
					Math.min(e.clientX - dragStart.x, window.innerWidth - size.width)
				)
				const newY = Math.max(
					0,
					Math.min(e.clientY - dragStart.y, window.innerHeight - size.height)
				)
				setPosition({ x: newX, y: newY })
			}
			if (isResizing) {
				const newWidth = Math.max(450, resizeStart.width + (e.clientX - resizeStart.mouseX))
				const newHeight = Math.max(
					400,
					resizeStart.height + (e.clientY - resizeStart.mouseY)
				)
				setSize({ width: newWidth, height: newHeight })
			}
		},
		[isDragging, isResizing, dragStart, resizeStart, size.width, size.height]
	)

	const handleMouseUp = useCallback(() => {
		setIsDragging(false)
		setIsResizing(false)
	}, [])

	useEffect(() => {
		if (isDragging || isResizing) {
			window.addEventListener('mousemove', handleMouseMove)
			window.addEventListener('mouseup', handleMouseUp)
		}
		return () => {
			window.removeEventListener('mousemove', handleMouseMove)
			window.removeEventListener('mouseup', handleMouseUp)
		}
	}, [isDragging, isResizing, handleMouseMove, handleMouseUp])

	useEffect(() => {
		if (open && !isComposing) {
			startComposing('default-account')
		}
	}, [open, isComposing, startComposing])

	// Auto-save
	useEffect(() => {
		if (!isDirty || !currentDraft) return
		const timer = setTimeout(() => saveDraft(), 3000)
		return () => clearTimeout(timer)
	}, [isDirty, currentDraft?.subject, currentDraft?.body, saveDraft])

	// Set initial content without resetting cursor
	useEffect(() => {
		if (editorRef.current && currentDraft?.body && editorRef.current.innerHTML === '') {
			editorRef.current.innerHTML = currentDraft.body
		}
	}, [currentDraft?.body])

	if (!open) return null

	return (
		<div
			className={`fixed z-50 flex flex-col overflow-hidden rounded-t-xl bg-zinc-950 text-zinc-100 shadow-2xl ring-1 ring-zinc-800 transition-shadow ${isDragging ? 'shadow-blue-900/20' : ''}`}
			style={{
				left: `${position.x}px`,
				top: `${position.y}px`,
				width: `${size.width}px`,
				height: `${size.height}px`,
				userSelect: isDragging || isResizing ? 'none' : 'auto',
				cursor: isDragging ? 'grabbing' : 'auto',
			}}>
			{/* Header */}
			<div
				className='flex w-full items-center justify-between bg-zinc-900 px-4 py-3 select-none'
				onMouseDown={(e) => {
					e.preventDefault()
					setIsDragging(true)
					setDragStart({ x: e.clientX - position.x, y: e.clientY - position.y })
				}}
				style={{ cursor: isDragging ? 'grabbing' : 'grab' }}>
				<h2 className='text-sm font-medium text-zinc-300'>{t('compose.newMessage')}</h2>
				<div className='flex items-center gap-1'>
					<Button
						variant='ghost'
						size='icon'
						className='h-7 w-7 text-zinc-400 hover:bg-zinc-800'>
						<Minimize2 className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className='h-7 w-7 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'
						onClick={() => {
							saveDraft()
							onOpenChange(false)
							stopComposing()
						}}>
						<X className='h-4 w-4' />
					</Button>
				</div>
			</div>

			{/* Form Fields */}
			<div className='flex flex-col px-4 pt-1'>
				<Input
					placeholder={t('compose.recipients')}
					value={currentDraft?.to.map((r) => r.email).join(', ') || ''}
					onChange={(e) => {
						const emails = e.target.value
							.split(',')
							.map((s) => s.trim())
							.filter(Boolean)
						updateCurrentDraft({ to: emails.map((email) => ({ email })) })
					}}
					className='h-11 rounded-none border-0 border-b border-zinc-900 bg-transparent px-0 text-sm placeholder:text-zinc-600 focus-visible:ring-0'
				/>
				<Input
					placeholder={t('compose.subject')}
					value={currentDraft?.subject || ''}
					onChange={(e) => setSubject(e.target.value)}
					className='h-11 rounded-none border-0 border-b border-zinc-900 bg-transparent px-0 text-sm font-medium placeholder:text-zinc-600 focus-visible:ring-0'
				/>
			</div>

			{/* Editor Area */}
			<div className='custom-scrollbar relative flex-1 overflow-y-auto p-4'>
				<div
					ref={editorRef}
					className='h-full min-h-[200px] w-full text-sm text-zinc-200 outline-none focus:outline-none'
					contentEditable
					suppressContentEditableWarning
					onInput={(e) => setBody(e.currentTarget.innerHTML)}
				/>
				{!currentDraft?.body && (
					<div className='pointer-events-none absolute top-4 left-4 text-sm text-zinc-600'>
						{t('compose.writeSomething')}
					</div>
				)}
			</div>

			{/* Footer */}
			<div className='mt-auto border-t border-zinc-900 bg-zinc-950/50 p-3'>
				<div className='flex items-center justify-between'>
					<div className='flex items-center gap-1'>
						<Button
							onClick={() => console.log('Sending...', currentDraft)}
							className='h-9 rounded-full bg-blue-600 px-6 font-semibold text-white hover:bg-blue-500'
							disabled={isSaving}>
							{isSaving ? '...' : t('actions.send')}
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-9 w-9 text-zinc-400 hover:bg-zinc-900'>
							<Paperclip className='h-5 w-5' />
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-9 w-9 text-zinc-400 hover:bg-zinc-900'>
							<Bold className='h-4 w-4' />
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-9 w-9 text-zinc-400 hover:bg-zinc-900'>
							<Italic className='h-4 w-4' />
						</Button>
					</div>

					<div className='flex items-center gap-1'>
						<Button
							variant='ghost'
							size='icon'
							className='h-9 w-9 text-zinc-400 hover:bg-zinc-900 hover:text-red-400'>
							<Trash2 className='h-4 w-4' />
						</Button>
					</div>
				</div>
			</div>

			{/* Resize Handle */}
			<div
				className='absolute right-0 bottom-0 h-4 w-4 cursor-se-resize'
				onMouseDown={handleResizeMouseDown}
			/>
		</div>
	)
}
