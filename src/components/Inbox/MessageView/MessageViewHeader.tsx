import { useState, useMemo } from 'react'
import {
	ArrowLeft,
	Reply,
	ReplyAll,
	Forward,
	Trash2,
	MailOpen,
	Code2,
	Star,
	FolderInput,
	Search,
} from 'lucide-react'
import { motion, type Variants } from 'framer-motion'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { Tooltip, TooltipTrigger, TooltipContent } from '@/components/ui/tooltip'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useThemeStore } from '@/stores/themeStore'
import type { MessageViewHeaderProps } from '@/types/components/shared'
import type { Mailbox } from '@/types/mail'

const SYSTEM_ROLE_ORDER = ['inbox', 'sent', 'drafts', 'archive', 'junk', 'trash']

export const MessageViewHeader = ({
	onBack,
	onReply,
	onReplyAll,
	onForward,
	onDelete,
	onMarkUnread,
	onToggleStar,
	onMoveTo,
	onViewSource,
	isDeleting = false,
	isStarred = false,
	accountId,
	currentMailbox,
}: MessageViewHeaderProps) => {
	const { t } = useTypedTranslation(['common', 'inbox'])
	const animationsEnabled = useAnimationsEnabled()
	const accentColor = useThemeStore((s) => s.accentColor)
	const [moveOpen, setMoveOpen] = useState(false)
	const [search, setSearch] = useState('')

	const { data: mailboxes } = useQuery({
		queryKey: ['mailboxes', accountId],
		queryFn: () => invoke<Mailbox[]>('fetch_mailboxes', { accountId }),
		enabled: moveOpen,
		staleTime: 30_000,
	})

	const filteredMailboxes = useMemo(() => {
		if (!mailboxes) return []
		const q = search.toLowerCase()
		return mailboxes
			.filter(
				(m) =>
					!m.name.startsWith('Virtual_') &&
					m.name !== currentMailbox &&
					m.display_name.toLowerCase().includes(q)
			)
			.sort((a, b) => {
				const ia = SYSTEM_ROLE_ORDER.indexOf(a.role)
				const ib = SYSTEM_ROLE_ORDER.indexOf(b.role)
				if (ia !== -1 && ib !== -1) return ia - ib
				if (ia !== -1) return -1
				if (ib !== -1) return 1
				return a.display_name.localeCompare(b.display_name)
			})
	}, [mailboxes, currentMailbox, search])

	const handleMove = (targetName: string) => {
		setMoveOpen(false)
		setSearch('')
		onMoveTo(targetName)
	}

	const ActionBtn = ({
		icon,
		tooltip,
		destructive,
		onClick,
		disabled,
		active,
	}: {
		icon: React.ReactNode
		tooltip: string
		destructive?: boolean
		onClick?: () => void
		disabled?: boolean
		active?: boolean
	}) => (
		<Tooltip>
			<TooltipTrigger asChild>
				<motion.button
					type='button'
					{...(animationsEnabled
						? { whileHover: { scale: 1.05 }, whileTap: { scale: 0.9 } }
						: {})}
					className={`flex h-8 w-8 items-center justify-center rounded-lg transition-all disabled:opacity-40 ${
						active
							? 'bg-sky-500/15 text-sky-400'
							: destructive
								? 'text-muted-foreground hover:bg-red-500/10 hover:text-red-400'
								: 'text-muted-foreground hover:text-foreground hover:bg-[var(--surface-hover)]'
					}`}
					onClick={onClick}
					disabled={disabled}>
					{icon}
				</motion.button>
			</TooltipTrigger>
			<TooltipContent sideOffset={6}>{tooltip}</TooltipContent>
		</Tooltip>
	)

	const container: Variants = {
		hidden: { opacity: 0 },
		show: { opacity: 1, transition: { staggerChildren: 0.04, delayChildren: 0.05 } },
	}
	const item: Variants = {
		hidden: { opacity: 0, y: -8, scale: 0.96 },
		show: {
			opacity: 1,
			y: 0,
			scale: 1,
			transition: { duration: 0.2, ease: [0.16, 1, 0.3, 1] },
		},
	}

	return (
		<motion.div
			className='flex shrink-0 flex-col border-b'
			style={{ borderColor: 'var(--border-faint)' }}
			initial={animationsEnabled ? 'hidden' : 'show'}
			animate='show'
			variants={container}>
			<div className='group/toolbar flex items-center gap-2 px-3 py-2'>
				{/* Back */}
				<motion.div variants={item}>
					<ActionBtn
						icon={<ArrowLeft className='h-4 w-4' />}
						tooltip={t('inbox:messageView.back')}
						onClick={onBack}
					/>
				</motion.div>

				<motion.div variants={item} className='h-4 w-px bg-[var(--border-subtle)]' />

				{/* Reply actions */}
				<motion.div variants={item} className='flex items-center gap-0.5'>
					<ActionBtn
						icon={<Reply className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.reply')}
						onClick={onReply}
					/>
					<ActionBtn
						icon={<ReplyAll className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.replyAll')}
						onClick={onReplyAll}
					/>
					<ActionBtn
						icon={<Forward className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.forward')}
						onClick={onForward}
					/>
				</motion.div>

				<motion.div variants={item} className='h-4 w-px bg-[var(--border-subtle)]' />

				{/* Secondary actions */}
				<motion.div variants={item} className='flex items-center gap-0.5'>
					<ActionBtn
						icon={<MailOpen className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.markUnread')}
						onClick={onMarkUnread}
					/>

					{/* Star */}
					<Tooltip>
						<TooltipTrigger asChild>
							<motion.button
								type='button'
								{...(animationsEnabled
									? { whileHover: { scale: 1.05 }, whileTap: { scale: 0.75 } }
									: {})}
								className={`flex h-8 w-8 items-center justify-center rounded-lg transition-all ${
									isStarred
										? 'bg-amber-400/10 text-amber-400 hover:text-amber-300'
										: 'text-muted-foreground hover:bg-[var(--surface-hover)] hover:text-amber-400'
								}`}
								onClick={onToggleStar}
								aria-label={
									isStarred
										? t('inbox:messageView.actions.unstar')
										: t('inbox:messageView.actions.star')
								}
								aria-pressed={isStarred}>
								<motion.div
									{...(animationsEnabled
										? {
												animate: isStarred
													? {
															scale: [1, 1.4, 1],
															rotate: [0, 20, -10, 0],
														}
													: { scale: 1, rotate: 0 },
												transition: { duration: 0.35, ease: 'easeOut' },
											}
										: {})}>
									<Star
										className='h-4 w-4'
										fill={isStarred ? 'currentColor' : 'none'}
									/>
								</motion.div>
							</motion.button>
						</TooltipTrigger>
						<TooltipContent sideOffset={6}>
							{isStarred
								? t('inbox:messageView.actions.unstar')
								: t('inbox:messageView.actions.star')}
						</TooltipContent>
					</Tooltip>

					{/* Move to... */}
					<Popover
						open={moveOpen}
						onOpenChange={(o) => {
							setMoveOpen(o)
							if (!o) setSearch('')
						}}>
						<Tooltip>
							<TooltipTrigger asChild>
								<PopoverTrigger asChild>
									<motion.button
										type='button'
										{...(animationsEnabled
											? {
													whileHover: { scale: 1.05 },
													whileTap: { scale: 0.9 },
												}
											: {})}
										className={`flex h-8 w-8 items-center justify-center rounded-lg transition-all ${
											moveOpen
												? 'bg-[var(--surface-active)] text-[var(--text-primary)]'
												: 'text-muted-foreground hover:text-foreground hover:bg-[var(--surface-hover)]'
										}`}
										aria-label={t('inbox:messageView.actions.moveTo')}>
										<FolderInput className='h-4 w-4' />
									</motion.button>
								</PopoverTrigger>
							</TooltipTrigger>
							<TooltipContent sideOffset={6}>
								{t('inbox:messageView.actions.moveTo')}
							</TooltipContent>
						</Tooltip>

						<PopoverContent
							align='start'
							sideOffset={6}
							className='w-56 overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-glass)] p-0 shadow-xl backdrop-blur-xl'>
							{/* Search */}
							<div className='flex items-center gap-2 border-b border-[var(--border-faint)] px-3 py-2'>
								<Search className='h-3.5 w-3.5 shrink-0 text-[var(--text-tertiary)]' />
								<input
									autoFocus
									value={search}
									onChange={(e) => setSearch(e.target.value)}
									placeholder={t('inbox:messageView.actions.moveToPlaceholder')}
									className='flex-1 bg-transparent text-xs text-[var(--text-primary)] outline-none placeholder:text-[var(--text-tertiary)]'
								/>
							</div>

							{/* Folder list */}
							<div className='max-h-52 overflow-y-auto py-1'>
								{filteredMailboxes.length === 0 ? (
									<p className='px-3 py-4 text-center text-xs text-[var(--text-tertiary)]'>
										{t('inbox:messageView.actions.moveToEmpty')}
									</p>
								) : (
									filteredMailboxes.map((mb) => (
										<button
											key={mb.name}
											type='button'
											onClick={() => handleMove(mb.name)}
											className='flex w-full items-center gap-2.5 px-3 py-2 text-left text-xs text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
											<span className='truncate'>{mb.display_name}</span>
										</button>
									))
								)}
							</div>
						</PopoverContent>
					</Popover>
				</motion.div>

				<div className='flex-1' />

				{/* View source - hidden until toolbar hover */}
				<motion.div
					variants={item}
					className='opacity-0 transition-opacity duration-150 group-hover/toolbar:opacity-100'>
					<ActionBtn
						icon={<Code2 className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.viewSource')}
						onClick={onViewSource}
					/>
				</motion.div>

				{/* Delete */}
				<motion.div variants={item}>
					<ActionBtn
						icon={<Trash2 className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.delete')}
						destructive
						onClick={onDelete}
						disabled={isDeleting}
					/>
				</motion.div>
			</div>
		</motion.div>
	)
}
