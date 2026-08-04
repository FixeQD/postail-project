import { useState, useRef, useEffect } from 'react'
import { Link as LinkIcon, Pencil, Check, X, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { motion } from 'framer-motion'

interface LinkInsertPopoverProps {
	onInsertLink: (url: string) => void
}

export function LinkInsertPopover({ onInsertLink }: LinkInsertPopoverProps) {
	const [open, setOpen] = useState(false)
	const [url, setUrl] = useState('')
	const inputRef = useRef<HTMLInputElement>(null)
	const savedRangeRef = useRef<Range | null>(null)

	const saveSelection = () => {
		const sel = window.getSelection()
		if (sel && sel.rangeCount > 0) {
			savedRangeRef.current = sel.getRangeAt(0).cloneRange()
		}
	}

	const restoreSelection = () => {
		const sel = window.getSelection()
		if (sel && savedRangeRef.current) {
			sel.removeAllRanges()
			sel.addRange(savedRangeRef.current)
		}
	}

	const handleOpen = (isOpen: boolean) => {
		try {
			if (isOpen) {
				saveSelection()
			} else {
				setUrl('')
			}
			setOpen(isOpen)
		} catch (e) {
			console.error('Failed to open link popover:', e)
			setOpen(isOpen)
		}
	}

	const handleSubmit = () => {
		if (!url.trim()) return
		restoreSelection()

		// Small delay so selection is restored before execCommand fires
		requestAnimationFrame(() => {
			onInsertLink(url.trim())
			setUrl('')
			setOpen(false)
		})
	}

	useEffect(() => {
		if (open) {
			setTimeout(() => inputRef.current?.focus(), 50)
		}
	}, [open])

	// Listen for Ctrl+K shortcut to toggle
	useEffect(() => {
		const handler = () => handleOpen(true)
		window.addEventListener('compose:insert-link', handler)
		return () => window.removeEventListener('compose:insert-link', handler)
	}, [])

	return (
		<Popover open={open} onOpenChange={handleOpen}>
			<PopoverTrigger asChild>
				<Button
					variant='ghost'
					size='icon'
					className='h-9 w-9 text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'>
					<LinkIcon className='h-4 w-4' />
				</Button>
			</PopoverTrigger>
			<PopoverContent align='center' sideOffset={8} className='w-72 p-2'>
				<div className='flex items-center gap-1.5'>
					<input
						ref={inputRef}
						type='url'
						placeholder='https://...'
						value={url}
						onChange={(e) => setUrl(e.target.value)}
						onKeyDown={(e) => {
							if (e.key === 'Enter') {
								e.preventDefault()
								handleSubmit()
							}
							if (e.key === 'Escape') {
								e.preventDefault()
								setOpen(false)
							}
						}}
						className='border-input placeholder:text-muted-foreground focus:ring-ring h-8 flex-1 rounded-md border bg-transparent px-2 text-sm outline-none focus:ring-1'
					/>
					<Button
						variant='ghost'
						size='icon'
						className='h-8 w-8 text-status-success hover:bg-status-success/15'
						onClick={handleSubmit}
						disabled={!url.trim()}>
						<Check className='h-4 w-4' />
					</Button>
				</div>
			</PopoverContent>
		</Popover>
	)
}

interface LinkEditTooltipProps {
	visible: boolean
	url: string
	rect: DOMRect | null
	onEdit: (url: string) => void
	onRemove: () => void
}

export function LinkEditTooltip({ visible, url, rect, onEdit, onRemove }: LinkEditTooltipProps) {
	const [isEditing, setIsEditing] = useState(false)
	const [editUrl, setEditUrl] = useState(url)
	const inputRef = useRef<HTMLInputElement>(null)

	useEffect(() => {
		setEditUrl(url)
		setIsEditing(false)
	}, [url, visible])

	useEffect(() => {
		if (isEditing) {
			setTimeout(() => inputRef.current?.focus(), 50)
		}
	}, [isEditing])

	if (!visible || !rect) return null

	const handleSave = () => {
		if (editUrl.trim()) {
			onEdit(editUrl.trim())
			setIsEditing(false)
		}
	}

	return (
		<motion.div
			initial={{ opacity: 0, scale: 0.95, y: rect.top > 40 ? 5 : -5 }}
			animate={{ opacity: 1, scale: 1, y: 0 }}
			exit={{ opacity: 0, scale: 0.95, y: rect.top > 40 ? 5 : -5 }}
			transition={{ duration: 0.15, ease: 'easeOut' }}
			className='link-edit-tooltip bg-popover text-popover-foreground ring-border fixed z-50 flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs shadow-lg ring-1'
			style={{
				left: `${rect.left + rect.width / 2}px`,
				top: `${rect.top > 40 ? rect.top - 8 : rect.bottom + 8}px`,
				transform: rect.top > 40 ? 'translate(-50%, -100%)' : 'translate(-50%, 0)',
			}}>
			{isEditing ? (
				<>
					<input
						ref={inputRef}
						type='url'
						value={editUrl}
						onChange={(e) => setEditUrl(e.target.value)}
						onKeyDown={(e) => {
							if (e.key === 'Enter') {
								e.preventDefault()
								handleSave()
							}
							if (e.key === 'Escape') {
								e.preventDefault()
								setIsEditing(false)
								setEditUrl(url)
							}
						}}
						className='border-input focus:ring-ring h-6 w-48 rounded-sm border bg-transparent px-1.5 outline-none focus:ring-1'
					/>
					<button
						className='rounded p-0.5 text-status-success transition-colors hover:bg-status-success/15'
						onClick={handleSave}>
						<Check className='h-3 w-3' />
					</button>
					<button
						className='text-muted-foreground hover:bg-accent rounded p-0.5 transition-colors'
						onClick={() => {
							setIsEditing(false)
							setEditUrl(url)
						}}>
						<X className='h-3 w-3' />
					</button>
				</>
			) : (
				<>
					<a
						href={url}
						target='_blank'
						rel='noopener noreferrer'
						className='max-w-48 truncate text-status-info underline'
						onClick={(e) => e.stopPropagation()}>
						{url.length > 60 ? url.slice(0, 56) + '…' : url}
					</a>
					<div className='bg-border mx-0.5 h-3 w-px' />
					<button
						className='hover:bg-accent rounded p-0.5 transition-colors'
						onClick={() => setIsEditing(true)}
						title='Edit link'>
						<Pencil className='h-3 w-3' />
					</button>
					<button
						className='rounded p-0.5 text-destructive transition-colors hover:bg-red-400/10'
						onClick={onRemove}
						title='Remove link'>
						<Trash2 className='h-3 w-3' />
					</button>
				</>
			)}
		</motion.div>
	)
}
