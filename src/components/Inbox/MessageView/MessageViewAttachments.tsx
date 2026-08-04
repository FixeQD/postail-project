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

import { useThemeStore } from '@/stores/themeStore'

export const MessageViewAttachments = ({
	attachments,
	accountId,
	mailbox,
	uid,
}: MessageViewAttachmentsProps) => {
	const accentColor = useThemeStore((s) => s.accentColor)
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
			return <FileImageIcon className='size-5 text-status-info' />
		if (mimeType.startsWith('text/')) return <FileTextIcon className='size-5 text-status-info' />
		if (
			mimeType.includes('zip') ||
			mimeType.includes('rar') ||
			mimeType.includes('tar') ||
			mimeType.includes('7z')
		)
			return <FileArchiveIcon className='size-5 text-status-warning' />
		return <FileIcon className='size-5 text-[var(--text-secondary)]' />
	}

	const container: Variants = {
		hidden: { opacity: 0, y: 10 },
		show: {
			opacity: 1,
			y: 0,
			transition: {
				staggerChildren: 0.04,
				delayChildren: 0.05,
				duration: 0.25,
				ease: [0.16, 1, 0.3, 1],
			},
		},
	}

	const item: Variants = {
		hidden: { opacity: 0, scale: 0.96, y: 8 },
		show: {
			opacity: 1,
			scale: 1,
			y: 0,
			transition: { duration: 0.2, ease: [0.16, 1, 0.3, 1] },
		},
	}

	return (
		<motion.div
			className='mt-8 border-t border-[var(--border-faint)] pt-6'
			variants={container}
			initial='hidden'
			animate='show'>
			<motion.div
				variants={item}
				className='mb-4 flex items-center gap-2 text-sm font-medium text-[var(--text-primary)]'>
				<PaperclipIcon className='size-4 text-[var(--text-secondary)]' />
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
						whileHover={{ y: -2 }}
						className='group relative flex items-center gap-3 overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-2.5 transition-all hover:border-transparent hover:shadow-lg'>
						{/* Glow Background on Hover */}
						<div
							className='absolute inset-0 opacity-0 transition-opacity duration-300 group-hover:opacity-10'
							style={{ backgroundColor: accentColor }}
						/>

						<div className='flex size-10 shrink-0 items-center justify-center rounded-lg bg-[var(--surface-active)] ring-1 ring-[var(--border-faint)] transition-transform ring-inset group-hover:scale-105'>
							{getIcon(att.mime_type)}
						</div>

						<div className='relative z-10 min-w-0 flex-1'>
							<p className='truncate text-sm font-medium text-[var(--text-primary)]'>
								{att.filename || 'Unnamed'}
							</p>
							<p className='text-xs text-[var(--text-secondary)]'>
								{formatFileSize(att.size)}
							</p>
						</div>

						<motion.button
							whileHover={{ scale: 1.1 }}
							whileTap={{ scale: 0.9 }}
							onClick={() => handleDownload(att)}
							disabled={downloading === att.part_id}
							style={{
								backgroundColor:
									downloading === att.part_id
										? 'transparent'
										: `${accentColor}1A`,
								color: accentColor,
							}}
							className={`relative z-10 mr-1 flex size-8 shrink-0 items-center justify-center rounded-full transition-all focus:opacity-100 disabled:cursor-not-allowed disabled:opacity-50 ${
								downloading === att.part_id
									? 'opacity-100'
									: 'opacity-0 shadow-sm backdrop-blur-sm group-hover:opacity-100'
							}`}>
							{downloading === att.part_id ? (
								<div
									className='size-4 animate-spin rounded-full border-2 border-t-transparent'
									style={{
										borderColor: accentColor,
										borderTopColor: 'transparent',
									}}
								/>
							) : (
								<DownloadIcon className='size-4' />
							)}
						</motion.button>
					</motion.div>
				))}
			</div>
		</motion.div>
	)
}
