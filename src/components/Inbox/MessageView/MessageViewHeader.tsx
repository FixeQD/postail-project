import { ArrowLeft, Reply, ReplyAll, Forward, Trash2, MailOpen, Code, FileText } from 'lucide-react'
import { motion } from 'framer-motion'
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

	return (
		<div className='flex items-center justify-between border-b border-white/[0.04] bg-transparent px-4 py-2'>
			{/* Left side - Back button */}
			<div className='flex items-center gap-2'>
				<ActionBtn
					icon={<ArrowLeft className='h-4 w-4' />}
					tooltip={t('inbox:messageView.back')}
					onClick={onBack}
				/>
			</div>

			{/* Center - View mode toggle */}
			{hasHtml && (
				<div className='flex items-center gap-1 rounded-lg bg-white/[0.04] p-1'>
					<button
						type='button'
						onClick={() => viewMode !== 'html' && onToggleViewMode()}
						className={`flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors ${
							viewMode === 'html'
								? 'bg-white/[0.08] text-slate-200'
								: 'text-slate-500 hover:text-slate-300'
						}`}>
						<Code className='h-3.5 w-3.5' />
						{t('inbox:messageView.viewMode.html')}
					</button>
					<button
						type='button'
						onClick={() => viewMode !== 'plain' && onToggleViewMode()}
						className={`flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium transition-colors ${
							viewMode === 'plain'
								? 'bg-white/[0.08] text-slate-200'
								: 'text-slate-500 hover:text-slate-300'
						}`}>
						<FileText className='h-3.5 w-3.5' />
						{t('inbox:messageView.viewMode.plain')}
					</button>
				</div>
			)}

			{/* Right side - Action buttons */}
			<div className='flex items-center gap-0.5'>
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

				<div className='mx-1 h-4 w-px bg-white/[0.08]' />

				<ActionBtn
					icon={<Trash2 className='h-4 w-4' />}
					tooltip={t('inbox:messageView.actions.delete')}
					destructive
					onClick={onDelete}
					disabled={isDeleting}
				/>
				<ActionBtn
					icon={<MailOpen className='h-4 w-4' />}
					tooltip={t('inbox:messageView.actions.markUnread')}
					onClick={onMarkUnread}
				/>
			</div>
		</div>
	)
}
