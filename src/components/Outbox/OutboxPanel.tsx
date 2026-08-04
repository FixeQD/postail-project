import { useEffect, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { X, RotateCcw, Ban, Send, AlertCircle, Clock, CheckCircle } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useOutboxStore, setupOutboxListeners } from '@/stores/outboxStore'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useToastStore } from '@/stores/toastStore'
import type { OutboxItem } from '@/stores/outboxStore'
import type { OutboxPanelProps } from '@/types/components/shared'

const statusConfig: Record<
	OutboxItem['status'],
	{ icon: typeof Send; color: string; bgColor: string; label: string }
> = {
	PENDING: {
		icon: Clock,
		color: 'text-status-info',
		bgColor: 'bg-status-info/15',
		label: 'Pending',
	},
	PROCESSING: {
		icon: Send,
		color: 'text-status-warning',
		bgColor: 'bg-status-warning/15',
		label: 'Sending',
	},
	SENT: {
		icon: CheckCircle,
		color: 'text-status-success',
		bgColor: 'bg-status-success/15',
		label: 'Sent',
	},
	RETRY: {
		icon: RotateCcw,
		color: 'text-accent-dynamic',
		bgColor: 'bg-accent-dynamic/10',
		label: 'Retrying',
	},
	FAILED: {
		icon: AlertCircle,
		color: 'text-destructive',
		bgColor: 'bg-destructive/15',
		label: 'Failed',
	},
}

export function OutboxPanel({ accountId, isOpen, onClose }: OutboxPanelProps) {
	const { t } = useTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const { items, isLoading, loadOutbox, retryMessage, cancelMessage } = useOutboxStore()
	const addToast = useToastStore((s) => s.addToast)
	const [retryingId, setRetryingId] = useState<string | null>(null)
	const [cancellingId, setCancellingId] = useState<string | null>(null)

	useEffect(() => {
		if (isOpen && accountId) {
			loadOutbox(accountId)
		}
	}, [isOpen, accountId, loadOutbox])

	useEffect(() => {
		let cleanupFn: (() => void) | undefined
		let cancelled = false

		const setup = async () => {
			const cleanup = await setupOutboxListeners()
			if (!cancelled) {
				cleanupFn = cleanup
			} else {
				cleanup()
			}
		}
		setup()

		return () => {
			cancelled = true
			cleanupFn?.()
		}
	}, [])

	const handleRetry = useCallback(
		async (outboxId: string) => {
			setRetryingId(outboxId)
			const result = await retryMessage(outboxId)
			setRetryingId(null)
			if (!result.ok) {
				addToast(`Retry failed: ${result.error}`, 'error')
			}
		},
		[retryMessage, addToast]
	)

	const handleCancel = useCallback(
		async (outboxId: string) => {
			setCancellingId(outboxId)
			const result = await cancelMessage(outboxId)
			setCancellingId(null)
			if (!result.ok) {
				addToast(`Cancel failed: ${result.error}`, 'error')
			}
		},
		[cancelMessage, addToast]
	)

	const activeItems = items.filter((item) => item.status !== 'SENT')
	const sentItems = items.filter((item) => item.status === 'SENT')

	if (!isOpen) return null

	return (
		<AnimatePresence>
			{isOpen && (
				<motion.div
					{...(animationsEnabled
						? {
								initial: { opacity: 0 },
								animate: { opacity: 1 },
								exit: { opacity: 0 },
								transition: { duration: 0.2 },
							}
						: {})}
					className='fixed inset-0 z-50 flex items-center justify-center p-4'
					onClick={onClose}>
					<motion.div
						{...(animationsEnabled
							? {
									initial: { opacity: 0 },
									animate: { opacity: 1 },
									exit: { opacity: 0 },
								}
							: {})}
						className='absolute inset-0 bg-black/60 backdrop-blur-sm'
					/>

					<motion.div
						{...(animationsEnabled
							? {
									initial: { opacity: 0, y: 24, scale: 0.96 },
									animate: { opacity: 1, y: 0, scale: 1 },
									exit: { opacity: 0, y: 16, scale: 0.97 },
									transition: { duration: 0.3, ease: [0.16, 1, 0.3, 1] },
								}
							: {})}
						onClick={(e) => e.stopPropagation()}>
						<Card className='flex max-h-[80vh] w-full max-w-4xl min-w-[640px] flex-col overflow-hidden border-[var(--border-subtle)] bg-[var(--surface-glass)] shadow-2xl ring-1 ring-[var(--border-subtle)] backdrop-blur-xl'>
							<CardHeader className='flex-shrink-0 border-b border-[var(--border-faint)] px-5 py-4'>
								<div className='flex items-center justify-between'>
									<div className='flex items-center gap-3'>
										<div
											className='flex h-8 w-8 items-center justify-center rounded-lg ring-1'
											style={{
												backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
												boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
											}}>
											<Send
												className='h-4 w-4'
												style={{ color: accentColor }}
											/>
										</div>
										<CardTitle className='text-lg font-semibold text-[var(--text-primary)]'>
											{t('outbox.title')}
											{activeItems.length > 0 && (
												<Badge
													variant='outline'
													className='ml-2 border-status-info/30 bg-status-info/15 text-xs text-status-info'>
													{activeItems.length}
												</Badge>
											)}
										</CardTitle>
									</div>
									<motion.div
										{...(animationsEnabled
											? {
													whileHover: { scale: 1.1 },
													whileTap: { scale: 0.9 },
												}
											: {})}>
										<Button
											variant='ghost'
											size='icon'
											className='h-8 w-8 text-[var(--text-secondary)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
											onClick={onClose}>
											<X className='h-4 w-4' />
										</Button>
									</motion.div>
								</div>
							</CardHeader>

							<CardContent className='hover-scrollbar flex-1 space-y-4 overflow-y-auto p-5'>
								{isLoading ? (
									<div className='flex h-32 items-center justify-center'>
										<div className='flex flex-col items-center gap-3'>
											<div className='relative h-8 w-8'>
												<div
													className='absolute inset-0 animate-spin rounded-full border-2 border-transparent'
													style={{ borderTopColor: accentColor }}
												/>
												<div
													className='absolute inset-1 animate-spin rounded-full border-2 border-transparent'
													style={{
														borderBottomColor: `rgba(var(--accent-rgb), 0.3)`,
														animationDirection: 'reverse',
														animationDuration: '1.5s',
													}}
												/>
											</div>
											<span className='text-sm text-[var(--text-tertiary)]'>
												{t('outbox.loading')}
											</span>
										</div>
									</div>
								) : items.length === 0 ? (
									<motion.div
										{...(animationsEnabled
											? {
													initial: { opacity: 0, scale: 0.95 },
													animate: { opacity: 1, scale: 1 },
													transition: { delay: 0.1, duration: 0.3 },
												}
											: {})}
										className='flex h-32 flex-col items-center justify-center text-center'>
										<div className='mb-3 flex h-14 w-14 items-center justify-center rounded-2xl bg-status-success/[0.08] ring-1 ring-status-success/30'>
											<CheckCircle className='h-6 w-6 text-status-success' />
										</div>
										<p className='text-sm font-medium text-[var(--text-secondary)]'>
											{t('outbox.empty')}
										</p>
										<p className='mt-1 text-xs text-[var(--text-tertiary)]'>
											{t(
												'outbox.emptyDescription',
												'No emails waiting to be sent'
											)}
										</p>
									</motion.div>
								) : (
									<div className='stagger-children'>
										{activeItems.length > 0 && (
											<div className='space-y-2'>
												<h3 className='ml-1 text-xs font-semibold tracking-wider text-[var(--text-tertiary)] uppercase'>
													{t('outbox.activeMessages')}
												</h3>
												{activeItems.map((item, index) => (
													<motion.div
														key={item.id}
														{...(animationsEnabled
															? {
																	initial: { opacity: 0, y: 12 },
																	animate: { opacity: 1, y: 0 },
																	transition: {
																		delay: index * 0.05,
																		duration: 0.3,
																	},
																}
															: {})}>
														<OutboxItemCard
															item={item}
															onRetry={handleRetry}
															onCancel={handleCancel}
															isRetrying={retryingId === item.id}
															isCancelling={cancellingId === item.id}
														/>
													</motion.div>
												))}
											</div>
										)}

										{sentItems.length > 0 && (
											<div className='space-y-2 border-t border-[var(--border-faint)] pt-4'>
												<h3 className='ml-1 text-xs font-semibold tracking-wider text-[var(--text-tertiary)] uppercase'>
													{t('outbox.recentlySent')}
												</h3>
												{sentItems.slice(0, 5).map((item, index) => (
													<motion.div
														key={item.id}
														{...(animationsEnabled
															? {
																	initial: { opacity: 0, y: 8 },
																	animate: { opacity: 1, y: 0 },
																	transition: {
																		delay: 0.1 + index * 0.04,
																		duration: 0.3,
																	},
																}
															: {})}>
														<OutboxItemCard
															item={item}
															onRetry={() => {}}
															onCancel={() => {}}
															isRetrying={false}
															isCancelling={false}
														/>
													</motion.div>
												))}
											</div>
										)}
									</div>
								)}
							</CardContent>
						</Card>
					</motion.div>
				</motion.div>
			)}
		</AnimatePresence>
	)
}

interface OutboxItemCardProps {
	item: OutboxItem
	onRetry: (id: string) => void
	onCancel: (id: string) => void
	isRetrying: boolean
	isCancelling: boolean
}

function OutboxItemCard({
	item,
	onRetry,
	onCancel,
	isRetrying,
	isCancelling,
}: OutboxItemCardProps) {
	const accentColor = useThemeStore((s) => s.accentColor)
	const config = statusConfig[item.status]
	const Icon = config.icon
	const isAccentStatus = item.status === 'RETRY'

	return (
		<div className='group space-y-2 rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-3.5 transition-colors hover:bg-[var(--surface-hover)]'>
			<div className='flex items-start justify-between gap-3'>
				<div className='flex min-w-0 flex-1 items-start gap-2.5'>
					<div
						className={`mt-0.5 ${isAccentStatus ? '' : config.color}`}
						style={isAccentStatus ? { color: accentColor } : undefined}>
						<Icon className='h-4 w-4' />
					</div>
					<div className='min-w-0 flex-1'>
						<p className='truncate text-sm font-medium text-[var(--text-primary)]'>
							{item.subject || '(No subject)'}
						</p>
						<p className='truncate text-xs text-[var(--text-tertiary)]'>
							{item.recipient || 'Unknown recipient'}
						</p>
					</div>
				</div>
				<Badge
					variant='outline'
					className={`flex-shrink-0 text-[10px] font-semibold ${config.bgColor} ${config.color} border-current/20`}>
					{config.label}
				</Badge>
			</div>

			{/* Retry attempt counter + countdown */}
			{item.status === 'RETRY' && (
				<div className='flex items-center justify-between'>
					<p className='text-xs' style={{ color: `rgba(var(--accent-rgb), 0.8)` }}>
						Attempt {item.attempts}/5
					</p>
					{item.nextRetry != null && (
						<RetryCountdown
							nextRetryTimestamp={item.nextRetry}
							accentColor={accentColor}
						/>
					)}
				</div>
			)}

			{item.lastError && item.status === 'FAILED' && (
				<Alert variant='destructive' className='border-destructive/30 bg-destructive/15 py-2'>
					<AlertDescription className='text-xs text-destructive'>
						{item.lastError}
					</AlertDescription>
				</Alert>
			)}

			{(item.status === 'FAILED' || item.status === 'RETRY') && (
				<div className='flex gap-2 pt-1'>
					<Button
						variant='outline'
						size='sm'
						className='h-7 text-xs'
						style={{
							borderColor: `rgba(var(--accent-rgb), 0.2)`,
							backgroundColor: `rgba(var(--accent-rgb), 0.05)`,
							color: `rgba(var(--accent-rgb), 0.85)`,
						}}
						onClick={() => onRetry(item.id)}
						disabled={isRetrying || isCancelling}>
						{isRetrying ? (
							<RotateCcw className='h-3 w-3 animate-spin' />
						) : (
							<RotateCcw className='mr-1 h-3 w-3' />
						)}
						{!isRetrying && 'Retry now'}
					</Button>
					<Button
						variant='outline'
						size='sm'
						className='h-7 border-destructive/30 bg-destructive/15 text-xs text-destructive hover:bg-destructive/15 '
						onClick={() => onCancel(item.id)}
						disabled={isRetrying || isCancelling}>
						{isCancelling ? (
							<Ban className='h-3 w-3 animate-pulse' />
						) : (
							<Ban className='mr-1 h-3 w-3' />
						)}
						{!isCancelling && 'Cancel'}
					</Button>
				</div>
			)}
		</div>
	)
}

/** Live countdown showing how long until the next automatic retry. */
function RetryCountdown({
	nextRetryTimestamp,
	// accentColor,
}: {
	nextRetryTimestamp: number
	accentColor: string
}) {
	const [secondsLeft, setSecondsLeft] = useState(() =>
		Math.max(0, nextRetryTimestamp - Math.floor(Date.now() / 1000))
	)

	useEffect(() => {
		if (secondsLeft <= 0) return
		const id = setInterval(() => {
			setSecondsLeft(() => {
				const next = Math.max(0, nextRetryTimestamp - Math.floor(Date.now() / 1000))
				if (next <= 0) clearInterval(id)
				return next
			})
		}, 1000)
		return () => clearInterval(id)
	}, [nextRetryTimestamp, secondsLeft])

	const label = formatCountdown(secondsLeft)

	return (
		<p className='text-xs tabular-nums' style={{ color: `rgba(var(--accent-rgb), 0.6)` }}>
			{secondsLeft > 0 ? `Retrying in ${label}` : 'Retrying…'}
		</p>
	)
}

function formatCountdown(seconds: number): string {
	if (seconds >= 3600) {
		const h = Math.floor(seconds / 3600)
		const m = Math.floor((seconds % 3600) / 60)
		return m > 0 ? `${h}h ${m}m` : `${h}h`
	}
	if (seconds >= 60) {
		const m = Math.floor(seconds / 60)
		const s = seconds % 60
		return s > 0 ? `${m}m ${s}s` : `${m}m`
	}
	return `${seconds}s`
}
