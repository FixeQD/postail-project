import { useState, useEffect, useRef } from 'react'
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

interface ComposeScreenProps {
	open: boolean
	onOpenChange: (open: boolean) => void
}

export function ComposeScreen({ open, onOpenChange }: ComposeScreenProps) {
	const { t } = useTranslation()
	const [to, setTo] = useState('')
	const [subject, setSubject] = useState('')
	const [isDragging, setIsDragging] = useState(false)
	const [dragStart, setDragStart] = useState({ x: 0, y: 0 })
	const [isResizing, setIsResizing] = useState(false)
	const [resizeStart, setResizeStart] = useState({ mouseX: 0, mouseY: 0, width: 0, height: 0 })
	const [position, setPosition] = useState({ x: 100, y: 100 })
	const [size, setSize] = useState({ width: 672, height: 600 })
	const modalRef = useRef<HTMLDivElement>(null)

	const handleMouseDown = (e: React.MouseEvent) => {
		e.preventDefault()
		setIsDragging(true)
		setDragStart({ x: e.clientX - position.x, y: e.clientY - position.y })
	}

	const handleResizeMouseDown = (e: React.MouseEvent) => {
		e.preventDefault()
		e.stopPropagation()
		setIsResizing(true)
		setResizeStart({
			mouseX: e.clientX,
			mouseY: e.clientY,
			width: size.width,
			height: size.height,
		})
	}

	const handleMouseMove = (e: MouseEvent) => {
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
			const newWidth = Math.max(
				400,
				Math.min(
					resizeStart.width + (e.clientX - resizeStart.mouseX),
					window.innerWidth - position.x
				)
			)
			const newHeight = Math.max(
				300,
				Math.min(
					resizeStart.height + (e.clientY - resizeStart.mouseY),
					window.innerHeight - position.y
				)
			)
			setSize({ width: newWidth, height: newHeight })
		}
	}

	const handleMouseUp = () => {
		setIsDragging(false)
		setIsResizing(false)
	}

	useEffect(() => {
		if (isDragging || isResizing) {
			document.addEventListener('mousemove', handleMouseMove)
			document.addEventListener('mouseup', handleMouseUp)
			return () => {
				document.removeEventListener('mousemove', handleMouseMove)
				document.removeEventListener('mouseup', handleMouseUp)
			}
		}
	}, [isDragging, isResizing])

	if (!open) return null

	return (
		<div
			ref={modalRef}
			className='fixed z-50 flex flex-col overflow-hidden rounded-xl bg-zinc-950 text-zinc-100 shadow-2xl ring-1 ring-zinc-800'
			style={{
				transform: `translate(${position.x}px, ${position.y}px)`,
				width: `${size.width}px`,
				height: `${size.height}px`,
				willChange: 'transform',
				userSelect: isDragging || isResizing ? 'none' : 'auto',
			}}>
			{/* Header */}
			<div
				className='flex items-center justify-between bg-zinc-900 px-4 py-3'
				onMouseDown={handleMouseDown}
				style={{ cursor: isDragging ? 'grabbing' : 'grab' }}>
				<h2 className='text-sm font-medium text-zinc-300'>{t('compose.newMessage')}</h2>
				<div className='flex items-center gap-1'>
					<Button
						variant='ghost'
						size='icon'
						className='h-6 w-6 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'>
						<Minimize2 className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						onClick={() => onOpenChange(false)}
						className='h-6 w-6 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'>
						<X className='h-4 w-4' />
					</Button>
				</div>
			</div>

			{/* Form Fields */}
			<div className='flex flex-col gap-1 px-4 pt-2'>
				<div className='relative'>
					<Input
						placeholder={t('compose.recipients')}
						value={to}
						onChange={(e) => setTo(e.target.value)}
						className='h-auto border-0 border-b border-zinc-800 bg-transparent px-0 py-3 text-sm placeholder:text-zinc-500 focus-visible:ring-0'
					/>
				</div>
				<div>
					<Input
						placeholder={t('compose.subject')}
						value={subject}
						onChange={(e) => setSubject(e.target.value)}
						className='h-auto border-0 border-b border-zinc-800 bg-transparent px-0 py-3 text-sm font-medium placeholder:text-zinc-500 focus-visible:ring-0'
					/>
				</div>
			</div>

			{/* Editor Area */}
			<div className='flex-1 overflow-y-auto p-4'>
				<div
					className='h-full min-h-[200px] w-full resize-none border-0 bg-transparent text-sm text-zinc-200 outline-none'
					contentEditable
					suppressContentEditableWarning
					data-placeholder={t('compose.writeSomething')}
				/>
			</div>

			{/* Footer / Toolbar */}
			<div className='mt-auto flex flex-col gap-2 border-t border-zinc-800 p-3'>
				{/* Formatting Icons Row */}
				<div className='flex items-center gap-1 overflow-x-auto pb-2 sm:pb-0'>
					<Button
						variant='ghost'
						size='icon'
						className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
						<Bold className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
						<Italic className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
						<Underline className='h-4 w-4' />
					</Button>
					<div className='mx-1 h-4 w-[1px] bg-zinc-800' />
					<Button
						variant='ghost'
						size='icon'
						className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
						<AlignLeft className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
						<List className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
						<ListOrdered className='h-4 w-4' />
					</Button>
				</div>

				{/* Action Row */}
				<div className='flex items-center justify-between pt-1'>
					<div className='flex items-center gap-2'>
						<Button className='rounded-full bg-blue-600 px-6 font-medium text-white hover:bg-blue-700'>
							{t('actions.send')}
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-9 w-9 text-zinc-400 hover:bg-zinc-800'>
							<Paperclip className='h-5 w-5' />
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-9 w-9 text-zinc-400 hover:bg-zinc-800'>
							<Link className='h-5 w-5' />
						</Button>
					</div>

					<div className='flex items-center gap-1'>
						<Button
							variant='ghost'
							size='icon'
							className='h-9 w-9 text-zinc-400 hover:bg-zinc-800 hover:text-red-400'>
							<Trash2 className='h-4 w-4' />
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-9 w-9 text-zinc-400 hover:bg-zinc-800'>
							<MoreVertical className='h-4 w-4' />
						</Button>
					</div>
				</div>
			</div>
			{/* Resize Handle */}
			<div
				className='absolute right-0 bottom-0 h-4 w-4 cursor-se-resize'
				onMouseDown={handleResizeMouseDown}
				style={{ background: 'transparent' }}
			/>
		</div>
	)
}
