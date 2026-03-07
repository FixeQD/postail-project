import { X } from 'lucide-react'
import { memo } from 'react'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { useTranslation } from 'react-i18next'
import { getFileIcon } from '@/lib/fileUtils'
import { formatFileSize } from '@/lib/formatFileSize'
import type { AttachmentListProps } from '@/types/components/compose'

export const AttachmentList = memo(({ attachments, onRemove }: AttachmentListProps) => {
	const { t } = useTranslation()
	if (attachments.length === 0) return null

	return (
		<div className='flex flex-wrap gap-2 border-t border-[var(--compose-input-border)] bg-[var(--compose-footer-bg)] px-4 py-2'>
			{attachments.map((file) => {
				const Icon = getFileIcon(file.contentType)
				return (
					<Tooltip key={file.id}>
						<TooltipTrigger asChild>
							<div className='group flex cursor-default items-center gap-2 rounded-md bg-[var(--compose-chip-bg)] px-2.5 py-1.5 text-sm ring-1 ring-[var(--compose-chip-border)] transition-colors hover:bg-[var(--compose-active)]'>
								<Icon className='h-4 w-4 text-[var(--compose-text-muted)]' />
								<div className='flex flex-col'>
									<span className='max-w-[150px] truncate font-medium text-[var(--compose-text)]'>
										{file.filename}
									</span>
									<span className='text-[10px] text-[var(--compose-placeholder)]'>
										{formatFileSize(file.size, 0)}
									</span>
								</div>
								<button
									onClick={(e) => {
										e.stopPropagation()
										onRemove(file.id)
									}}
									className='ml-1 rounded-full p-0.5 text-[var(--compose-text-muted)] opacity-0 transition-all group-hover:opacity-100 hover:bg-[var(--compose-hover)] hover:text-red-400'>
									<X className='h-3.5 w-3.5' />
								</button>
							</div>
						</TooltipTrigger>
						<TooltipContent
							side='top'
							align='start'
							className='flex max-w-xs min-w-48 flex-col gap-1 border-[var(--compose-ring)] bg-[var(--compose-suggestions-bg)] p-2 text-[var(--compose-text)]'>
							<div className='flex flex-col gap-0.5'>
								<span className='text-[10px] font-bold tracking-wider text-[var(--compose-placeholder)] uppercase'>
									{t('compose.fileInfo.hash')}
								</span>
								<span className='font-mono break-all text-[var(--compose-text-muted)]'>
									{file.hash}
								</span>
							</div>
							<div className='flex flex-col gap-0.5'>
								<span className='text-[10px] font-bold tracking-wider text-[var(--compose-placeholder)] uppercase'>
									{t('compose.fileInfo.type')}
								</span>
								<span className='text-[var(--compose-text-muted)]'>
									{file.contentType}
								</span>
							</div>
							{file.path && (
								<div className='flex flex-col gap-0.5'>
									<span className='text-[10px] font-bold tracking-wider text-[var(--compose-placeholder)] uppercase'>
										{t('compose.fileInfo.path')}
									</span>
									<span className='break-all text-[var(--compose-text-muted)] italic'>
										{file.path}
									</span>
								</div>
							)}
						</TooltipContent>
					</Tooltip>
				)
			})}
		</div>
	)
})
