import { useEffect, type MouseEvent } from 'react'
import { Virtuoso } from 'react-virtuoso'
import { formatDistanceToNow } from 'date-fns'
import { Trash2, Edit, FileText } from 'lucide-react'
import { motion } from 'framer-motion'
import { useTranslation } from 'react-i18next'
import { useDraftStore } from '@/stores/draftStore'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import type { ComposeDraft } from '@/types/compose'
import { Button } from '@/components/ui/button'

interface DraftsListProps {
	accountId: string
	onDraftClick: (draft: ComposeDraft) => void
}

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
				className='relative border-b border-white/[0.04] px-5 py-4'>
				<h2 className='text-sm font-semibold tracking-wide text-slate-200'>
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
				<div className='pointer-events-none absolute inset-x-0 bottom-0 h-px bg-gradient-to-r from-transparent via-white/[0.04] to-transparent' />
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
					<div className='flex h-20 w-20 items-center justify-center rounded-2xl bg-slate-900/50 ring-1 ring-white/[0.06]'>
						<FileText className='h-8 w-8 text-slate-700' />
					</div>
					<p className='mt-4 text-sm font-medium text-slate-400'>No drafts</p>
					<p className='mt-1 text-xs text-slate-600'>
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
									className='group relative flex cursor-pointer items-center border-b border-white/[0.04] px-5 py-3.5 transition-all duration-150 hover:bg-white/[0.03]'>
									{/* Left accent line on hover */}
									<div
										className='absolute top-1/2 left-0 h-0 w-[3px] -translate-y-1/2 rounded-r-full transition-all duration-200 group-hover:h-[60%]'
										style={{ backgroundColor: accentColor }}
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
											<h3 className='truncate text-[13px] font-medium text-slate-200 transition-colors group-hover:text-white'>
												{draft.subject || t('compose.noSubject')}
											</h3>
											<span className='shrink-0 text-xs text-slate-600 tabular-nums'>
												{formatDistanceToNow(new Date(draft.updatedAt), {
													addSuffix: true,
												})}
											</span>
										</div>
										<div className='mt-1 text-xs text-slate-500'>
											{t('compose.to')}:{' '}
											<span className='text-slate-400'>
												{draft.to.length > 0
													? draft.to.map((r) => r.email).join(', ')
													: '-'}
											</span>
										</div>
										{draft.body && (
											<div className='mt-1 truncate text-xs text-slate-600'>
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
														whileTap: { scale: 0.85 },
													}
												: {})}>
											<Button
												variant='ghost'
												size='icon'
												className='h-7 w-7 text-slate-500 hover:bg-white/[0.08] hover:text-slate-200'
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
														whileTap: { scale: 0.85 },
													}
												: {})}>
											<Button
												variant='ghost'
												size='icon'
												className='h-7 w-7 text-slate-500 hover:bg-red-500/10 hover:text-red-400'
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
