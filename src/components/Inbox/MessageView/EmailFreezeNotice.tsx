import React from 'react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { AlertTriangle, X } from 'lucide-react'
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

	const handleResume = async () => {
		try {
			await invoke('resume_email_webview')
		} catch (e) {
			console.error('Failed to resume webview', e)
		}
	}

	return (
		<div className='relative flex items-center justify-between gap-4 border-b border-amber-200/50 bg-amber-50/80 px-4 py-3 dark:border-amber-900/30 dark:bg-status-warning/15'>
			<div className='flex items-center gap-3'>
				<div className='flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-amber-100 dark:bg-status-warning/15'>
					<AlertTriangle className='h-4 w-4 text-status-warning dark:text-status-warning' />
				</div>
				<div className='flex flex-col'>
					<span className='text-sm font-semibold text-amber-900 dark:text-status-warning'>
						{t('messageView.freezeNotice.title')}
					</span>
					<span className='text-xs text-amber-700/80 dark:text-status-warning'>
						{t('messageView.freezeNotice.description')} •{' '}
						{t('messageView.freezeNotice.memoryStats', { memory: memoryMB })}
					</span>
				</div>
			</div>

			<div className='flex items-center gap-2'>
				<Button
					variant='ghost'
					size='sm'
					onClick={onDismiss}
					className='h-8 text-xs text-amber-700 hover:bg-amber-100 hover:text-amber-900 dark:text-status-warning dark:hover:bg-status-warning/15 dark:hover:text-status-warning'>
					{t('messageView.freezeNotice.readAsStatic')}
				</Button>
				<Button
					size='sm'
					onClick={handleResume}
					className='h-8 border-0 bg-status-warning text-xs text-white hover:bg-amber-600 dark:bg-amber-600 dark:hover:bg-status-warning'>
					{t('messageView.freezeNotice.resume')}
				</Button>
				<button
					onClick={onDismiss}
					className='ml-2 flex h-8 w-8 items-center justify-center rounded-md text-status-warning hover:bg-amber-100 dark:hover:bg-status-warning/15'>
					<X className='h-4 w-4' />
				</button>
			</div>
		</div>
	)
}
