import React, { useState, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { AlertTriangle, X, GripHorizontal } from 'lucide-react'
import { Button } from '@/components/ui/button'

export interface FreezeStats {
	memory_bytes: number
}

interface EmailFreezeNoticeProps {
	stats: FreezeStats
	onDismiss: () => void
}

export const EmailFreezeNotice: React.FC<EmailFreezeNoticeProps> = ({ stats, onDismiss }) => {
	const { t } = useTranslation('inbox')
	const memoryMB = (stats.memory_bytes / 1024 / 1024).toFixed(1)
	const [position, setPosition] = useState({ x: 20, y: 20 })
	const [isDragging, setIsDragging] = useState(false)
	const dragStartInfo = useRef({ startX: 0, startY: 0, initialPosX: 0, initialPosY: 0 })
	const containerRef = useRef<HTMLDivElement>(null)

	const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
		if ((e.target as HTMLElement).closest('.drag-handle')) {
			setIsDragging(true)
			dragStartInfo.current = {
				startX: e.clientX,
				startY: e.clientY,
				initialPosX: position.x,
				initialPosY: position.y,
			}
			e.currentTarget.setPointerCapture(e.pointerId)
		}
	}

	const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
		if (!isDragging) return
		const dx = e.clientX - dragStartInfo.current.startX
		const dy = e.clientY - dragStartInfo.current.startY
		setPosition({
			x: dragStartInfo.current.initialPosX - dx, // Subtracted because x is right-aligned
			y: dragStartInfo.current.initialPosY + dy,
		})
	}

	const handlePointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
		setIsDragging(false)
		e.currentTarget.releasePointerCapture(e.pointerId)
	}

	const handleResume = async () => {
		try {
			await invoke('resume_email_webview')
		} catch (e) {
			console.error('Failed to resume webview', e)
		}
	}

	return (
		<div
			ref={containerRef}
			className='pointer-events-auto absolute z-50 flex w-80 flex-col rounded-lg border border-slate-200 bg-white shadow-xl dark:border-slate-700 dark:bg-slate-800'
			style={{
				top: `${position.y}px`,
				right: `${position.x}px`,
				touchAction: 'none',
			}}
			onPointerDown={handlePointerDown}
			onPointerMove={handlePointerMove}
			onPointerUp={handlePointerUp}
			onPointerCancel={handlePointerUp}>
			<div className='drag-handle flex cursor-grab items-center justify-between rounded-t-lg border-b border-slate-100 bg-slate-50 px-3 py-2 transition-colors hover:bg-slate-100 active:cursor-grabbing dark:border-slate-700/50 dark:bg-slate-800/80 dark:hover:bg-slate-700/50'>
				<div className='flex items-center gap-2'>
					<GripHorizontal className='h-4 w-4 text-slate-400' />
					<span className='text-sm font-semibold text-slate-700 select-none dark:text-slate-300'>
						{t('messageView.freezeNotice.title')}
					</span>
				</div>
				<button
					onClick={onDismiss}
					className='cursor-pointer text-slate-400 hover:text-slate-600 dark:hover:text-slate-200'>
					<X className='h-4 w-4' />
				</button>
			</div>

			<div className='p-4'>
				<div className='mb-3 flex gap-3'>
					<AlertTriangle className='mt-0.5 h-5 w-5 shrink-0 text-amber-500' />
					<p className='text-sm leading-relaxed text-slate-600 select-none dark:text-slate-300'>
						{t('messageView.freezeNotice.description')}
					</p>
				</div>

				<div className='mb-5 flex items-center justify-between rounded-md border border-slate-100 bg-slate-50 px-3 py-2 font-mono text-xs text-slate-600 select-none dark:border-slate-800 dark:bg-slate-900/50 dark:text-slate-400'>
					<span>{t('messageView.freezeNotice.memoryStats', { memory: memoryMB })}</span>
				</div>

				<div className='flex justify-end gap-2'>
					<Button variant='ghost' size='sm' onClick={onDismiss} className='text-xs'>
						{t('messageView.freezeNotice.readAsStatic')}
					</Button>
					<Button
						size='sm'
						onClick={handleResume}
						className='bg-amber-500 text-xs text-white hover:bg-amber-600'>
						{t('messageView.freezeNotice.resume')}
					</Button>
				</div>
			</div>
		</div>
	)
}
