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
		<div className='flex flex-wrap gap-2 border-t border-zinc-900 bg-zinc-950/30 px-4 py-2'>
			{attachments.map((file) => {
				const Icon = getFileIcon(file.contentType)
				return (
					<Tooltip key={file.id}>
						<TooltipTrigger asChild>
							<div className='group flex cursor-default items-center gap-2 rounded-md bg-zinc-800/50 px-2.5 py-1.5 text-sm ring-1 ring-zinc-800 transition-colors hover:bg-zinc-800'>
								<Icon className='h-4 w-4 text-zinc-400' />
								<div className='flex flex-col'>
									<span className='max-w-[150px] truncate font-medium text-zinc-200'>
										{file.filename}
									</span>
									<span className='text-[10px] text-zinc-500'>
										{formatFileSize(file.size, 0)}
									</span>
								</div>
								<button
									onClick={(e) => {
										e.stopPropagation()
										onRemove(file.id)
									}}
									className='ml-1 rounded-full p-0.5 text-zinc-500 opacity-0 transition-all group-hover:opacity-100 hover:bg-zinc-700 hover:text-red-400'>
									<X className='h-3.5 w-3.5' />
								</button>
							</div>
						</TooltipTrigger>
						<TooltipContent
							side='top'
							align='start'
							className='flex max-w-xs min-w-48 flex-col gap-1 border-zinc-800 bg-zinc-900 p-2 text-zinc-300'>
							<div className='flex flex-col gap-0.5'>
								<span className='text-[10px] font-bold tracking-wider text-zinc-500 uppercase'>
									{t('compose.fileInfo.hash')}
								</span>
								<span className='font-mono break-all text-zinc-400'>
									{file.hash}
								</span>
							</div>
							<div className='flex flex-col gap-0.5'>
								<span className='text-[10px] font-bold tracking-wider text-zinc-500 uppercase'>
									{t('compose.fileInfo.type')}
								</span>
								<span className='text-zinc-400'>{file.contentType}</span>
							</div>
							{file.path && (
								<div className='flex flex-col gap-0.5'>
									<span className='text-[10px] font-bold tracking-wider text-zinc-500 uppercase'>
										{t('compose.fileInfo.path')}
									</span>
									<span className='break-all text-zinc-400 italic'>
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
