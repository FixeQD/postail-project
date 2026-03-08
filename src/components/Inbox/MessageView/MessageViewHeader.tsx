import { ArrowLeft, Reply, ReplyAll, Forward, Trash2, MailOpen, Code2 } from 'lucide-react'
import { motion, type Variants } from 'framer-motion'
import { Tooltip, TooltipTrigger, TooltipContent } from '@/components/ui/tooltip'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import type { MessageViewHeaderProps } from '@/types/components/shared'

export const MessageViewHeader = ({
	onBack,
	onReply,
	onReplyAll,
	onForward,
	onDelete,
	onMarkUnread,
	onViewSource,
	isDeleting = false,
}: MessageViewHeaderProps) => {
	const { t } = useTypedTranslation(['common', 'inbox'])
	const animationsEnabled = useAnimationsEnabled()

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
						? { whileHover: { scale: 1.08 }, whileTap: { scale: 0.88 } }
						: {})}
					className={`flex h-8 w-8 items-center justify-center rounded-lg transition-colors disabled:opacity-40 ${
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
		hidden: { opacity: 0, y: -4 },
		show: { opacity: 1, y: 0, transition: { duration: 0.18, ease: 'easeOut' } },
	}

	return (
		<motion.div
			className='flex shrink-0 flex-col border-b'
			style={{ borderColor: 'var(--border-faint)' }}
			initial={animationsEnabled ? 'hidden' : 'show'}
			animate='show'
			variants={container}>
			{/* Top bar: nav + actions */}
			<div className='group/toolbar flex items-center gap-2 px-3 py-2'>
				{/* Back + prev/next navigation */}
				<motion.div variants={item}>
					<ActionBtn
						icon={<ArrowLeft className='h-4 w-4' />}
						tooltip={t('inbox:messageView.back')}
						onClick={onBack}
					/>
				</motion.div>

				{/* Divider */}
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

				{/* Divider */}
				<motion.div variants={item} className='h-4 w-px bg-[var(--border-subtle)]' />

				{/* Secondary actions */}
				<motion.div variants={item}>
					<ActionBtn
						icon={<MailOpen className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.markUnread')}
						onClick={onMarkUnread}
					/>
				</motion.div>

				{/* Spacer */}
				<div className='flex-1' />

				{/* View source - hidden until hover on toolbar */}
				<motion.div
					variants={item}
					className='opacity-0 transition-opacity duration-150 group-hover/toolbar:opacity-100'>
					<ActionBtn
						icon={<Code2 className='h-4 w-4' />}
						tooltip='View source (EML)'
						onClick={onViewSource}
					/>
				</motion.div>

				{/* Destructive */}
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
