import { useEffect, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { Send, CheckCircle, AlertCircle, Loader2 } from 'lucide-react'
import { useOutboxStore, setupOutboxListeners } from '@/stores/outboxStore'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

interface StatusBarProps {
	onOpenOutbox: () => void
}

/**
 * Render a status bar showing sync state and outbox activity, including a button to open the outbox.
 *
 * Displays counts for pending, sending, and failed outbox items, updates iconography and label based
 * on current activity, and briefly shows a success indicator when a message finishes sending.
 *
 * @param onOpenOutbox - Callback invoked when the outbox button is clicked to open the outbox UI
 * @returns The React element for the status bar containing the sync indicator, outbox status button, and tooltip
 */
export function StatusBar({ onOpenOutbox }: StatusBarProps) {
	const { t } = useTranslation()
	const { items } = useOutboxStore()
	const [pendingCount, setPendingCount] = useState(0)
	const [sendingCount, setSendingCount] = useState(0)
	const [failedCount, setFailedCount] = useState(0)
	const [showSuccess, setShowSuccess] = useState(false)

	useEffect(() => {
		const pending = items.filter((i) => i.status === 'PENDING' || i.status === 'RETRY').length
		const sending = items.filter((i) => i.status === 'PROCESSING').length
		const failed = items.filter((i) => i.status === 'FAILED').length

		setPendingCount(pending)
		setSendingCount(sending)
		setFailedCount(failed)
	}, [items])

	useEffect(() => {
		let cleanupFn: (() => void) | undefined

		const setup = async () => {
			cleanupFn = await setupOutboxListeners(
				undefined, // processing
				() => {
					setShowSuccess(true)
					setTimeout(() => setShowSuccess(false), 3000)
				},
				undefined, // retry
				undefined // failed
			)
		}
		setup()

		return () => {
			cleanupFn?.()
		}
	}, [])

	const getStatusIcon = useCallback(() => {
		if (showSuccess) return <CheckCircle className='h-3 w-3 text-green-500' />
		if (sendingCount > 0) return <Loader2 className='h-3 w-3 animate-spin text-yellow-500' />
		if (failedCount > 0) return <AlertCircle className='h-3 w-3 text-red-500' />
		if (pendingCount > 0) return <Send className='h-3 w-3 text-blue-500' />
		return <CheckCircle className='h-3 w-3 text-zinc-500' />
	}, [sendingCount, failedCount, pendingCount, showSuccess])

	const getStatusText = useCallback(() => {
		if (showSuccess) return t('statusBar.sent', 'Message sent!')
		if (sendingCount > 0)
			return t('statusBar.sending', 'Sending {{count}}...', { count: sendingCount })
		if (failedCount > 0)
			return t('statusBar.failed', '{{count}} failed', { count: failedCount })
		if (pendingCount > 0)
			return t('statusBar.pending', '{{count}} pending', { count: pendingCount })
		return t('statusBar.ready', 'Ready')
	}, [sendingCount, failedCount, pendingCount, showSuccess, t])

	const hasActivity = pendingCount > 0 || sendingCount > 0 || failedCount > 0

	return (
		<TooltipProvider>
			<div className='flex h-6 shrink-0 items-center justify-between border-t border-zinc-800 bg-zinc-950 px-2 text-xs text-zinc-500'>
				{/* Left section - Sync status (placeholder) */}
				<div className='flex items-center gap-2'>
					<div className='flex items-center gap-1.5'>
						<div className='h-1.5 w-1.5 rounded-full bg-green-500' />
						<span className='text-zinc-600'>{t('statusBar.synced', 'Synced')}</span>
					</div>
				</div>

				{/* Right section - Outbox status */}
				<div className='flex items-center gap-3'>
					<Tooltip>
						<TooltipTrigger asChild>
							<Button
								variant='ghost'
								size='sm'
								className={`h-5 gap-1.5 px-2 text-xs ${
									hasActivity ? 'text-zinc-300 hover:text-white' : 'text-zinc-600'
								}`}
								onClick={onOpenOutbox}>
								{getStatusIcon()}
								<span>{getStatusText()}</span>
							</Button>
						</TooltipTrigger>
						<TooltipContent side='top'>
							<p>
								{hasActivity
									? t('statusBar.clickToView', 'Click to view outbox')
									: t('statusBar.noMessages', 'No messages in outbox')}
							</p>
						</TooltipContent>
					</Tooltip>
				</div>
			</div>
		</TooltipProvider>
	)
}