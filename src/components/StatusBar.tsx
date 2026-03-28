import { useEffect, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import {
	Send,
	CheckCircle,
	AlertCircle,
	Loader2,
	RefreshCw,
	X,
	ChevronUp,
	Mail,
} from 'lucide-react'
import { useOutboxStore, setupOutboxListeners } from '@/stores/outboxStore'
import { useSyncStore, setupSyncListeners } from '@/stores/syncStore'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useAccountStore } from '@/stores/accountStore'
import type { StatusBarProps } from '@/types/components/shared'

export function StatusBar({ onOpenOutbox }: StatusBarProps) {
	const { accounts } = useAccountStore()
	const { t } = useTranslation()
	const { items } = useOutboxStore()
	const { statuses, getAllStatuses, cancelSync, retrySync } = useSyncStore()
	const [pendingCount, setPendingCount] = useState(0)
	const [sendingCount, setSendingCount] = useState(0)
	const [failedCount, setFailedCount] = useState(0)
	const [showSuccess, setShowSuccess] = useState(false)
	const [isSyncMenuOpen, setIsSyncMenuOpen] = useState(false)

	useEffect(() => {
		const pending = items.filter((i) => i.status === 'PENDING' || i.status === 'RETRY').length
		const sending = items.filter((i) => i.status === 'PROCESSING').length
		const failed = items.filter((i) => i.status === 'FAILED').length

		setPendingCount(pending)
		setSendingCount(sending)
		setFailedCount(failed)
	}, [items])

	useEffect(() => {
		let cleanupOutbox: (() => void) | undefined
		let cleanupSync: (() => void) | undefined

		const setup = async () => {
			cleanupOutbox = await setupOutboxListeners(
				undefined,
				() => {
					setShowSuccess(true)
					setTimeout(() => setShowSuccess(false), 3000)
				},
				undefined,
				undefined
			)
			cleanupSync = await setupSyncListeners()
		}
		setup()

		return () => {
			cleanupOutbox?.()
			cleanupSync?.()
		}
	}, [])

	useEffect(() => {
		if (accounts.length === 0) return

		const loadStatuses = () => {
			useSyncStore
				.getState()
				.loadInitialStatuses(accounts.map((a) => ({ id: a.id, email: a.email })))
		}

		loadStatuses()
		const interval = setInterval(loadStatuses, 2000)
		return () => clearInterval(interval)
	}, [accounts])

	const getOutboxStatusIcon = useCallback(() => {
		if (showSuccess)
			return (
				<motion.div
					initial={{ scale: 0 }}
					animate={{ scale: 1 }}
					transition={{ type: 'spring', stiffness: 200, damping: 20 }}>
					<CheckCircle className='h-3 w-3 text-green-400' />
				</motion.div>
			)
		if (sendingCount > 0) return <Loader2 className='h-3 w-3 animate-spin text-amber-400' />
		if (failedCount > 0) return <AlertCircle className='h-3 w-3 text-red-400' />
		if (pendingCount > 0) return <Send className='h-3 w-3 text-blue-400' />
		return <CheckCircle className='h-3 w-3 text-slate-600' />
	}, [sendingCount, failedCount, pendingCount, showSuccess])

	const getOutboxStatusText = useCallback(() => {
		if (showSuccess) return t('statusBar.sent')
		if (sendingCount > 0) return t('statusBar.sending', { count: sendingCount })
		if (failedCount > 0) return t('statusBar.failed', { count: failedCount })
		if (pendingCount > 0) return t('statusBar.pending', { count: pendingCount })
		return t('statusBar.ready')
	}, [sendingCount, failedCount, pendingCount, showSuccess, t])

	const getGlobalSyncStatus = useCallback(() => {
		const allStatuses = getAllStatuses()
		if (allStatuses.length === 0) {
			return { status: 'idle' as const, count: 0 }
		}

		const syncingCount = allStatuses.filter((s) => s.status === 'syncing').length
		const errorCount = allStatuses.filter((s) => s.status === 'error').length

		if (syncingCount > 0) {
			return { status: 'syncing' as const, count: syncingCount }
		}
		if (errorCount > 0) {
			return { status: 'error' as const, count: errorCount }
		}
		return { status: 'idle' as const, count: 0 }
	}, [getAllStatuses])

	const getGlobalSyncIcon = useCallback(() => {
		const globalStatus = getGlobalSyncStatus()
		switch (globalStatus.status) {
			case 'syncing':
				return <Loader2 className='h-3 w-3 animate-spin text-blue-400' />
			case 'error':
				return <AlertCircle className='h-3 w-3 text-red-400' />
			default:
				return <CheckCircle className='h-3 w-3 text-green-500/70' />
		}
	}, [getGlobalSyncStatus])

	const getGlobalSyncText = useCallback(() => {
		const globalStatus = getGlobalSyncStatus()
		switch (globalStatus.status) {
			case 'syncing':
				return t('statusBar.syncingAccounts', {
					count: globalStatus.count,
				})
			case 'error':
				return t('statusBar.syncError', { count: globalStatus.count })
			default:
				return t('statusBar.allSynced')
		}
	}, [getGlobalSyncStatus, t])

	const handleCancelSync = async (accountId: string, e: React.MouseEvent) => {
		e.stopPropagation()
		await cancelSync(accountId)
	}

	const handleRetrySync = async (accountId: string, e: React.MouseEvent) => {
		e.stopPropagation()
		await retrySync(accountId)
	}

	const getAccountStatusDisplay = (accountId: string) => {
		const status = statuses.get(accountId)
		if (!status) return null

		const account = accounts.find((a) => a.id === accountId)
		if (!account) return null

		switch (status.status) {
			case 'syncing': {
				const mailboxCounter = status.mailboxProgress
					? `${status.mailboxProgress.currentMailbox}/${status.mailboxProgress.totalMailboxes}`
					: null
				return (
					<div className='flex items-center gap-2'>
						<Loader2 className='h-3 w-3 animate-spin text-blue-400' />
						<span className='text-foreground/80'>
							{status.mailbox
								? t('statusBar.syncingMailbox', {
										mailbox: status.mailbox,
									})
								: t('statusBar.syncing')}
							{mailboxCounter && ` (${mailboxCounter})`}
						</span>
						<Button
							variant='ghost'
							size='sm'
							className='text-tertiary h-4 w-4 p-0 hover:text-red-400'
							onClick={(e) => handleCancelSync(accountId, e)}>
							<X className='h-3 w-3' />
						</Button>
					</div>
				)
			}
			case 'error':
				return (
					<div className='flex items-center gap-2'>
						<AlertCircle className='h-3 w-3 text-red-400' />
						<span className='max-w-[150px] truncate text-red-400' title={status.error}>
							{status.error || t('statusBar.syncFailed')}
						</span>
						<Button
							variant='ghost'
							size='sm'
							className='text-tertiary h-4 w-4 p-0 hover:text-blue-400'
							onClick={(e) => handleRetrySync(accountId, e)}>
							<RefreshCw className='h-3 w-3' />
						</Button>
					</div>
				)
			default:
				return (
					<div className='flex items-center gap-2'>
						<CheckCircle className='h-3 w-3 text-green-500/70' />
						<span className='text-tertiary'>
							{t('statusBar.lastSynced', {
								time: useSyncStore.getState().getFormattedLastSync(accountId),
							})}
						</span>
					</div>
				)
		}
	}

	const hasActivity = pendingCount > 0 || sendingCount > 0 || failedCount > 0

	return (
		<TooltipProvider>
			<div className='text-muted-foreground relative z-50 flex h-7 shrink-0 items-center justify-between border-t border-[var(--border-subtle)] bg-[var(--app-bg)] px-2 text-xs transition-colors'>
				{/* Background accent glow */}
				<div className='absolute inset-0 bg-gradient-to-r from-transparent via-[var(--accent-color)] to-transparent opacity-[0.02]' />

				{/* Left section - Sync status */}
				<DropdownMenu open={isSyncMenuOpen} onOpenChange={setIsSyncMenuOpen}>
					<DropdownMenuTrigger asChild>
						<Button
							variant='ghost'
							size='sm'
							className='text-muted-foreground hover:text-foreground relative z-10 h-5 gap-1.5 px-2 text-xs transition-colors hover:bg-[var(--surface-hover)]'>
							{getGlobalSyncIcon()}
							<span>{getGlobalSyncText()}</span>
							<div
								className='transition-transform duration-200 ease-out'
								style={{ transform: `rotate(${isSyncMenuOpen ? 180 : 0}deg)` }}>
								<ChevronUp className='h-3 w-3' />
							</div>
						</Button>
					</DropdownMenuTrigger>
					<DropdownMenuContent
						align='start'
						sideOffset={8}
						className='w-72 overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-glass)] p-1 shadow-xl backdrop-blur-xl'>
						<DropdownMenuLabel className='text-muted-foreground px-2 py-1.5 text-xs font-semibold'>
							{t('statusBar.syncStatus')}
						</DropdownMenuLabel>
						<DropdownMenuSeparator className='mx-1 my-1 bg-[var(--border-subtle)]' />
						{accounts.length === 0 ? (
							<DropdownMenuItem
								disabled
								className='text-tertiary px-2 py-1.5 text-xs'>
								{t('statusBar.noAccounts')}
							</DropdownMenuItem>
						) : (
							<div className='flex flex-col gap-0.5'>
								{accounts.map((account) => (
									<DropdownMenuItem
										key={account.id}
										className='flex cursor-default flex-col items-start gap-1.5 rounded-md px-2 py-2 transition-colors focus:bg-[var(--surface-hover)]'
										onSelect={(e) => e.preventDefault()}>
										<div className='flex w-full items-center gap-2'>
											<div className='flex h-5 w-5 shrink-0 items-center justify-center rounded bg-[var(--surface-active)] ring-1 ring-[var(--border-subtle)]'>
												<Mail className='text-muted-foreground h-3 w-3' />
											</div>
											<span className='text-foreground min-w-0 flex-1 truncate text-xs font-medium'>
												{account.email}
											</span>
										</div>
										<div className='w-full pl-7 text-[11px]'>
											{getAccountStatusDisplay(account.id)}
										</div>
									</DropdownMenuItem>
								))}
							</div>
						)}
					</DropdownMenuContent>
				</DropdownMenu>

				{/* Right section - Outbox status */}
				<div className='relative z-10 flex items-center gap-3'>
					<Tooltip>
						<TooltipTrigger asChild>
							<Button
								variant='ghost'
								size='sm'
								className={`h-5 gap-1.5 px-2 text-xs transition-colors hover:bg-[var(--surface-hover)] ${
									hasActivity
										? 'text-foreground/80 hover:text-foreground'
										: 'text-tertiary hover:text-muted-foreground'
								}`}
								onClick={onOpenOutbox}>
								<AnimatePresence mode='wait'>
									<motion.div
										key={
											showSuccess
												? 'success'
												: sendingCount > 0
													? 'sending'
													: failedCount > 0
														? 'failed'
														: 'idle'
										}
										initial={{ scale: 0.5, opacity: 0 }}
										animate={{ scale: 1, opacity: 1 }}
										exit={{ scale: 0.5, opacity: 0 }}
										transition={{ duration: 0.15 }}>
										{getOutboxStatusIcon()}
									</motion.div>
								</AnimatePresence>
								<span>{getOutboxStatusText()}</span>
							</Button>
						</TooltipTrigger>
						<TooltipContent
							side='top'
							className='text-foreground border-[var(--border-subtle)] bg-[var(--surface-glass)]'>
							<p>
								{hasActivity
									? t('statusBar.clickToView')
									: t('statusBar.noMessages')}
							</p>
						</TooltipContent>
					</Tooltip>
				</div>
			</div>
		</TooltipProvider>
	)
}
