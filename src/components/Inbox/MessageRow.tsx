import { memo } from 'react'
import { motion } from 'framer-motion'
import { Star, Trash2, MailOpen, Mail, Loader2 } from 'lucide-react'
import type { MessageRowProps } from '@/types/components/inbox'

const ActionBtn = ({
	icon,
	tooltip,
	destructive,
	onClick,
	animationsEnabled,
}: {
	icon: React.ReactNode
	tooltip: string
	destructive?: boolean
	onClick?: (e: React.MouseEvent) => void
	animationsEnabled: boolean
}) => {
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

const DateOrActions = memo(
	({
		isUnread,
		zenMode,
		animationsEnabled,
		formattedDate,
		onDelete,
		onToggleRead,
		t,
		isOptimistic,
	}: {
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

		const dateClass = `text-xs tabular-nums ${animationsEnabled ? 'transition-opacity duration-150 ease-out group-hover:opacity-0' : 'group-hover:hidden'} ${
			isUnread && !zenMode ? 'text-foreground/80 font-medium' : 'text-tertiary'
		}`

		return (
			<>
				<span className={dateClass}>{formattedDate}</span>
				<div
					className={`absolute right-0 flex items-center gap-0.5 ${
						animationsEnabled
							? 'pointer-events-none translate-x-2 opacity-0 transition-all duration-150 ease-out group-hover:pointer-events-auto group-hover:translate-x-0 group-hover:opacity-100'
							: 'hidden group-hover:flex'
					}`}>
					<ActionBtn
						icon={<Trash2 className='h-[15px] w-[15px]' />}
						tooltip={t('inbox:messageList.actions.delete')}
						destructive
						onClick={onDelete}
						animationsEnabled={animationsEnabled}
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
						animationsEnabled={animationsEnabled}
					/>
				</div>
			</>
		)
	}
)

export const MessageRow = memo(
	({
		message,
		isUnread,
		zenMode,
		accentColor,
		animationsEnabled,
		previewLines,
		formatDate,
		t,
		onMessageClick,
		onDelete,
		onToggleRead,
		onToggleStar,
		isFocused,
	}: MessageRowProps) => {
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

		const rowBase = `message-unread-indicator group relative flex w-full cursor-pointer select-none border-b text-left transition-colors duration-150 outline-none ${
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
					className={`${rowBase} items-center px-4 py-3`}
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

					<div className='relative ml-3 flex w-24 shrink-0 items-center justify-end'>
						<DateOrActions
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
				className={`${rowBase} items-start px-4 py-3`}
				style={{ borderColor: 'var(--border-faint)', ...focusedStyle }}>
				{activeIndicator}
				{checkboxStar}

				<div className='flex min-w-0 flex-1 flex-col gap-0.5'>
					{/* Row 1: sender + date */}
					<div className='flex items-center justify-between gap-2'>
						<span className={`${senderClass} min-w-0`}>{sender}</span>
						<div className='relative ml-2 flex shrink-0 items-center'>
							<DateOrActions
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
