import { useEffect, useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
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
import type { AccountMeta } from '@/types/accounts'

interface StatusBarProps {
	onOpenOutbox: () => void
	accounts: AccountMeta[]
}

export function StatusBar({ onOpenOutbox, accounts }: StatusBarProps) {
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
				undefined, // processing
				() => {
					setShowSuccess(true)
					setTimeout(() => setShowSuccess(false), 3000)
				},
				undefined, // retry
				undefined // failed
			)
			cleanupSync = await setupSyncListeners()
		}
		setup()

		return () => {
			cleanupOutbox?.()
			cleanupSync?.()
		}
	}, [])

	// Load initial sync statuses for all accounts and poll every 2 seconds
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
		if (showSuccess) return <CheckCircle className='h-3 w-3 text-green-500' />
		if (sendingCount > 0) return <Loader2 className='h-3 w-3 animate-spin text-yellow-500' />
		if (failedCount > 0) return <AlertCircle className='h-3 w-3 text-red-500' />
		if (pendingCount > 0) return <Send className='h-3 w-3 text-blue-500' />
		return <CheckCircle className='h-3 w-3 text-zinc-500' />
	}, [sendingCount, failedCount, pendingCount, showSuccess])

	const getOutboxStatusText = useCallback(() => {
		if (showSuccess) return t('statusBar.sent', 'Message sent!')
		if (sendingCount > 0)
			return t('statusBar.sending', 'Sending {{count}}...', { count: sendingCount })
		if (failedCount > 0)
			return t('statusBar.failed', '{{count}} failed', { count: failedCount })
		if (pendingCount > 0)
			return t('statusBar.pending', '{{count}} pending', { count: pendingCount })
		return t('statusBar.ready', 'Ready')
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
				return <Loader2 className='h-3 w-3 animate-spin text-blue-500' />
			case 'error':
				return <AlertCircle className='h-3 w-3 text-red-500' />
			default:
				return <CheckCircle className='h-3 w-3 text-green-500' />
		}
	}, [getGlobalSyncStatus])

	const getGlobalSyncText = useCallback(() => {
		const globalStatus = getGlobalSyncStatus()
		switch (globalStatus.status) {
			case 'syncing':
				return t('statusBar.syncingAccounts', 'Syncing {{count}}...', {
					count: globalStatus.count,
				})
			case 'error':
				return t('statusBar.syncError', '{{count}} errors', { count: globalStatus.count })
			default:
				return t('statusBar.allSynced', 'All synced')
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
						<Loader2 className='h-3 w-3 animate-spin text-blue-500' />
						<span className='text-zinc-300'>
							{status.mailbox
								? t('statusBar.syncingMailbox', 'Syncing {{mailbox}}', {
										mailbox: status.mailbox,
									})
								: t('statusBar.syncing', 'Syncing...')}
							{mailboxCounter && ` (${mailboxCounter})`}
						</span>
						<Button
							variant='ghost'
							size='sm'
							className='h-4 w-4 p-0 text-zinc-500 hover:text-red-400'
							onClick={(e) => handleCancelSync(accountId, e)}>
							<X className='h-3 w-3' />
						</Button>
					</div>
				)
			}
			case 'error':
				return (
					<div className='flex items-center gap-2'>
						<AlertCircle className='h-3 w-3 text-red-500' />
						<span className='max-w-[150px] truncate text-red-400' title={status.error}>
							{status.error || t('statusBar.syncFailed', 'Sync failed')}
						</span>
						<Button
							variant='ghost'
							size='sm'
							className='h-4 w-4 p-0 text-zinc-500 hover:text-blue-400'
							onClick={(e) => handleRetrySync(accountId, e)}>
							<RefreshCw className='h-3 w-3' />
						</Button>
					</div>
				)
			default:
				return (
					<div className='flex items-center gap-2'>
						<CheckCircle className='h-3 w-3 text-green-500' />
						<span className='text-zinc-500'>
							{t('statusBar.lastSynced', 'Last synced {{time}}', {
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
			<div className='flex h-6 shrink-0 items-center justify-between border-t border-zinc-800 bg-zinc-950 px-2 text-xs text-zinc-500'>
				{/* Left section - Sync status */}
				<DropdownMenu open={isSyncMenuOpen} onOpenChange={setIsSyncMenuOpen}>
					<DropdownMenuTrigger asChild>
						<Button
							variant='ghost'
							size='sm'
							className='h-5 gap-1.5 px-2 text-xs text-zinc-500 hover:text-zinc-300'>
							{getGlobalSyncIcon()}
							<span>{getGlobalSyncText()}</span>
							<ChevronUp
								className={`h-3 w-3 transition-transform ${isSyncMenuOpen ? 'rotate-180' : ''}`}
							/>
						</Button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align='start' className='w-72 border-zinc-800 bg-zinc-900'>
						<DropdownMenuLabel className='text-zinc-400'>
							{t('statusBar.syncStatus', 'Sync Status')}
						</DropdownMenuLabel>
						<DropdownMenuSeparator className='bg-zinc-800' />
						{accounts.length === 0 ? (
							<DropdownMenuItem disabled className='text-zinc-600'>
								{t('statusBar.noAccounts', 'No accounts added')}
							</DropdownMenuItem>
						) : (
							accounts.map((account) => (
								<DropdownMenuItem
									key={account.id}
									className='flex cursor-default flex-col items-start gap-1 py-2'
									onSelect={(e) => e.preventDefault()}>
									<div className='flex w-full items-center gap-2'>
										<Mail className='h-3 w-3 text-zinc-500' />
										<span className='truncate font-medium text-zinc-300'>
											{account.email}
										</span>
									</div>
									<div className='w-full pl-5'>
										{getAccountStatusDisplay(account.id)}
									</div>
								</DropdownMenuItem>
							))
						)}
					</DropdownMenuContent>
				</DropdownMenu>

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
								{getOutboxStatusIcon()}
								<span>{getOutboxStatusText()}</span>
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
