import { ArrowLeft, Reply, ReplyAll, Forward, Trash2, MailOpen, Code, FileText } from 'lucide-react'
import { motion, type Variants } from 'framer-motion'
import { Tooltip, TooltipTrigger, TooltipContent } from '@/components/ui/tooltip'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'

interface MessageViewHeaderProps {
	onBack: () => void
	onReply: () => void
	onReplyAll: () => void
	onForward: () => void
	onDelete: () => void
	onMarkUnread: () => void
	viewMode: 'html' | 'plain'
	onToggleViewMode: () => void
	hasHtml?: boolean
	isDeleting?: boolean
}

export const MessageViewHeader = ({
	onBack,
	onReply,
	onReplyAll,
	onForward,
	onDelete,
	onMarkUnread,
	viewMode,
	onToggleViewMode,
	hasHtml = true,
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
	}: {
		icon: React.ReactNode
		tooltip: string
		destructive?: boolean
		onClick?: () => void
		disabled?: boolean
	}) => (
		<Tooltip>
			<TooltipTrigger asChild>
				<motion.button
					type='button'
					{...(animationsEnabled
						? { whileHover: { scale: 1.1 }, whileTap: { scale: 0.85 } }
						: {})}
					className={`flex h-8 w-8 items-center justify-center rounded-lg transition-colors disabled:opacity-50 ${
						destructive
							? 'text-slate-400 hover:bg-red-500/10 hover:text-red-400'
							: 'text-slate-400 hover:bg-white/[0.08] hover:text-slate-200'
					}`}
					onClick={onClick}
					disabled={disabled}>
					{icon}
				</motion.button>
			</TooltipTrigger>
			<TooltipContent sideOffset={5}>
				{tooltip}
			</TooltipContent>
		</Tooltip>
	)

	const container: Variants = {
		hidden: { opacity: 0 },
		show: {
			opacity: 1,
			transition: {
				staggerChildren: 0.05,
				delayChildren: 0.1,
			},
		},
	}

	const item: Variants = {
		hidden: { opacity: 0, scale: 0.8 },
		show: {
			opacity: 1,
			scale: 1,
			transition: { type: 'spring', stiffness: 300, damping: 24 },
		},
	}

	return (
		<motion.div
			className='flex items-center justify-between border-b border-white/[0.04] bg-transparent px-4 py-2'
			initial={animationsEnabled ? 'hidden' : 'show'}
			animate='show'
			variants={container}>
			{/* Left side - Back button */}
			<div className='flex items-center gap-2'>
				<motion.div variants={item}>
					<ActionBtn
						icon={<ArrowLeft className='h-4 w-4' />}
						tooltip={t('inbox:messageView.back')}
						onClick={onBack}
					/>
				</motion.div>
			</div>

			{/* Center - View mode toggle */}
			{hasHtml && (
				<motion.div
					variants={item}
					className='flex items-center gap-1 rounded-lg bg-white/[0.04] p-1'>
					<button
						type='button'
						onClick={() => viewMode !== 'html' && onToggleViewMode()}
						className={`relative z-10 flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors ${
							viewMode === 'html' ? 'text-slate-200' : 'text-slate-500 hover:text-slate-300'
						}`}>
						{viewMode === 'html' && animationsEnabled && (
							<motion.div
								layoutId='viewModeParams'
								className='absolute inset-0 z-[-1] rounded-md bg-white/[0.08]'
								transition={{ type: 'spring', bounce: 0.2, duration: 0.6 }}
							/>
						)}
						{/* Fallback background for when animations are disabled */}
						{viewMode === 'html' && !animationsEnabled && (
							<div className='absolute inset-0 z-[-1] rounded-md bg-white/[0.08]' />
						)}
						<Code className='h-3.5 w-3.5' />
						{t('inbox:messageView.viewMode.html')}
					</button>
					<button
						type='button'
						onClick={() => viewMode !== 'plain' && onToggleViewMode()}
						className={`relative z-10 flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors ${
							viewMode === 'plain' ? 'text-slate-200' : 'text-slate-500 hover:text-slate-300'
						}`}>
						{viewMode === 'plain' && animationsEnabled && (
							<motion.div
								layoutId='viewModeParams'
								className='absolute inset-0 z-[-1] rounded-md bg-white/[0.08]'
								transition={{ type: 'spring', bounce: 0.2, duration: 0.6 }}
							/>
						)}
						{/* Fallback background for when animations are disabled */}
						{viewMode === 'plain' && !animationsEnabled && (
							<div className='absolute inset-0 z-[-1] rounded-md bg-white/[0.08]' />
						)}
						<FileText className='h-3.5 w-3.5' />
						{t('inbox:messageView.viewMode.plain')}
					</button>
				</motion.div>
			)}

			{/* Right side - Action buttons */}
			<motion.div variants={container} className='flex items-center gap-0.5'>
				<motion.div variants={item}>
					<ActionBtn
						icon={<Reply className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.reply')}
						onClick={onReply}
					/>
				</motion.div>
				<motion.div variants={item}>
					<ActionBtn
						icon={<ReplyAll className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.replyAll')}
						onClick={onReplyAll}
					/>
				</motion.div>
				<motion.div variants={item}>
					<ActionBtn
						icon={<Forward className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.forward')}
						onClick={onForward}
					/>
				</motion.div>

				<motion.div variants={item} className='mx-1 h-4 w-px bg-white/[0.08]' />

				<motion.div variants={item}>
					<ActionBtn
						icon={<Trash2 className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.delete')}
						destructive
						onClick={onDelete}
						disabled={isDeleting}
					/>
				</motion.div>
				<motion.div variants={item}>
					<ActionBtn
						icon={<MailOpen className='h-4 w-4' />}
						tooltip={t('inbox:messageView.actions.markUnread')}
						onClick={onMarkUnread}
					/>
				</motion.div>
			</motion.div>
		</motion.div>
	)
}
