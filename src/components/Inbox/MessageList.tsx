import { useRef, useCallback, useState, useEffect, useMemo, memo } from 'react'
import { Virtuoso, type VirtuosoHandle } from 'react-virtuoso'
import { useInfiniteQuery, useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

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
import { Loader2 } from 'lucide-react'
import type {
	MessageListProps,
	MessageRowProps,
	DragMessagePayload,
} from '@/types/components/inbox'

const BATCH_SIZE = 50

const syncedMailboxes = new Set<string>()

const DateOrActions = memo(
	({
		isHovered,
		isUnread,
		zenMode,
		animationsEnabled,
		formattedDate,
		onDelete,
		onToggleRead,
		t,
		isOptimistic,
	}: {
		isHovered: boolean
		isUnread: boolean
		zenMode: boolean
		animationsEnabled: boolean
		formattedDate: string
		onDelete: () => void
		onToggleRead: () => void
		t: (key: string) => string
		isOptimistic?: boolean
	}) => {
		if (isOptimistic) {
			return (
				<div className='text-muted-foreground flex items-center gap-1.5 text-xs font-medium'>
					<Loader2 className='h-3 w-3 animate-spin' />
					{t('inbox:messageList.moving')}
				</div>
			)
		}

		const dateClass = `text-xs tabular-nums ${
			isUnread && !zenMode ? 'text-foreground/80 font-medium' : 'text-tertiary'
		}`

		const actions = (
			<div className='flex items-center gap-0.5'>
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
			</div>
		)

		if (!animationsEnabled) {
			return isHovered ? actions : <span className={dateClass}>{formattedDate}</span>
		}

		return (
			<AnimatePresence mode='wait'>
				{isHovered ? (
					<motion.div
						key='actions'
						initial={{ opacity: 0, x: 8 }}
						animate={{ opacity: 1, x: 0 }}
						exit={{ opacity: 0, x: 8 }}
						transition={{ duration: 0.15, ease: 'easeOut' }}>
						{actions}
					</motion.div>
				) : (
					<motion.span
						key='date'
						initial={{ opacity: 0, x: -8 }}
						animate={{ opacity: 1, x: 0 }}
						exit={{ opacity: 0, x: -8 }}
						transition={{ duration: 0.15, ease: 'easeOut' }}
						className={dateClass}>
						{formattedDate}
					</motion.span>
				)}
			</AnimatePresence>
		)
	}
)

const MessageRow = memo(
	({
		message,
		isUnread,
		zenMode,
		accentColor,
		animationsEnabled,
		previewLines,
		formatDate,
		onMessageClick,
		onDelete,
		onToggleRead,
		onToggleStar,
		isFocused,
	}: MessageRowProps) => {
		const { t } = useTypedTranslation()
		const [isHovered, setIsHovered] = useState(false)

		const sender = message.from[0]?.replace(/<.*>/g, '').trim() || message.from.join(', ')
		const subject = message.subject || '(No Subject)'
		const snippet = message.snippet ?? ''
		const formattedDate = formatDate(message.internal_date)

		const senderClass = `truncate text-[13px] leading-tight ${
			isUnread && !zenMode
				? 'text-foreground font-semibold'
				: 'text-foreground/80 font-medium'
		}`
		const subjectClass = `text-[13px] leading-snug ${
			isUnread && !zenMode ? 'text-foreground font-semibold' : 'text-foreground/70'
		}`
		const snippetClass = 'text-xs leading-snug text-[var(--text-tertiary)]'

		const isOptimistic = message.uid < 0

		const rowBase = `message-unread-indicator group relative flex w-full cursor-pointer select-none border-b text-left transition-all duration-200 outline-none ${
			isUnread && !zenMode ? 'is-unread' : ''
		} ${
			isFocused
				? 'z-10 shadow-sm'
				: isUnread && !zenMode
					? 'bg-[var(--surface-panel)] hover:bg-[var(--surface-hover)] hover:z-10 hover:shadow-sm'
					: 'bg-transparent hover:bg-[var(--surface-panel)] hover:z-10 hover:shadow-sm'
		} ${isOptimistic ? 'opacity-60 cursor-wait' : ''}`

		const focusedStyle = isFocused ? { backgroundColor: `${accentColor}10` } : {}

		const activeIndicator = isFocused && (
			<div
				className='absolute top-0 bottom-0 left-0 w-[3px] transition-all duration-150'
				style={{
					backgroundColor: accentColor,
					boxShadow: `1px 0 8px ${accentColor}80`,
				}}
			/>
		)

		const checkboxStar = (
			<div className='flex shrink-0 items-center gap-2.5 pr-3'>
				<input
					type='checkbox'
					className='border-muted-foreground/40 h-[15px] w-[15px] cursor-pointer rounded border bg-transparent transition-colors focus:ring-1 focus:ring-offset-0'
					style={{ accentColor, color: accentColor }}
					onClick={(e) => e.stopPropagation()}
				/>
				<motion.button
					type='button'
					{...(animationsEnabled ? { whileTap: { scale: 0.75 } } : {})}
					className={`rounded-md p-0.5 transition-colors focus:outline-none ${
						message.starred
							? 'text-amber-400 hover:text-amber-300'
							: 'text-muted-foreground/40 hover:text-amber-400'
					}`}
					onClick={(e) => {
						e.stopPropagation()
						onToggleStar(message.uid, message.mailbox)
					}}
					aria-label={message.starred ? 'Unstar message' : 'Star message'}
					aria-pressed={message.starred}>
					<motion.div
						{...(animationsEnabled
							? {
									animate: message.starred
										? { scale: [1, 1.35, 1], rotate: [0, 15, -10, 0] }
										: { scale: 1, rotate: 0 },
									transition: { duration: 0.35, ease: 'easeOut' },
								}
							: {})}>
						<Star
							className='h-4 w-4'
							fill={message.starred ? 'currentColor' : 'none'}
						/>
					</motion.div>
				</motion.button>
			</div>
		)

		const unreadDot = isUnread && !zenMode && (
			<div
				className='ml-2 h-2.5 w-2.5 shrink-0 self-center rounded-full'
				style={{
					backgroundColor: accentColor,
					boxShadow: `0 0 8px ${accentColor}80`,
				}}
			/>
		)

		const tagPills = message.tags?.length > 0 && (
			<div className='mt-1.5 flex flex-wrap gap-1'>
				{message.tags
					.filter((tag) => tag && tag !== 'null')
					.map((tag) => (
						<span
							key={tag}
							className='text-tertiary rounded bg-[var(--surface-active)] px-1.5 py-0.5 text-[10px] font-medium whitespace-nowrap ring-1 ring-[var(--border-subtle)]'>
							{tag}-
						</span>
					))}
			</div>
		)

		// ── 1-line compact layout ──────────────────────────────────────
		if (previewLines === 1) {
			return (
				<motion.div
					role='button'
					tabIndex={0}
					onMouseEnter={() => setIsHovered(true)}
					onMouseLeave={() => setIsHovered(false)}
					onClick={(e) => {
						if (isOptimistic) {
							e.preventDefault()
							return
						}
						onMessageClick(message.uid, message.mailbox)
					}}
					onKeyDown={(e) => {
						if (isOptimistic) return
						if (e.key === 'Enter' || e.key === ' ') {
							e.preventDefault()
							onMessageClick(message.uid, message.mailbox)
						}
					}}
					className={`${rowBase} items-center px-4 py-3 transition-transform duration-150 hover:scale-[1.01]`}
					style={{ borderColor: 'var(--border-faint)', ...focusedStyle }}>
					{activeIndicator}
					{checkboxStar}

					<div className='flex min-w-0 flex-1 items-center gap-3'>
						<div className={`w-44 shrink-0 ${senderClass}`}>{sender}</div>
						<div className='flex min-w-0 flex-1 items-baseline gap-1.5'>
							<span className={`max-w-[45%] shrink-0 truncate ${subjectClass}`}>
								{subject}
							</span>
							{snippet && (
								<span className={`truncate ${snippetClass}`}>— {snippet}</span>
							)}
							{message.tags?.length > 0 && (
								<div className='ml-2 flex gap-1 opacity-80'>
									{message.tags
										.filter((t) => t && t !== 'null')
										.map((tag) => (
											<div
												key={tag}
												className='h-1.5 w-1.5 rounded-full'
												style={{ backgroundColor: accentColor }}
												title={tag}
											/>
										))}
								</div>
							)}
						</div>
					</div>

					<div className='ml-3 flex w-24 shrink-0 justify-end'>
						<DateOrActions
							isHovered={isHovered}
							isUnread={isUnread}
							zenMode={zenMode}
							animationsEnabled={animationsEnabled}
							formattedDate={formattedDate}
							onDelete={() => onDelete(message.uid, message.mailbox)}
							onToggleRead={() =>
								onToggleRead(message.uid, isUnread, message.mailbox)
							}
							t={t}
							isOptimistic={isOptimistic}
						/>
					</div>

					{unreadDot}
				</motion.div>
			)
		}

		// ── 2 or 3-line card layout ────────────────────────────────────
		return (
			<motion.div
				role='button'
				tabIndex={0}
				onMouseEnter={() => setIsHovered(true)}
				onMouseLeave={() => setIsHovered(false)}
				onClick={(e) => {
					if (isOptimistic) {
						e.preventDefault()
						return
					}
					onMessageClick(message.uid, message.mailbox)
				}}
				onKeyDown={(e) => {
					if (isOptimistic) return
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault()
						onMessageClick(message.uid, message.mailbox)
					}
				}}
				className={`${rowBase} items-start px-4 py-3 transition-transform duration-150 hover:scale-[1.005]`}
				style={{ borderColor: 'var(--border-faint)', ...focusedStyle }}>
				{activeIndicator}
				{checkboxStar}

				<div className='flex min-w-0 flex-1 flex-col gap-0.5'>
					{/* Row 1: sender + date */}
					<div className='flex items-center justify-between gap-2'>
						<span className={`${senderClass} min-w-0`}>{sender}</span>
						<div className='ml-2 flex shrink-0 items-center'>
							<DateOrActions
								isHovered={isHovered}
								isUnread={isUnread}
								zenMode={zenMode}
								animationsEnabled={animationsEnabled}
								formattedDate={formattedDate}
								onDelete={() => onDelete(message.uid, message.mailbox)}
								onToggleRead={() =>
									onToggleRead(message.uid, isUnread, message.mailbox)
								}
								t={t}
								isOptimistic={isOptimistic}
							/>
							{unreadDot}
						</div>
					</div>

					{/* Row 2: subject */}
					<span className={`${subjectClass} truncate`}>{subject}</span>

					{/* Row 3 (2-line: snippet inline after subject / 3-line: separate) */}
					{snippet && previewLines === 2 && (
						<p className={`${snippetClass} line-clamp-1`}>{snippet}</p>
					)}
					{snippet && previewLines === 3 && (
						<p className={`${snippetClass} line-clamp-2`}>{snippet}</p>
					)}

					{/* Tags Row */}
					{tagPills}
				</div>
			</motion.div>
		)
	}
)

export const MessageList = ({ account, mailbox, focusedUid, onMessageClick }: MessageListProps) => {
	const { t } = useTypedTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const queryClient = useQueryClient()
	const virtuosoRef = useRef<VirtuosoHandle>(null)
	const [isSyncing, setIsSyncing] = useState(false)
	const [syncError, setSyncError] = useState<string | null>(null)
	const syncingRef = useRef(false)
	const syncedRef = { current: syncedMailboxes }
	const { settings } = useSettingsStore()
	const zenMode = settings['zen-mode']
	const previewLines = settings['preview-lines']
	const accentColor = useThemeStore((s) => s.accentColor)

	const mailboxKey = `${account.id}:${mailbox}`

	const { data: mailboxes, isLoading: mailboxesLoading } = useQuery({
		queryKey: ['mailboxes', account.id],
		queryFn: () => invoke<Mailbox[]>('fetch_mailboxes', { accountId: account.id }),
		enabled: !!account,
	})

	const currentMailbox = mailboxes?.find((m) => m.name === mailbox)

	const needsSync = (() => {
		if (mailbox.startsWith('Virtual_')) return false
		if (syncedRef.current.has(mailboxKey)) return false
		if (mailboxesLoading || !mailboxes) return true
		if (!currentMailbox) return true
		return !currentMailbox.last_synced_uid
	})()

	useEffect(() => {
		syncingRef.current = false
		setIsSyncing(false)
		setSyncError(null)

		if (!needsSync || mailbox.startsWith('Virtual_')) return

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
		if (isSyncing || syncError || needsSync || mailbox.startsWith('Virtual_')) return

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
				if (mailbox.startsWith('Virtual_')) return undefined
				if (lastPage.length < BATCH_SIZE) return undefined
				const lastMessage = lastPage[lastPage.length - 1]
				return lastMessage?.uid
			},

			enabled: !needsSync && !isSyncing,
			staleTime: 1000,
		})

	const allMessages = useMemo(() => data?.pages.flatMap((page) => page) ?? [], [data?.pages])

	// Force-refresh infinite query by removing cache, triggering a full refetch from page 1
	const refreshMessages = useCallback(() => {
		queryClient.invalidateQueries({ queryKey: ['messages', account.id, mailbox] })
	}, [account.id, mailbox, queryClient])

	// After first page loads, backfill snippets for messages that are missing them
	useEffect(() => {
		if (!data?.pages[0]?.length || mailbox.startsWith('Virtual_')) return
		const hasMissingSnippets = data.pages[0].some((m) => !m.snippet)
		if (!hasMissingSnippets) return

		invoke('backfill_snippets', { accountId: account.id, mailbox })
			.then((count) => {
				if ((count as number) > 0) {
					refreshMessages()
				}
			})
			.catch(() => {})
	}, [data?.pages[0], account.id, mailbox, refreshMessages])

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
		async (uid: number, msgMailbox: string) => {
			const queryKey = ['messages', account.id, mailbox] as const
			const previousData = queryClient.getQueryData<{
				pages: MailHeader[][]
				pageParams: unknown[]
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
					mailbox: msgMailbox,
					uids: [uid],
				})
				const trashMailbox = mailboxes?.find((m) => m.role === 'trash')
				if (trashMailbox) {
					invoke('sync_single_mailbox', {
						accountId: account.id,
						mailbox: trashMailbox.name,
					}).catch(console.error)

					queryClient.invalidateQueries({
						queryKey: ['messages', account.id, trashMailbox.name],
					})
				}

				// Invalidate current mailbox to ensure it reflects the server state properly
				queryClient.invalidateQueries({
					queryKey,
				})
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
		async (uid: number, currentlyUnread: boolean, msgMailbox: string) => {
			try {
				await invoke('mark_read', {
					accountId: account.id,
					mailbox: msgMailbox,
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

	const handleToggleStar = useCallback(
		async (uid: number, msgMailbox: string) => {
			const queryKey = ['messages', account.id, mailbox]
			const previousData = queryClient.getQueryData(queryKey)

			queryClient.setQueryData(queryKey, (old: unknown) => {
				if (!old || typeof old !== 'object') return old
				const typedOld = old as { pages: MailHeader[][] }
				return {
					...typedOld,
					pages: typedOld.pages.map((page) =>
						page.map((msg) =>
							msg.uid === uid ? { ...msg, starred: !msg.starred } : msg
						)
					),
				}
			})

			try {
				await invoke('toggle_starred', {
					accountId: account.id,
					mailbox: msgMailbox,
					uid,
				})
			} catch (error) {
				// Roll back on failure
				queryClient.setQueryData(queryKey, previousData)
				toast.error('Failed to toggle star', { description: String(error) })
			}
		},
		[account.id, mailbox, queryClient]
	)

	const virtuosoContext = useMemo(
		() => ({
			accountId: account.id,
			currentMailboxRole: currentMailbox?.role,
			focusedUid,
			zenMode,
			accentColor,
			animationsEnabled,
			previewLines,
			formatDate,
			onMessageClick,
			handleDeleteMessage,
			handleToggleReadStatus,
			handleToggleStar,
			isFetchingNextPage,
			t,
		}),
		[
			account.id,
			currentMailbox?.role,
			focusedUid,
			zenMode,
			accentColor,
			animationsEnabled,
			previewLines,
			formatDate,
			onMessageClick,
			handleDeleteMessage,
			handleToggleReadStatus,
			handleToggleStar,
			isFetchingNextPage,
			t,
		]
	)

	const virtuosoComponents = useMemo(
		() => ({
			Footer: ({ context }: { context?: typeof virtuosoContext }) => {
				if (!context?.isFetchingNextPage) return null

				return (
					<div className='flex items-center justify-center py-4'>
						<motion.div
							{...(context.animationsEnabled
								? {
										initial: { opacity: 0, scale: 0.9 },
										animate: { opacity: 1, scale: 1 },
									}
								: {})}
							className='flex items-center gap-2'>
							<div
								className='h-4 w-4 animate-spin rounded-full border-2 border-transparent'
								style={{ borderTopColor: context.accentColor }}
							/>
							<span className='text-muted-foreground text-xs'>
								{context.t('inbox:messageList.loadingMore')}
							</span>
						</motion.div>
					</div>
				)
			},
		}),
		[]
	)

	const renderItemContent = useCallback(
		(_index: number, message: any, context: typeof virtuosoContext) => {
			const isUnread = !message.flags.includes('\\Seen')
			const isOptimistic = message.uid < 0

			const canDrag =
				context.currentMailboxRole !== 'sent' &&
				context.currentMailboxRole !== 'drafts' &&
				!message.mailbox.startsWith('Virtual_') &&
				!isOptimistic

			const handleDragStart = (e: React.DragEvent<HTMLDivElement>) => {
				if (!canDrag) {
					e.preventDefault()
					return
				}
				const payload: DragMessagePayload = {
					accountId: context.accountId,
					mailbox: message.mailbox,
					uid: message.uid,
					message,
				}
				e.dataTransfer.setData('application/postail-message', JSON.stringify(payload))
				e.dataTransfer.effectAllowed = 'move'
			}

			return (
				<div draggable={canDrag} onDragStart={handleDragStart}>
					<MessageRow
						message={message}
						isUnread={isUnread}
						isFocused={message.uid === context.focusedUid}
						zenMode={context.zenMode}
						accentColor={context.accentColor}
						animationsEnabled={context.animationsEnabled}
						previewLines={context.previewLines}
						formatDate={context.formatDate}
						onMessageClick={context.onMessageClick}
						onDelete={context.handleDeleteMessage}
						onToggleRead={context.handleToggleReadStatus}
						onToggleStar={context.handleToggleStar}
					/>
				</div>
			)
		},
		[]
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
						<span className='text-foreground/80 text-sm font-medium'>
							Syncing messages...
						</span>
						<span className='text-tertiary text-xs'>
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
					<p className='text-tertiary max-w-xs text-center text-xs'>{syncError}</p>
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
						className='text-foreground/80 mt-1 rounded-lg px-4 py-1.5 text-xs font-medium ring-1 ring-[var(--border-subtle)] transition-colors hover:bg-[var(--surface-hover)]'>
						Retry
					</button>
				</motion.div>
			</div>
		)
	}

	if (isLoading) {
		return (
			<div className='flex h-full flex-col overflow-hidden'>
				{[...Array(15)].map((_, i) => (
					<div
						key={i}
						className={`relative flex shrink-0 border-b border-[var(--border-faint)] bg-transparent px-4 py-3 ${
							previewLines === 1 ? 'items-center' : 'items-start'
						}`}>
						<div className='skeleton-shimmer' />
						<div className='flex shrink-0 items-center gap-2.5 pr-3'>
							<div className='h-[15px] w-[15px] rounded bg-[var(--surface-active)]' />
							<div className='h-4 w-4 rounded bg-[var(--surface-active)]' />
						</div>

						{previewLines === 1 ? (
							<>
								<div className='flex min-w-0 flex-1 items-center gap-3'>
									<div className='h-4 w-32 shrink-0 rounded bg-[var(--surface-active)]' />
									<div className='flex min-w-0 flex-1 items-baseline gap-1.5'>
										<div className='h-4 w-48 rounded bg-[var(--surface-active)]' />
										<div className='h-3 w-64 rounded bg-[var(--surface-active)] opacity-60' />
									</div>
								</div>
								<div className='ml-3 flex w-24 shrink-0 justify-end'>
									<div className='h-3 w-12 rounded bg-[var(--surface-active)]' />
								</div>
							</>
						) : (
							<div className='flex min-w-0 flex-1 flex-col gap-1.5 py-0.5'>
								<div className='flex items-center justify-between gap-2'>
									<div className='h-4 w-32 rounded bg-[var(--surface-active)]' />
									<div className='ml-2 flex shrink-0 items-center'>
										<div className='h-3 w-12 rounded bg-[var(--surface-active)]' />
									</div>
								</div>
								<div className='h-4 w-[60%] rounded bg-[var(--surface-active)]' />
								<div className='mt-0.5 flex flex-col gap-1.5'>
									<div className='h-3 w-[90%] rounded bg-[var(--surface-active)] opacity-60' />
									{previewLines === 3 && (
										<div className='h-3 w-[75%] rounded bg-[var(--surface-active)] opacity-60' />
									)}
								</div>
							</div>
						)}
					</div>
				))}
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
			<div className='text-muted-foreground flex h-full flex-col items-center justify-center'>
				<motion.div
					{...(animationsEnabled
						? {
								initial: { opacity: 0, scale: 0.9 },
								animate: { opacity: 1, scale: 1 },
								transition: { duration: 0.4, ease: [0.16, 1, 0.3, 1] },
							}
						: {})}
					className='flex flex-col items-center'>
					<div
						className='flex h-24 w-24 items-center justify-center rounded-3xl bg-[var(--surface-panel)] shadow-xl ring-1 ring-[var(--border-subtle)]'
						style={{ boxShadow: `0 8px 32px -8px ${accentColor}33` }}>
						<Mail className='h-10 w-10 opacity-50' style={{ color: accentColor }} />
					</div>
					<p className='text-foreground/80 mt-6 text-sm font-medium'>
						{t('inbox:messageList.empty.title')}
					</p>
					<p className='text-tertiary mt-1.5 text-xs'>
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
					context={virtuosoContext}
					components={virtuosoComponents}
					itemContent={renderItemContent}
				/>
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
			{...(animationsEnabled ? { whileHover: { scale: 1.1 }, whileTap: { scale: 0.9 } } : {})}
			className={`flex h-7 w-7 items-center justify-center rounded-lg transition-all ${
				destructive
					? 'text-muted-foreground hover:bg-red-500/10 hover:text-red-400'
					: 'text-muted-foreground hover:text-foreground hover:bg-[var(--surface-active)]'
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
