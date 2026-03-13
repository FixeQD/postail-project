import { useEffect, type MouseEvent } from 'react'
import { Virtuoso } from 'react-virtuoso'
import { formatDistanceToNow } from 'date-fns'
import { Trash2, Edit, FileText } from 'lucide-react'
import { motion } from 'framer-motion'
import { useTranslation } from 'react-i18next'
import { useDraftStore } from '@/stores/draftStore'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { Button } from '@/components/ui/button'
import type { DraftsListProps } from '@/types/components/shared'

export const DraftsList = ({ accountId, onDraftClick }: DraftsListProps) => {
	const { t } = useTranslation()
	const { drafts, loadDrafts, deleteDraft } = useDraftStore()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()

	useEffect(() => {
		const controller = new AbortController()
		const current = accountId
		loadDrafts(accountId, controller.signal).then((responseAccountId) => {
			if (responseAccountId !== current) return
		})
		return () => {
			controller.abort()
		}
	}, [accountId, loadDrafts])

	const handleDelete = async (draftId: string, e: MouseEvent) => {
		e.stopPropagation()
		await deleteDraft(draftId)
	}

	return (
		<div className='flex h-full flex-col'>
			{/* Header */}
			<motion.div
				{...(animationsEnabled
					? {
							initial: { opacity: 0, y: -8 },
							animate: { opacity: 1, y: 0 },
							transition: { duration: 0.3, ease: [0.16, 1, 0.3, 1] },
						}
					: {})}
				className='relative border-b px-5 py-4'
				style={{ borderColor: 'var(--border-faint)' }}>
				<h2 className='text-foreground text-sm font-semibold tracking-wide'>
					{t('inbox:sidebar.mailboxes.drafts')}
				</h2>
				{drafts.length > 0 && (
					<span
						className='ml-2 inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-semibold ring-1'
						style={{
							backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
							color: accentColor,
							boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
						}}>
						{drafts.length}
					</span>
				)}
				<div className='pointer-events-none absolute inset-x-0 bottom-0 h-px bg-gradient-to-r from-transparent via-black/[0.04] to-transparent dark:via-white/[0.04]' />
			</motion.div>

			{/* Empty state */}
			{drafts.length === 0 && (
				<motion.div
					{...(animationsEnabled
						? {
								initial: { opacity: 0, scale: 0.95 },
								animate: { opacity: 1, scale: 1 },
								transition: { duration: 0.4, ease: [0.16, 1, 0.3, 1] },
							}
						: {})}
					className='flex flex-1 flex-col items-center justify-center'>
					<div
						className='flex h-24 w-24 items-center justify-center rounded-3xl bg-[var(--surface-panel)] shadow-xl ring-1 ring-[var(--border-subtle)]'
						style={{ boxShadow: `0 8px 32px -8px ${accentColor}33` }}>
						<FileText className='h-10 w-10 opacity-50' style={{ color: accentColor }} />
					</div>
					<p className='text-foreground/80 mt-6 text-sm font-medium'>No drafts</p>
					<p className='text-tertiary mt-1.5 text-xs'>
						Your saved drafts will appear here
					</p>
				</motion.div>
			)}

			{/* List */}
			{drafts.length > 0 && (
				<div className='flex-1'>
					<Virtuoso
						data={drafts}
						itemContent={(index, draft) => {
							if (!draft.id) return null
							return (
								<motion.div
									key={draft.id}
									{...(animationsEnabled
										? {
												initial: { opacity: 0, y: 8 },
												animate: { opacity: 1, y: 0 },
												transition: {
													delay: Math.min(index * 0.04, 0.3),
													duration: 0.3,
													ease: [0.16, 1, 0.3, 1],
												},
											}
										: {})}
									onClick={() => onDraftClick(draft)}
									className='group relative flex cursor-pointer items-center border-b px-5 py-3.5 transition-all duration-150 hover:bg-[var(--surface-panel)]'
									style={{ borderColor: 'var(--border-faint)' }}>
									{/* Left accent line on hover */}
									<div
										className='absolute top-1/2 left-0 h-0 w-[3px] -translate-y-1/2 rounded-r-full transition-all duration-300 group-hover:h-[60%]'
										style={{
											backgroundColor: accentColor,
											boxShadow: `1px 0 8px ${accentColor}80`,
										}}
									/>

									{/* Draft icon */}
									<div
										className='mr-3.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-xl ring-1 transition-all duration-200'
										style={{
											backgroundColor: `rgba(var(--accent-rgb), 0.08)`,
											boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.12)`,
										}}>
										<Edit
											className='h-4 w-4'
											style={{ color: `rgba(var(--accent-rgb), 0.8)` }}
										/>
									</div>

									{/* Content */}
									<div className='min-w-0 flex-1'>
										<div className='flex items-center justify-between gap-3'>
											<h3 className='text-foreground/80 group-hover:text-foreground truncate text-[13px] font-medium transition-colors'>
												{draft.subject || t('compose.noSubject')}
											</h3>
											<span className='text-tertiary shrink-0 text-xs tabular-nums'>
												{formatDistanceToNow(new Date(draft.updatedAt), {
													addSuffix: true,
												})}
											</span>
										</div>
										<div className='text-muted-foreground mt-1 text-xs'>
											{t('compose.to')}:{' '}
											<span className='text-foreground/70'>
												{draft.to.length > 0
													? draft.to.map((r) => r.email).join(', ')
													: '-'}
											</span>
										</div>
										{draft.body && (
											<div className='text-tertiary mt-1 truncate text-xs'>
												{draft.body.slice(0, 120)}
											</div>
										)}
									</div>

									{/* Actions - visible on hover */}
									<div className='ml-3 flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100'>
										<motion.div
											{...(animationsEnabled
												? {
														whileHover: { scale: 1.1 },
														whileTap: { scale: 0.9 },
													}
												: {})}>
											<Button
												variant='ghost'
												size='icon'
												className='text-muted-foreground hover:text-foreground h-7 w-7 transition-all hover:bg-[var(--surface-active)]'
												onClick={(e) => {
													e.stopPropagation()
													onDraftClick(draft)
												}}>
												<Edit className='h-[15px] w-[15px]' />
											</Button>
										</motion.div>
										<motion.div
											{...(animationsEnabled
												? {
														whileHover: { scale: 1.1 },
														whileTap: { scale: 0.9 },
													}
												: {})}>
											<Button
												variant='ghost'
												size='icon'
												className='text-muted-foreground h-7 w-7 transition-all hover:bg-red-500/10 hover:text-red-400'
												onClick={(e) => handleDelete(draft.id!, e)}>
												<Trash2 className='h-[15px] w-[15px]' />
											</Button>
										</motion.div>
									</div>
								</motion.div>
							)
						}}
					/>
				</div>
			)}
		</div>
	)
}
