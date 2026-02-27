import {
	FileIcon,
	FileImageIcon,
	FileTextIcon,
	FileArchiveIcon,
	DownloadIcon,
	PaperclipIcon,
} from 'lucide-react'
import { formatFileSize } from '@/lib/formatFileSize'
import type { AttachmentMeta } from '@/types/mail'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { motion, type Variants } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { toast } from '@/stores/toastStore'
import { useState } from 'react'
import type { MessageViewAttachmentsProps } from '@/types/components/shared'

export const MessageViewAttachments = ({
	attachments,
	accountId,
	mailbox,
	uid,
}: MessageViewAttachmentsProps) => {
	const { t } = useTypedTranslation(['inbox'])
	const [downloading, setDownloading] = useState<string | null>(null)

	const handleDownload = async (att: AttachmentMeta) => {
		try {
			setDownloading(att.part_id)
			const saved = await invoke<boolean>('save_attachment', {
				accountId,
				mailbox,
				uid,
				partId: att.part_id,
				filename: att.filename || 'unnamed',
			})
			if (saved) {
				toast.success(t('inbox:messageView.attachments.downloadSuccess'))
			}
		} catch (error) {
			console.error('Download failed:', error)
			toast.error(t('inbox:messageView.attachments.downloadError'))
		} finally {
			setDownloading(null)
		}
	}

	if (!attachments || attachments.length === 0) {
		return null
	}

	const getIcon = (mimeType: string) => {
		if (mimeType.startsWith('image/'))
			return <FileImageIcon className='size-5 text-purple-400' />
		if (mimeType.startsWith('text/')) return <FileTextIcon className='size-5 text-blue-400' />
		if (
			mimeType.includes('zip') ||
			mimeType.includes('rar') ||
			mimeType.includes('tar') ||
			mimeType.includes('7z')
		)
			return <FileArchiveIcon className='size-5 text-amber-400' />
		return <FileIcon className='size-5 text-slate-400' />
	}

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
		hidden: { opacity: 0, scale: 0.95 },
		show: {
			opacity: 1,
			scale: 1,
			transition: { type: 'spring', stiffness: 300, damping: 24 },
		},
	}

	return (
		<motion.div
			className='mt-8 border-t border-white/[0.06] pt-6'
			variants={container}
			initial='hidden'
			animate='show'>
			<motion.div
				variants={item}
				className='mb-3 flex items-center gap-2 text-sm font-medium text-slate-300'>
				<PaperclipIcon className='size-4 text-slate-400' />
				<span>
					{attachments.length}{' '}
					{attachments.length === 1
						? t('inbox:messageView.attachments.one')
						: t('inbox:messageView.attachments.other')}
				</span>
			</motion.div>

			<div className='grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3'>
				{attachments.map((att) => (
					<motion.div
						key={att.part_id}
						variants={item}
						className='group relative flex items-center gap-3 rounded-lg border border-white/[0.06] bg-slate-900/30 p-3 transition-colors hover:border-white/[0.1] hover:bg-slate-800/50'>
						<div className='flex size-10 shrink-0 items-center justify-center rounded-lg bg-white/[0.03] ring-1 ring-white/[0.05] ring-inset'>
							{getIcon(att.mime_type)}
						</div>

						<div className='min-w-0 flex-1'>
							<p className='truncate text-sm font-medium text-slate-200'>
								{att.filename || 'Unnamed'}
							</p>
							<p className='text-xs text-slate-500'>{formatFileSize(att.size)}</p>
						</div>

						<button
							onClick={() => handleDownload(att)}
							disabled={downloading === att.part_id}
							className={`flex size-8 shrink-0 items-center justify-center rounded-md text-slate-400 transition-all hover:bg-white/[0.08] hover:text-slate-200 focus:opacity-100 disabled:cursor-not-allowed disabled:opacity-50 ${
								downloading === att.part_id
									? 'opacity-100'
									: 'opacity-0 group-hover:opacity-100'
							}`}>
							{downloading === att.part_id ? (
								<div className='size-4 animate-spin rounded-full border-2 border-slate-400 border-t-transparent' />
							) : (
								<DownloadIcon className='size-4' />
							)}
						</button>
					</motion.div>
				))}
			</div>
		</motion.div>
	)
}
