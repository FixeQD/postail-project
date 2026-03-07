import { useRef, useCallback, useState, useEffect, useMemo, memo } from 'react'
import { Virtuoso, type VirtuosoHandle } from 'react-virtuoso'
import { useInfiniteQuery, useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { invokeWithErrorLog } from '@/lib/tauri'
import { listen } from '@tauri-apps/api/event'
import { format, isToday, isYesterday, isThisYear } from 'date-fns'
import { Star, Trash2, MailOpen, Mail, FolderSync } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { toast } from '@/components/ui/custom/Toaster'
import type { MailHeader, Mailbox } from '@/types/mail'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useSettingsStore } from '@/stores/settingsStore'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import type { MessageListProps, MessageRowProps } from '@/types/components/inbox'

const BATCH_SIZE = 50

const syncedMailboxes = new Set<string>()

const MessageRow = memo(
	({
		message,
		isUnread,
		isHovered,
		zenMode,
		accentColor,
		animationsEnabled,
		formatDate,
		onMessageClick,
		onMouseEnter,
		onMouseLeave,
		onDelete,
		onToggleRead,
		isFocused,
	}: MessageRowProps) => {
		const { t } = useTypedTranslation()

		return (
			<motion.div
				role='button'
				tabIndex={0}
				onMouseEnter={onMouseEnter}
				onMouseLeave={onMouseLeave}
				onClick={() => onMessageClick(message.uid)}
				onKeyDown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault()
						onMessageClick(message.uid)
					}
				}}
				className={`message-unread-indicator group relative flex w-full cursor-pointer items-center border-b border-white/[0.04] px-4 py-3 text-left transition-all duration-150 outline-none focus-visible:bg-white/[0.05] ${
					isUnread && !zenMode ? 'is-unread' : ''
				} ${
					isFocused
						? 'bg-white/[0.06] shadow-[inset_3px_0_0_0_var(--accent-color)]'
						: isUnread && !zenMode
							? 'bg-slate-900/30 hover:bg-slate-900/60'
							: 'bg-transparent hover:bg-white/[0.03]'
				}`}
				whileHover={
					animationsEnabled
						? {
								scale: 1.01,
								transition: { type: 'spring', damping: 20, stiffness: 300 },
							}
						: {}
				}>
				{/* Checkbox & Star */}
				<div className='flex items-center gap-2.5 pr-3'>
					<input
						type='checkbox'
						className='h-[15px] w-[15px] cursor-pointer rounded border-slate-700 bg-transparent transition-colors focus:ring-1 focus:ring-offset-0'
						style={{
							accentColor: accentColor,
							color: accentColor,
						}}
						onClick={(e) => e.stopPropagation()}
					/>
					<span
						role='button'
						tabIndex={0}
						className='rounded-md p-0.5 text-slate-700 transition-colors hover:text-amber-400 focus:outline-none'
						onClick={(e) => e.stopPropagation()}
						onKeyDown={(e) => {
							if (e.key === 'Enter' || e.key === ' ') {
								e.preventDefault()
								// TODO: implement star toggle
							}
						}}>
						<Star className='h-4 w-4' />
					</span>
				</div>

				{/* Main Content */}
				<div className='flex min-w-0 flex-1 items-baseline gap-3'>
					{/* Sender */}
					<div
						className={`w-44 shrink-0 truncate text-[13px] ${
							isUnread && !zenMode
								? 'font-semibold text-white'
								: 'font-medium text-slate-300'
						}`}>
						{message.from[0]?.replace(/<.*>/, '').trim() || message.from.join(', ')}
					</div>

					{/* Subject & Snippet */}
					<div className='flex min-w-0 flex-1 items-baseline gap-2'>
						<span
							className={`truncate text-[13px] ${
								isUnread && !zenMode
									? 'font-semibold text-slate-200'
									: 'text-slate-400'
							}`}>
							{message.subject || '(No Subject)'}
						</span>
						<span className='truncate text-xs text-slate-600'>- {message.snippet}</span>
					</div>
				</div>

				{/* Date or Actions */}
				<div className='ml-3 flex w-24 shrink-0 justify-end'>
					{animationsEnabled ? (
						<AnimatePresence mode='wait'>
							{isHovered ? (
								<motion.div
									key='actions'
									initial={{ opacity: 0, scale: 0.9 }}
									animate={{ opacity: 1, scale: 1 }}
									exit={{ opacity: 0, scale: 0.9 }}
									transition={{ duration: 0.12 }}
									className='flex items-center gap-0.5'>
									<ActionBtn
										icon={<Trash2 className='h-[15px] w-[15px]' />}
										tooltip={t('inbox:messageList.actions.delete')}
										destructive
										onClick={onDelete}
									/>
									<ActionBtn
										icon={
											isUnread ? (
												<MailOpen className='h-[15px] w-[15px]' />
											) : (
												<Mail className='h-[15px] w-[15px]' />
											)
										}
										tooltip={
											isUnread
												? t('inbox:messageList.actions.markRead')
												: t('inbox:messageList.actions.markUnread')
										}
										onClick={onToggleRead}
									/>
								</motion.div>
							) : (
								<motion.span
									key='date'
									initial={{ opacity: 0 }}
									animate={{ opacity: 1 }}
									exit={{ opacity: 0 }}
									transition={{ duration: 0.1 }}
									className={`text-xs tabular-nums ${
										isUnread && !zenMode
											? 'font-medium text-slate-300'
											: 'text-slate-600'
									}`}>
									{formatDate(message.internal_date)}
								</motion.span>
							)}
						</AnimatePresence>
					) : (
						<span
							className={`text-xs tabular-nums ${
								isUnread && !zenMode
									? 'font-medium text-slate-300'
									: 'text-slate-600'
							}`}>
							{formatDate(message.internal_date)}
						</span>
					)}
				</div>

				{/* Unread dot indicator (right side) */}
				{isUnread && !zenMode && (
					<div
						className='ml-2 h-2 w-2 shrink-0 rounded-full'
						style={{
							backgroundColor: accentColor,
							boxShadow: `0 1px 3px rgba(var(--accent-rgb), 0.3)`,
						}}
					/>
				)}
			</motion.div>
		)
	}
)

export const MessageList = ({ account, mailbox, focusedUid, onMessageClick }: MessageListProps) => {
	const { t } = useTypedTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const queryClient = useQueryClient()
	const virtuosoRef = useRef<VirtuosoHandle>(null)
	const [hoveredMessageId, setHoveredMessageId] = useState<number | null>(null)
	const [isSyncing, setIsSyncing] = useState(false)
	const [syncError, setSyncError] = useState<string | null>(null)
	const syncingRef = useRef(false)
	const syncedRef = { current: syncedMailboxes }
	const { settings } = useSettingsStore()
	const zenMode = settings['zen-mode']
	const accentColor = useThemeStore((s) => s.accentColor)

	const mailboxKey = `${account.id}:${mailbox}`

	const { data: mailboxes, isLoading: mailboxesLoading } = useQuery({
		queryKey: ['mailboxes', account.id],
		queryFn: () => invoke<Mailbox[]>('fetch_mailboxes', { accountId: account.id }),
		enabled: !!account,
	})

	const currentMailbox = mailboxes?.find((m) => m.name === mailbox)

	const needsSync = (() => {
		if (syncedRef.current.has(mailboxKey)) return false
		if (mailboxesLoading || !mailboxes) return true
		if (!currentMailbox) return true
		return !currentMailbox.last_synced_uid
	})()

	useEffect(() => {
		syncingRef.current = false
		setIsSyncing(false)
		setSyncError(null)

		if (!needsSync) return

		let cancelled = false
		const doSync = async () => {
			syncingRef.current = true
			setIsSyncing(true)
			try {
				await invoke('sync_single_mailbox', {
					accountId: account.id,
					mailbox,
				})
				if (!cancelled) {
					syncedRef.current.add(mailboxKey)
				}
			} catch (e) {
				if (!cancelled) {
					setSyncError(String(e))
				}
			} finally {
				syncingRef.current = false
				setIsSyncing(false)
				queryClient.invalidateQueries({ queryKey: ['mailboxes', account.id] })
				queryClient.invalidateQueries({
					queryKey: ['messages', account.id, mailbox],
				})
			}
		}
		doSync()

		return () => {
			cancelled = true
		}
	}, [needsSync, account.id, mailbox, mailboxKey, queryClient])

	useEffect(() => {
		if (isSyncing || syncError || needsSync) return

		let stopped = false

		const startWatch = async () => {
			try {
				await invoke('watch_mailbox', { accountId: account.id, mailbox })
			} catch (e) {
				if (!stopped) {
					console.error('Failed to start mailbox watch:', e)
				}
			}
		}
		startWatch()

		return () => {
			stopped = true
			invokeWithErrorLog(
				'unwatch_mailbox',
				{ accountId: account.id, mailbox },
				'unwatch_mailbox'
			)
		}
	}, [account.id, mailbox, isSyncing, syncError, needsSync])

	const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading, error } =
		useInfiniteQuery({
			queryKey: ['messages', account.id, mailbox],
			queryFn: async ({ pageParam }) => {
				const anchor = pageParam as number | undefined
				const messages = await invoke<MailHeader[]>('fetch_headers', {
					accountId: account.id,
					mailbox,
					anchor,
					limit: BATCH_SIZE,
				})
				return messages
			},
			initialPageParam: undefined as number | undefined,
			getNextPageParam: (lastPage: MailHeader[]) => {
				if (lastPage.length < BATCH_SIZE) return undefined
				const lastMessage = lastPage[lastPage.length - 1]
				return lastMessage?.uid
			},

			enabled: !needsSync && !isSyncing,
		})

	const allMessages = useMemo(() => data?.pages.flatMap((page) => page) ?? [], [data?.pages])

	// Force-refresh infinite query by removing cache, triggering a full refetch from page 1
	const refreshMessages = useCallback(() => {
		queryClient.invalidateQueries({ queryKey: ['messages', account.id, mailbox] })
	}, [account.id, mailbox, queryClient])

	useEffect(() => {
		const unlisten = listen('sync:completed', (event: { payload: { accountId: string } }) => {
			if (event.payload.accountId === account.id) {
				if (syncedRef.current.has(mailboxKey)) {
					refreshMessages()
				}
			}
		})

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [account.id, refreshMessages])

	useEffect(() => {
		const unlisten = listen(
			'sync:new_messages',
			(event: { payload: { accountId: string; mailbox: string; count: number } }) => {
				const p = event.payload
				if (p.accountId === account.id && p.mailbox === mailbox) {
					refreshMessages()
					queryClient.invalidateQueries({
						queryKey: ['mailboxes', account.id],
					})
				}
			}
		)

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [account.id, mailbox, refreshMessages, queryClient])

	useEffect(() => {
		const unlisten = listen(
			'sync:progress',
			(event: {
				payload: { accountId: string; mailbox: string; current: number; total: number }
			}) => {
				const p = event.payload
				if (
					p.accountId === account.id &&
					p.mailbox === mailbox &&
					p.current === p.total &&
					p.total > 0 &&
					syncedRef.current.has(mailboxKey)
				) {
					refreshMessages()
				}
			}
		)

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [account.id, mailbox, refreshMessages])

	const formatDate = useCallback(
		(dateString: string) => {
			const date = new Date(dateString)
			if (isToday(date)) return format(date, 'HH:mm')
			if (isYesterday(date)) return t('inbox:messageList.date.yesterday')
			if (isThisYear(date)) return format(date, 'MMM d')
			return format(date, 'dd/MM/yyyy')
		},
		[t]
	)

	const loadMore = useCallback(() => {
		if (hasNextPage && !isFetchingNextPage) {
			fetchNextPage()
		}
	}, [hasNextPage, isFetchingNextPage, fetchNextPage])

	const handleDeleteMessage = useCallback(
		async (uid: number) => {
			const queryKey = ['messages', account.id, mailbox] as const
			const previousData = queryClient.getQueryData<{
				pages: MailHeader[][]
			}>(queryKey)

			if (previousData) {
				const newPages = previousData.pages.map((page: MailHeader[]) =>
					page.filter((m: MailHeader) => m.uid !== uid)
				)
				queryClient.setQueryData(queryKey, {
					...previousData,
					pages: newPages,
				})
			}

			try {
				await invoke('delete_messages', {
					accountId: account.id,
					mailbox,
					uids: [uid],
				})
				const trashMailbox = mailboxes?.find((m) => m.role === 'trash')
				if (trashMailbox) {
					queryClient.invalidateQueries({
						queryKey: ['messages', account.id, trashMailbox.name],
					})
				}
			} catch (error) {
				// Rollback on error
				if (previousData) {
					queryClient.setQueryData(queryKey, previousData)
				}
				toast.error('Failed to delete message', {
					description: String(error),
				})
			}
		},
		[account.id, mailbox, queryClient, mailboxes]
	)

	const handleToggleReadStatus = useCallback(
		async (uid: number, currentlyUnread: boolean) => {
			try {
				await invoke('mark_read', {
					accountId: account.id,
					mailbox,
					uids: [uid],
					read: currentlyUnread,
				})
				queryClient.invalidateQueries({
					queryKey: ['messages', account.id, mailbox],
				})
			} catch (error) {
				toast.error(`Failed to mark message as ${currentlyUnread ? 'read' : 'unread'}`, {
					description: String(error),
				})
			}
		},
		[account.id, mailbox, queryClient]
	)

	// Syncing state
	if (isSyncing || needsSync) {
		return (
			<div className='flex h-full items-center justify-center'>
				<motion.div
					{...(animationsEnabled
						? {
								initial: { opacity: 0, scale: 0.9 },
								animate: { opacity: 1, scale: 1 },
								transition: { duration: 0.3, ease: [0.16, 1, 0.3, 1] },
							}
						: {})}
					className='flex flex-col items-center gap-4'>
					<div className='relative flex h-16 w-16 items-center justify-center'>
						<div
							className='absolute inset-0 animate-spin rounded-full border-2 border-transparent'
							style={{
								borderTopColor: accentColor,
								animationDuration: '1.2s',
							}}
						/>
						<div
							className='absolute inset-2 animate-spin rounded-full border-2 border-transparent'
							style={{
								borderBottomColor: `rgba(var(--accent-rgb), 0.3)`,
								animationDirection: 'reverse',
								animationDuration: '1.8s',
							}}
						/>
						<FolderSync className='h-6 w-6' style={{ color: accentColor }} />
					</div>
					<div className='flex flex-col items-center gap-1'>
						<span className='text-sm font-medium text-slate-300'>
							Syncing messages...
						</span>
						<span className='text-xs text-slate-600'>
							{currentMailbox?.display_name || mailbox}
						</span>
					</div>
				</motion.div>
			</div>
		)
	}

	// Sync error state
	if (syncError) {
		return (
			<div className='flex h-full items-center justify-center'>
				<motion.div
					{...(animationsEnabled
						? {
								initial: { opacity: 0, scale: 0.9 },
								animate: { opacity: 1, scale: 1 },
							}
						: {})}
					className='flex flex-col items-center gap-3'>
					<div className='flex h-12 w-12 items-center justify-center rounded-full bg-red-500/10 ring-1 ring-red-500/20'>
						<FolderSync className='h-5 w-5 text-red-400' />
					</div>
					<p className='text-sm font-medium text-red-400'>Failed to sync mailbox</p>
					<p className='max-w-xs text-center text-xs text-slate-600'>{syncError}</p>
					<button
						type='button'
						onClick={() => {
							syncedRef.current.delete(mailboxKey)
							setSyncError(null)
							syncingRef.current = false
							setIsSyncing(false)
							queryClient.invalidateQueries({
								queryKey: ['mailboxes', account.id],
							})
						}}
						className='mt-1 rounded-lg px-4 py-1.5 text-xs font-medium text-slate-300 ring-1 ring-white/[0.08] transition-colors hover:bg-white/[0.04]'>
						Retry
					</button>
				</motion.div>
			</div>
		)
	}

	if (isLoading) {
		return (
			<div className='flex h-full items-center justify-center'>
				<div className='flex flex-col items-center gap-3'>
					<div className='relative h-10 w-10'>
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
					<span className='text-sm text-slate-500'>Loading messages...</span>
				</div>
			</div>
		)
	}

	if (error) {
		return (
			<div className='flex h-full items-center justify-center'>
				<div className='flex flex-col items-center gap-2 text-red-400'>
					<div className='flex h-12 w-12 items-center justify-center rounded-full bg-red-500/10 ring-1 ring-red-500/20'>
						<Mail className='h-5 w-5' />
					</div>
					<p className='text-sm font-medium'>{t('inbox:messageList.error')}</p>
				</div>
			</div>
		)
	}

	if (allMessages.length === 0) {
		return (
			<div className='flex h-full flex-col items-center justify-center text-slate-500'>
				<motion.div
					{...(animationsEnabled
						? {
								initial: { opacity: 0, scale: 0.9 },
								animate: { opacity: 1, scale: 1 },
								transition: { duration: 0.4, ease: [0.16, 1, 0.3, 1] },
							}
						: {})}
					className='flex flex-col items-center'>
					<div className='flex h-24 w-24 items-center justify-center rounded-2xl bg-slate-900/50 ring-1 ring-white/[0.06]'>
						<Mail className='h-10 w-10 text-slate-700' />
					</div>
					<p className='mt-5 text-sm font-medium text-slate-400'>
						{t('inbox:messageList.empty.title')}
					</p>
					<p className='mt-1 text-xs text-slate-600'>
						{t('inbox:messageList.empty.subtitle')}
					</p>
				</motion.div>
			</div>
		)
	}

	return (
		<div className='flex h-full flex-1 flex-col'>
			<div className='smooth-scroll-container'>
				<Virtuoso
					ref={virtuosoRef}
					data={allMessages}
					endReached={loadMore}
					overscan={200}
					itemContent={(_index, message) => {
						const isUnread = !message.flags.includes('\\Seen')
						const isHovered = hoveredMessageId === message.uid

						return (
							<MessageRow
								message={message}
								isUnread={isUnread}
								isHovered={isHovered}
								isFocused={message.uid === focusedUid}
								zenMode={zenMode}
								accentColor={accentColor}
								animationsEnabled={animationsEnabled}
								formatDate={formatDate}
								onMessageClick={onMessageClick}
								onMouseEnter={() => setHoveredMessageId(message.uid)}
								onMouseLeave={() => setHoveredMessageId(null)}
								onDelete={() => handleDeleteMessage(message.uid)}
								onToggleRead={() => handleToggleReadStatus(message.uid, isUnread)}
							/>
						)
					}}
				/>
				{isFetchingNextPage && (
					<div className='flex items-center justify-center py-4'>
						<motion.div
							{...(animationsEnabled
								? {
										initial: { opacity: 0, scale: 0.9 },
										animate: { opacity: 1, scale: 1 },
									}
								: {})}
							className='flex items-center gap-2'>
							<div
								className='h-4 w-4 animate-spin rounded-full border-2 border-transparent'
								style={{ borderTopColor: accentColor }}
							/>
							<span className='text-xs text-slate-400'>
								{t('inbox:messageList.loadingMore')}
							</span>
						</motion.div>
					</div>
				)}
			</div>
		</div>
	)
}

const ActionBtn = ({
	icon,
	tooltip,
	destructive,
	onClick,
}: {
	icon: React.ReactNode
	tooltip: string
	destructive?: boolean
	onClick?: (e: React.MouseEvent) => void
}) => {
	const animationsEnabled = useAnimationsEnabled()
	return (
		<motion.button
			type='button'
			{...(animationsEnabled
				? { whileHover: { scale: 1.1 }, whileTap: { scale: 0.85 } }
				: {})}
			className={`flex h-7 w-7 items-center justify-center rounded-lg transition-colors ${
				destructive
					? 'text-slate-400 hover:bg-red-500/10 hover:text-red-400'
					: 'text-slate-400 hover:bg-white/[0.08] hover:text-slate-200'
			}`}
			title={tooltip}
			onClick={(e) => {
				e.stopPropagation()
				onClick?.(e)
			}}>
			{icon}
		</motion.button>
	)
}
