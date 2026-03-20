import { useState } from 'react'
import { motion } from 'framer-motion'
import { ChevronDown, ChevronUp } from 'lucide-react'
import type { ThreadView as ThreadViewType } from '@/types/mail'
import { MessageViewMeta } from './MessageViewMeta'
import { MessageViewBody } from './MessageViewBody'
import { useAnimationsEnabled } from '@/hooks/useMotion'

interface ThreadViewProps {
	thread: ThreadViewType
	currentUid: number
	onReply?: () => void
	onReplyAll?: () => void
	onForward?: () => void
	blockExternalImages: boolean
	viewMode: 'plain' | 'html'
}

export const ThreadView = ({
	thread,
	currentUid,
	blockExternalImages,
	viewMode,
}: ThreadViewProps) => {
	const animationsEnabled = useAnimationsEnabled()
	const [expandedUids, setExpandedUids] = useState<Set<number>>(new Set([currentUid]))

	const toggleExpanded = (uid: number) => {
		const next = new Set(expandedUids)
		if (next.has(uid)) {
			next.delete(uid)
		} else {
			next.add(uid)
		}
		setExpandedUids(next)
	}

	if (!thread.messages || thread.messages.length <= 1) {
		return null
	}

	return (
		<div className='thread-view flex flex-col gap-3'>
			{thread.messages.map((msg, idx) => {
				const isExpanded = expandedUids.has(msg.header.uid)
				const isCurrent = msg.is_current

				return (
					<motion.div
						key={`${msg.header.uid}-${idx}`}
						className={`rounded-lg border transition-colors ${
							isCurrent
								? 'border-[var(--border-active)] bg-[var(--surface-active)]'
								: 'border-[var(--border-faint)] bg-[var(--surface-secondary)]'
						}`}
						{...(animationsEnabled && {
							initial: { opacity: 0, y: 10 },
							animate: { opacity: 1, y: 0 },
							transition: { delay: idx * 0.05 },
						})}>
						{/* Thread message header - collapse/expand */}
						<button
							type='button'
							onClick={() => toggleExpanded(msg.header.uid)}
							className='w-full px-4 py-3 text-left transition-colors hover:bg-[var(--surface-hover)]'>
							<div className='flex items-center justify-between gap-3'>
								<div className='flex min-w-0 flex-1 items-center gap-3'>
									<div
										className={`shrink-0 transition-transform ${isExpanded ? '' : ''}`}>
										{isExpanded ? (
											<ChevronUp className='h-4 w-4 text-[var(--text-secondary)]' />
										) : (
											<ChevronDown className='h-4 w-4 text-[var(--text-secondary)]' />
										)}
									</div>
									<div className='min-w-0 flex-1'>
										<p className='truncate text-sm font-medium text-[var(--text-primary)]'>
											{msg.header.from[0] || 'Unknown'}
										</p>
										<p className='text-xs text-[var(--text-tertiary)]'>
											{new Date(msg.header.internal_date).toLocaleString()}
										</p>
									</div>
								</div>
								{msg.is_current && (
									<span className='shrink-0 rounded-full bg-[var(--accent-color)]/20 px-2 py-1 text-[10px] font-semibold tracking-tight text-[var(--accent-color)] uppercase'>
										Current
									</span>
								)}
							</div>
						</button>

						{/* Message content - expandable */}
						{isExpanded && (
							<motion.div
								{...(animationsEnabled && {
									initial: { opacity: 0, height: 0 },
									animate: { opacity: 1, height: 'auto' },
									transition: { duration: 0.2 },
								})}
								className='border-t border-[var(--border-faint)]'>
								<MessageViewMeta header={msg.header} />

								<div className='px-5 py-4'>
									<MessageViewBody
										htmlContent={msg.body_html_safe}
										plainContent={msg.body_plain}
										viewMode={viewMode}
										allowExternalResources={!blockExternalImages}
										inline_images={[]}
										onExternalDetected={() => {}}
									/>
								</div>
							</motion.div>
						)}
					</motion.div>
				)
			})}
		</div>
	)
}
