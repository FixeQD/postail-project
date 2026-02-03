import { useEffect, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { X, RotateCcw, Ban, Send, AlertCircle, Clock, CheckCircle } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useOutboxStore, setupOutboxListeners } from '@/stores/outboxStore'
import type { OutboxItem } from '@/stores/outboxStore'

interface OutboxPanelProps {
	accountId: string
	isOpen: boolean
	onClose: () => void
}

const statusConfig: Record<
	OutboxItem['status'],
	{ icon: typeof Send; color: string; bgColor: string; label: string }
> = {
	PENDING: {
		icon: Clock,
		color: 'text-blue-500',
		bgColor: 'bg-blue-500/10',
		label: 'Pending',
	},
	PROCESSING: {
		icon: Send,
		color: 'text-yellow-500',
		bgColor: 'bg-yellow-500/10',
		label: 'Sending',
	},
	SENT: {
		icon: CheckCircle,
		color: 'text-green-500',
		bgColor: 'bg-green-500/10',
		label: 'Sent',
	},
	RETRY: {
		icon: RotateCcw,
		color: 'text-orange-500',
		bgColor: 'bg-orange-500/10',
		label: 'Retrying',
	},
	FAILED: {
		icon: AlertCircle,
		color: 'text-red-500',
		bgColor: 'bg-red-500/10',
		label: 'Failed',
	},
}

/**
 * Renders a modal outbox panel that shows active and recently sent messages for an account.
 *
 * @param accountId - Identifier of the account whose outbox will be loaded and displayed
 * @param isOpen - Whether the panel is visible
 * @param onClose - Callback invoked when the panel's close action is triggered
 * @returns The rendered outbox panel element, or `null` when `isOpen` is false
 */
export function OutboxPanel({ accountId, isOpen, onClose }: OutboxPanelProps) {
	const { t } = useTranslation()
	const { items, isLoading, loadOutbox, retryMessage, cancelMessage } = useOutboxStore()
	const [retryingId, setRetryingId] = useState<string | null>(null)
	const [cancellingId, setCancellingId] = useState<string | null>(null)

	useEffect(() => {
		if (isOpen && accountId) {
			loadOutbox(accountId)
		}
	}, [isOpen, accountId, loadOutbox])

	useEffect(() => {
		let cleanupFn: (() => void) | undefined

		const setup = async () => {
			cleanupFn = await setupOutboxListeners()
		}
		setup()

		return () => {
			cleanupFn?.()
		}
	}, [])

	const handleRetry = useCallback(
		async (outboxId: string) => {
			setRetryingId(outboxId)
			await retryMessage(outboxId)
			setRetryingId(null)
		},
		[retryMessage]
	)

	const handleCancel = useCallback(
		async (outboxId: string) => {
			setCancellingId(outboxId)
			await cancelMessage(outboxId)
			setCancellingId(null)
		},
		[cancelMessage]
	)

	const activeItems = items.filter((item) => item.status !== 'SENT')
	const sentItems = items.filter((item) => item.status === 'SENT')

	if (!isOpen) return null

	return (
		<div className='fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4'>
			<Card className='flex max-h-[80vh] w-full max-w-2xl flex-col overflow-hidden border-zinc-800 bg-zinc-950'>
				<CardHeader className='flex-shrink-0 border-b border-zinc-800 px-4 py-3'>
					<div className='flex items-center justify-between'>
						<CardTitle className='text-lg font-semibold text-zinc-200'>
							{t('outbox.title', 'Outbox')}
							{activeItems.length > 0 && (
								<Badge
									variant='outline'
									className='ml-2 border-blue-500/30 bg-blue-500/10 text-blue-500'>
									{activeItems.length}
								</Badge>
							)}
						</CardTitle>
						<Button
							variant='ghost'
							size='icon'
							className='h-8 w-8 text-zinc-500 hover:bg-zinc-800 hover:text-zinc-300'
							onClick={onClose}>
							<X className='h-4 w-4' />
						</Button>
					</div>
				</CardHeader>

				<CardContent className='flex-1 space-y-4 overflow-y-auto p-4'>
					{isLoading ? (
						<div className='flex h-32 items-center justify-center text-zinc-500'>
							{t('outbox.loading', 'Loading...')}
						</div>
					) : items.length === 0 ? (
						<div className='flex h-32 flex-col items-center justify-center text-center'>
							<CheckCircle className='mb-3 h-10 w-10 text-green-500' />
							<p className='text-zinc-400'>{t('outbox.empty', 'Outbox is empty')}</p>
							<p className='mt-1 text-sm text-zinc-600'>
								{t('outbox.emptyDescription', 'No emails waiting to be sent')}
							</p>
						</div>
					) : (
						<>
							{/* Active Messages */}
							{activeItems.length > 0 && (
								<div className='space-y-2'>
									<h3 className='text-sm font-medium text-zinc-400'>
										{t('outbox.activeMessages', 'Active Messages')}
									</h3>
									{activeItems.map((item) => (
										<OutboxItemCard
											key={item.id}
											item={item}
											onRetry={handleRetry}
											onCancel={handleCancel}
											isRetrying={retryingId === item.id}
											isCancelling={cancellingId === item.id}
										/>
									))}
								</div>
							)}

							{/* Recently Sent */}
							{sentItems.length > 0 && (
								<div className='space-y-2 border-t border-zinc-800 pt-4'>
									<h3 className='text-sm font-medium text-zinc-400'>
										{t('outbox.recentlySent', 'Recently Sent')}
									</h3>
									{sentItems.slice(0, 5).map((item) => (
										<OutboxItemCard
											key={item.id}
											item={item}
											onRetry={() => {}}
											onCancel={() => {}}
											isRetrying={false}
											isCancelling={false}
										/>
									))}
								</div>
							)}
						</>
					)}
				</CardContent>
			</Card>
		</div>
	)
}

interface OutboxItemCardProps {
	item: OutboxItem
	onRetry: (id: string) => void
	onCancel: (id: string) => void
	isRetrying: boolean
	isCancelling: boolean
}

/**
 * Render a card showing an outbox item's subject, recipient, status badge, optional error message, and action buttons for retrying or cancelling when applicable.
 *
 * @param item - The outbox item to display.
 * @param onRetry - Callback invoked with the item's `id` when the Retry button is pressed.
 * @param onCancel - Callback invoked with the item's `id` when the Cancel button is pressed.
 * @param isRetrying - Whether a retry operation is currently in progress for this item; disables action buttons and shows a spinner.
 * @param isCancelling - Whether a cancel operation is currently in progress for this item; disables action buttons and shows a pulsing icon.
 * @returns A JSX element representing the outbox item card.
 */
function OutboxItemCard({
	item,
	onRetry,
	onCancel,
	isRetrying,
	isCancelling,
}: OutboxItemCardProps) {
	const config = statusConfig[item.status]
	const Icon = config.icon

	return (
		<div className='space-y-2 rounded-lg border border-zinc-800 bg-zinc-900/50 p-3'>
			<div className='flex items-start justify-between gap-3'>
				<div className='flex min-w-0 flex-1 items-start gap-2'>
					<div className={`mt-0.5 ${config.color}`}>
						<Icon className='h-4 w-4' />
					</div>
					<div className='min-w-0 flex-1'>
						<p className='truncate text-sm font-medium text-zinc-200'>
							{item.subject || '(No subject)'}
						</p>
						<p className='truncate text-xs text-zinc-500'>
							{item.recipient || 'Unknown recipient'}
						</p>
					</div>
				</div>
				<Badge
					variant='outline'
					className={`flex-shrink-0 text-[10px] ${config.bgColor} ${config.color} border-current`}>
					{config.label}
				</Badge>
			</div>

			{item.status === 'RETRY' && item.attempts > 0 && (
				<p className='text-xs text-orange-400'>Attempt {item.attempts}/5</p>
			)}

			{item.lastError && item.status === 'FAILED' && (
				<Alert variant='destructive' className='py-2'>
					<AlertDescription className='text-xs'>{item.lastError}</AlertDescription>
				</Alert>
			)}

			{(item.status === 'FAILED' || item.status === 'RETRY') && (
				<div className='flex gap-2 pt-1'>
					<Button
						variant='outline'
						size='sm'
						className='h-7 border-orange-500/30 text-xs text-orange-400 hover:bg-orange-500/10'
						onClick={() => onRetry(item.id)}
						disabled={isRetrying || isCancelling}>
						{isRetrying ? (
							<RotateCcw className='h-3 w-3 animate-spin' />
						) : (
							<RotateCcw className='mr-1 h-3 w-3' />
						)}
						Retry
					</Button>
					<Button
						variant='outline'
						size='sm'
						className='h-7 border-red-500/30 text-xs text-red-400 hover:bg-red-500/10'
						onClick={() => onCancel(item.id)}
						disabled={isRetrying || isCancelling}>
						{isCancelling ? (
							<Ban className='h-3 w-3 animate-pulse' />
						) : (
							<Ban className='mr-1 h-3 w-3' />
						)}
						Cancel
					</Button>
				</div>
			)}
		</div>
	)
}