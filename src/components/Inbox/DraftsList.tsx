import { useEffect } from 'react'
import { Virtuoso } from 'react-virtuoso'
import { formatDistanceToNow } from 'date-fns'
import { Trash2, Edit } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useDraftStore } from '@/stores/draftStore'
import type { ComposeDraft } from '@/types/compose'
import { Button } from '@/components/ui/button'

interface DraftsListProps {
	accountId: string
	onDraftClick: (draft: ComposeDraft) => void
}

export const DraftsList = ({ accountId, onDraftClick }: DraftsListProps) => {
	const { t } = useTranslation()
	const { drafts, loadDrafts, deleteDraft } = useDraftStore()

	useEffect(() => {
		loadDrafts(accountId)
	}, [accountId, loadDrafts])

	const handleDelete = async (draftId: string, e: React.MouseEvent) => {
		e.stopPropagation()
		await deleteDraft(draftId)
	}

	return (
		<div className='flex h-full flex-col'>
			<div className='border-b border-slate-800 p-4'>
				<h2 className='text-lg font-semibold text-slate-200'>
					{t('inbox:sidebar.mailboxes.drafts')}
				</h2>
			</div>
			<div className='flex-1'>
				<Virtuoso
					data={drafts}
					itemContent={(_, draft) => (
						<div
							key={draft.id}
							className='flex cursor-pointer items-center border-b border-slate-800 p-4 hover:bg-slate-900'
							onClick={() => onDraftClick(draft)}>
							<div className='flex-1'>
								<div className='flex items-center justify-between'>
									<h3 className='truncate text-sm font-medium text-slate-200'>
										{draft.subject || t('compose.noSubject')}
									</h3>
									<span className='text-xs text-slate-500'>
										{formatDistanceToNow(new Date(draft.updatedAt), {
											addSuffix: true,
										})}
									</span>
								</div>
								<div className='mt-1 text-xs text-slate-400'>
									{t('compose.to')}: {draft.to.map((r) => r.email).join(', ')}
								</div>
								<div className='mt-1 truncate text-xs text-slate-500'>
									{draft.body?.slice(0, 100)}...
								</div>
							</div>
							<div className='ml-4 flex items-center gap-2'>
								<Button
									variant='ghost'
									size='icon'
									className='h-8 w-8 text-slate-400 hover:text-slate-200'
									onClick={(e) => {
										e.stopPropagation()
										onDraftClick(draft)
									}}>
									<Edit className='h-4 w-4' />
								</Button>
								<Button
									variant='ghost'
									size='icon'
									className='h-8 w-8 text-red-400 hover:text-red-300'
									onClick={(e) => handleDelete(draft.id!, e)}>
									<Trash2 className='h-4 w-4' />
								</Button>
							</div>
						</div>
					)}
				/>
			</div>
		</div>
	)
}
