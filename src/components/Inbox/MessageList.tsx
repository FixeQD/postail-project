import { useRef, useCallback, useState, useEffect } from 'react'
import { Virtuoso, type VirtuosoHandle } from 'react-virtuoso'
import { useInfiniteQuery, useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { format, isToday, isYesterday, isThisYear } from 'date-fns'
import { Star, Archive, Trash2, MailOpen, Mail, FolderSync } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import type { MailHeader, Mailbox } from '../../types/mail'
import type { AccountMeta } from '../../types/accounts'
import { useTypedTranslation } from '../../hooks/useTypedTranslation'
import { useSettingsStore } from '@/stores/settingsStore'
import { useThemeStore } from '@/stores/themeStore'

interface MessageListProps {
	account: AccountMeta
	mailbox: string
	onMessageClick: (messageId: number) => void
}

const BATCH_SIZE = 50

export const MessageList = ({ account, mailbox, onMessageClick }: MessageListProps) => {
	const { t } = useTypedTranslation()
	const queryClient = useQueryClient()
	const virtuosoRef = useRef<VirtuosoHandle>(null)
	const [hoveredMessageId, setHoveredMessageId] = useState<number | null>(null)
	const [isSyncing, setIsSyncing] = useState(false)
	const [syncError, setSyncError] = useState<string | null>(null)
	const syncingRef = useRef(false)
	const syncedRef = useRef<Set<string>>(new Set())
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
			invoke('unwatch_mailbox', { accountId: account.id }).catch((e) =>
				console.error('Failed to stop mailbox watch:', e)
			)
		}
	}, [account.id, mailbox, isSyncing, syncError, needsSync])

	useEffect(() => {
		const unlisten = listen('sync:completed', (event: { payload: { accountId: string } }) => {
			if (event.payload.accountId === account.id) {
				queryClient.invalidateQueries({
					queryKey: ['messages', account.id, mailbox],
				})
			}
		})

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [account.id, mailbox, queryClient])

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
					p.total > 0
				) {
					queryClient.invalidateQueries({
						queryKey: ['messages', account.id, mailbox],
					})
				}
			}
		)

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [account.id, mailbox, queryClient])

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
			// Block message fetching until sync is done
			enabled: !needsSync && !isSyncing,
		})

	const allMessages = data?.pages.flatMap((page) => page) ?? []

	const formatDate = (dateString: string) => {
		const date = new Date(dateString)
		if (isToday(date)) return format(date, 'HH:mm')
		if (isYesterday(date)) return t('inbox:messageList.date.yesterday')
		if (isThisYear(date)) return format(date, 'MMM d')
		return format(date, 'dd/MM/yyyy')
	}

	const loadMore = useCallback(() => {
		if (hasNextPage && !isFetchingNextPage) {
			fetchNextPage()
		}
	}, [hasNextPage, isFetchingNextPage, fetchNextPage])

	// Syncing state
	if (isSyncing) {
		return (
			<div className='flex h-full items-center justify-center'>
				<motion.div
					initial={{ opacity: 0, scale: 0.9 }}
					animate={{ opacity: 1, scale: 1 }}
					transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
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
					initial={{ opacity: 0, scale: 0.9 }}
					animate={{ opacity: 1, scale: 1 }}
					className='flex flex-col items-center gap-3'>
					<div className='flex h-12 w-12 items-center justify-center rounded-full bg-red-500/10 ring-1 ring-red-500/20'>
						<FolderSync className='h-5 w-5 text-red-400' />
					</div>
					<p className='text-sm font-medium text-red-400'>Failed to sync mailbox</p>
					<p className='max-w-xs text-center text-xs text-slate-600'>{syncError}</p>
					<button
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
					initial={{ opacity: 0, scale: 0.9 }}
					animate={{ opacity: 1, scale: 1 }}
					transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
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
			<Virtuoso
				ref={virtuosoRef}
				data={allMessages}
				endReached={loadMore}
				overscan={200}
				itemContent={(_index, message) => {
					const isUnread = !message.flags.includes('\\Seen')
					const isHovered = hoveredMessageId === message.uid

					return (
						<div
							onMouseEnter={() => setHoveredMessageId(message.uid)}
							onMouseLeave={() => setHoveredMessageId(null)}
							onClick={() => onMessageClick(message.uid)}
							className={`message-unread-indicator group relative flex cursor-pointer items-center border-b border-white/[0.04] px-4 py-3 transition-all duration-150 ${
								isUnread && !zenMode ? 'is-unread' : ''
							} ${
								isUnread && !zenMode
									? 'bg-slate-900/30 hover:bg-slate-900/60'
									: 'bg-transparent hover:bg-white/[0.03]'
							}`}>
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
								<button
									className='rounded-md p-0.5 text-slate-700 transition-colors hover:text-amber-400 focus:outline-none'
									onClick={(e) => e.stopPropagation()}>
									<Star className='h-4 w-4' />
								</button>
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
									{message.from[0]?.replace(/<.*>/, '').trim() ||
										message.from.join(', ')}
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
									<span className='truncate text-xs text-slate-600'>
										- {message.snippet}
									</span>
								</div>
							</div>

							{/* Date or Actions */}
							<div className='ml-3 flex w-24 shrink-0 justify-end'>
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
												icon={<Archive className='h-[15px] w-[15px]' />}
												tooltip={t('inbox:messageList.actions.archive')}
											/>
											<ActionBtn
												icon={<Trash2 className='h-[15px] w-[15px]' />}
												tooltip={t('inbox:messageList.actions.delete')}
												destructive
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
						</div>
					)
				}}
			/>
		</div>
	)
}

const ActionBtn = ({
	icon,
	tooltip,
	destructive,
}: {
	icon: React.ReactNode
	tooltip: string
	destructive?: boolean
}) => (
	<motion.button
		whileHover={{ scale: 1.1 }}
		whileTap={{ scale: 0.85 }}
		className={`flex h-7 w-7 items-center justify-center rounded-lg transition-colors ${
			destructive
				? 'text-slate-400 hover:bg-red-500/10 hover:text-red-400'
				: 'text-slate-400 hover:bg-white/[0.08] hover:text-slate-200'
		}`}
		title={tooltip}
		onClick={(e) => e.stopPropagation()}>
		{icon}
	</motion.button>
)
