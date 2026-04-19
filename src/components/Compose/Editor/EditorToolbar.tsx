import { useEffect, useState, useCallback, memo } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { invokeWithErrorLog } from '@/lib/tauri'
import { useTranslation } from 'react-i18next'
import type { EmailAttachment } from '@/types/compose'
import type { EditorToolbarProps } from '@/types/components/compose'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { CompatibilityButton } from '@/components/Compose/CompatibilityButton'
import {
	Paperclip,
	Bold,
	Italic,
	Underline,
	Strikethrough,
	List as ListIcon,
	ListOrdered,
	Link as LinkIcon,
	Code,
	FileType,
	MailCheck,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useDraftStore } from '@/stores/draftStore'

const useEditorFormats = () => {
	const [formats, setFormats] = useState({
		bold: false,
		italic: false,
		underline: false,
		strikethrough: false,
		ordered: false,
		unordered: false,
	})

	useEffect(() => {
		const isFormatActive = (command: string) => {
			const selection = window.getSelection()
			if (!selection || !selection.anchorNode) return false

			let node: Node | null = selection.anchorNode
			if (node.nodeType === Node.TEXT_NODE) {
				node = node.parentNode
			}

			while (
				node &&
				node instanceof Element &&
				node.getAttribute('contenteditable') !== 'true'
			) {
				const style = window.getComputedStyle(node)
				switch (command) {
					case 'bold':
						if (
							node.nodeName === 'B' ||
							node.nodeName === 'STRONG' ||
							parseInt(style.fontWeight) >= 600 ||
							style.fontWeight === 'bold'
						)
							return true
						break
					case 'italic':
						if (
							node.nodeName === 'I' ||
							node.nodeName === 'EM' ||
							style.fontStyle === 'italic'
						)
							return true
						break
					case 'underline':
						if (
							node.nodeName === 'U' ||
							style.textDecorationLine.includes('underline')
						)
							return true
						break
					case 'strikeThrough':
						if (
							node.nodeName === 'STRIKE' ||
							node.nodeName === 'S' ||
							node.nodeName === 'DEL' ||
							style.textDecorationLine.includes('line-through')
						)
							return true
						break
					case 'insertOrderedList':
						if (node.nodeName === 'OL') return true
						break
					case 'insertUnorderedList':
						if (node.nodeName === 'UL') return true
						break
				}
				node = node.parentNode
			}
			return false
		}

		const handleSelectionChange = () => {
			setFormats({
				bold: isFormatActive('bold'),
				italic: isFormatActive('italic'),
				underline: isFormatActive('underline'),
				strikethrough: isFormatActive('strikeThrough'),
				ordered: isFormatActive('insertOrderedList'),
				unordered: isFormatActive('insertUnorderedList'),
			})
		}

		document.addEventListener('selectionchange', handleSelectionChange)
		return () => document.removeEventListener('selectionchange', handleSelectionChange)
	}, [])

	return formats
}

export function EditorToolbar({ onAttach }: EditorToolbarProps) {
	const { t } = useTranslation()
	const formats = useEditorFormats()
	const { currentDraft, editorMode, setEditorMode, addAttachment, updateCurrentDraft } =
		useDraftStore()
	const requestReadReceipt = currentDraft?.requestReadReceipt ?? false

	const [pendingAttachment, setPendingAttachment] = useState<{
		attachment: EmailAttachment
		path: string
	} | null>(null)
	const [dialogOpen, setDialogOpen] = useState(false)

	const exec = (command: string, value: string | undefined = undefined) => {
		document.execCommand(command, false, value)
	}

	const handleAttachment = useCallback(async () => {
		try {
			const selected = await open({
				multiple: true,
				title: 'Select attachments',
			})

			if (!selected) return

			const paths = Array.isArray(selected) ? selected : [selected]

			for (const path of paths) {
				if (!path) continue
				const attachment = await invokeWithErrorLog<EmailAttachment>(
					'add_attachment',
					{ path },
					'add_attachment'
				)
				if (!attachment) continue

				const isDuplicate = currentDraft?.attachments?.some(
					(a) => a.hash === attachment.hash
				)

				if (isDuplicate) {
					setPendingAttachment({ attachment, path })
					setDialogOpen(true)
					break
				} else {
					addAttachment({ ...attachment, path })
					if (onAttach) onAttach()
				}
			}
		} catch (err) {
			console.error('Failed to open file picker:', err)
		}
	}, [currentDraft, addAttachment, onAttach])

	const confirmDuplicate = () => {
		if (pendingAttachment) {
			addAttachment({ ...pendingAttachment.attachment, path: pendingAttachment.path })
			setPendingAttachment(null)
		}
		setDialogOpen(false)
	}

	const handleAttachFile = useCallback(() => {
		handleAttachment()
	}, [handleAttachment])

	useEffect(() => {
		window.addEventListener('compose:attach-file', handleAttachFile)
		return () => window.removeEventListener('compose:attach-file', handleAttachFile)
	}, [handleAttachFile])

	return (
		<div className='flex items-center gap-2'>
			{editorMode === 'rich-text' && (
				<>
					<Button
						variant='ghost'
						size='icon'
						className='h-9 w-9 text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'
						onClick={handleAttachment}>
						<Paperclip className='h-5 w-5' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.bold ? 'bg-[var(--compose-active)] text-[var(--compose-text)]' : 'text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'}`}
						onClick={() => exec('bold')}>
						<Bold className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.italic ? 'bg-[var(--compose-active)] text-[var(--compose-text)]' : 'text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'}`}
						onClick={() => exec('italic')}>
						<Italic className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.underline ? 'bg-[var(--compose-active)] text-[var(--compose-text)]' : 'text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'}`}
						onClick={() => exec('underline')}>
						<Underline className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.strikethrough ? 'bg-[var(--compose-active)] text-[var(--compose-text)]' : 'text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'}`}
						onClick={() => exec('strikeThrough')}>
						<Strikethrough className='h-4 w-4' />
					</Button>
					<div className='mx-1 h-4 w-px bg-[var(--compose-separator)]' />
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.unordered ? 'bg-[var(--compose-active)] text-[var(--compose-text)]' : 'text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'}`}
						onClick={() => exec('insertUnorderedList')}>
						<ListIcon className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.ordered ? 'bg-[var(--compose-active)] text-[var(--compose-text)]' : 'text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'}`}
						onClick={() => exec('insertOrderedList')}>
						<ListOrdered className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className='h-9 w-9 text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'
						onClick={() => {
							const url = prompt('Enter URL')
							if (url) exec('createLink', url)
						}}>
						<LinkIcon className='h-4 w-4' />
					</Button>
					<div className='mx-1 h-4 w-px bg-[var(--compose-separator)]' />
					<Button
						variant='ghost'
						size='icon'
						title={t('compose.requestReadReceipt')}
						className={`h-9 w-9 ${requestReadReceipt ? 'bg-[var(--compose-active)] text-sky-400' : 'text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'}`}
						onClick={() =>
							updateCurrentDraft({ requestReadReceipt: !requestReadReceipt })
						}>
						<MailCheck className='h-4 w-4' />
					</Button>
				</>
			)}

			<div className='mx-1 h-4 w-px bg-[var(--compose-separator)]' />

			<Button
				variant='ghost'
				size='icon'
				title={editorMode === 'rich-text' ? 'Switch to Source' : 'Switch to Rich Text'}
				className={`h-9 w-9 text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)] ${editorMode === 'source' ? 'bg-[var(--compose-active)] text-blue-400' : ''}`}
				onClick={() => setEditorMode(editorMode === 'rich-text' ? 'source' : 'rich-text')}>
				{editorMode === 'rich-text' ? (
					<Code className='h-4 w-4' />
				) : (
					<FileType className='h-4 w-4' />
				)}
			</Button>

			{/* Compatibility panel button - only in source mode */}
			{editorMode === 'source' && (
				<>
					<div className='mx-1 h-4 w-px bg-[var(--compose-separator)]' />
					<CompatibilityButton
						isOpen={useDraftStore.getState().compatibilityPanelOpen}
						onClick={() => useDraftStore.getState().toggleCompatibilityPanel()}
						issues={useDraftStore.getState().compatibilityIssues}
						isLoading={useDraftStore.getState().isValidating}
					/>
				</>
			)}

			<ConfirmationDialog
				open={dialogOpen}
				onOpenChange={setDialogOpen}
				title={t('compose.duplicateTitle')}
				description={t('compose.duplicateMessage', {
					filename: pendingAttachment?.attachment.filename,
				})}
				confirmLabel={t('compose.addAnyway')}
				cancelLabel={t('actions.cancel')}
				onConfirm={confirmDuplicate}
			/>
		</div>
	)
}

export default memo(EditorToolbar)
