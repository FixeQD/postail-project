import { useEffect, useState, useCallback, useRef, memo } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import type { EmailAttachment } from '@/types/compose'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import {
	$getSelection,
	$isRangeSelection,
	FORMAT_TEXT_COMMAND,
	$createTextNode,
	$insertNodes,
} from 'lexical'
import { $getNearestNodeOfType } from '@lexical/utils'
import { ListNode, INSERT_ORDERED_LIST_COMMAND, INSERT_UNORDERED_LIST_COMMAND } from '@lexical/list'
import { LinkNode, TOGGLE_LINK_COMMAND, $createLinkNode, formatUrl } from '@lexical/link'
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
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Popover, PopoverTrigger, PopoverContent } from '@/components/ui/popover'
import { Input } from '@/components/ui/input'
import { useDraftStore } from '@/stores/draftStore'

const useEditorFormats = (editor: any, linkPopoverOpen: boolean) => {
	const [formats, setFormats] = useState({
		bold: false,
		italic: false,
		underline: false,
		strikethrough: false,
		ordered: false,
		unordered: false,
		link: false,
	})
	const [linkData, setLinkData] = useState({ url: '', text: '' })

	useEffect(() => {
		return editor.registerUpdateListener(({ editorState }: { editorState: any }) => {
			editorState.read(() => {
				const selection = $getSelection()
				if ($isRangeSelection(selection)) {
					const anchorNode = selection.anchor.getNode()
					const listNode = $getNearestNodeOfType(anchorNode, ListNode)
					const linkNode = $getNearestNodeOfType(anchorNode, LinkNode)

					const newFormats = {
						bold: selection.hasFormat('bold'),
						italic: selection.hasFormat('italic'),
						underline: selection.hasFormat('underline'),
						strikethrough: selection.hasFormat('strikethrough'),
						ordered: listNode?.getListType() === 'number',
						unordered: listNode?.getListType() === 'bullet',
						link: !!linkNode,
					}
					setFormats(newFormats)

					if (!linkPopoverOpen) {
						setLinkData({
							url: linkNode?.getURL() || '',
							text: selection.getTextContent(),
						})
					}
				}
			})
		})
	}, [editor, linkPopoverOpen])

	return { formats, linkData }
}

const LinkPopover = memo(({ editor, formats, linkData }: any) => {
	const [open, setOpen] = useState(false)
	const [localText, setLocalText] = useState('')
	const [localUrl, setLocalUrl] = useState('')
	const textInputRef = useRef<HTMLInputElement>(null)

	useEffect(() => {
		if (open) {
			setLocalText(linkData.text)
			setLocalUrl(linkData.url)
			setTimeout(() => textInputRef.current?.focus(), 0)
		}
	}, [open, linkData])

	const applyLink = useCallback(() => {
		const url = formatUrl(localUrl.trim())
		if (!url) {
			editor.dispatchCommand(TOGGLE_LINK_COMMAND, null)
		} else {
			editor.update(() => {
				const selection = $getSelection()
				if ($isRangeSelection(selection)) {
					if (selection.getTextContent().trim().length > 0 && !localText.trim()) {
						editor.dispatchCommand(TOGGLE_LINK_COMMAND, url)
					} else {
						const linkNode = $createLinkNode(url)
						linkNode.append($createTextNode(localText.trim() || url))
						selection.insertNodes([linkNode])
					}
				} else {
					const linkNode = $createLinkNode(url)
					linkNode.append($createTextNode(localText.trim() || url))
					$insertNodes([linkNode])
				}
			})
		}
		setOpen(false)
		editor.focus()
	}, [editor, localText, localUrl])

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button variant='ghost' size='icon' className={`h-9 w-9 ${formats.link ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400 hover:bg-zinc-800'}`}>
					<LinkIcon className='h-4 w-4' />
				</Button>
			</PopoverTrigger>
			<PopoverContent side='bottom' align='start' className='rounded-md border border-zinc-800 bg-zinc-900 p-3 text-zinc-200 shadow-md'>
				<div className='flex flex-col gap-2'>
					<label className='text-xs text-zinc-400'>Text</label>
					<Input ref={textInputRef} value={localText} onChange={e => setLocalText(e.target.value)} className='bg-transparent' />
					<label className='text-xs text-zinc-400'>URL</label>
					<Input value={localUrl} onChange={e => setLocalUrl(e.target.value)} className='bg-transparent' onKeyDown={e => e.key === 'Enter' && applyLink()} />
					<div className='mt-2 flex justify-end gap-2'>
						<Button onClick={applyLink}>Apply</Button>
						<Button variant='ghost' onClick={() => { editor.dispatchCommand(TOGGLE_LINK_COMMAND, null); setOpen(false); }}>Remove</Button>
					</div>
				</div>
			</PopoverContent>
		</Popover>
	)
})

export function EditorToolbar() {
	const { t } = useTranslation()
	const [editor] = useLexicalComposerContext()
	const { formats, linkData } = useEditorFormats(editor, false)
	const { currentDraft, editorMode, setEditorMode, addAttachment } = useDraftStore()

	const [pendingAttachment, setPendingAttachment] = useState<{ attachment: EmailAttachment, path: string } | null>(null)
	const [dialogOpen, setDialogOpen] = useState(false)

	const exec = (cmd: any, val?: any) => {
		editor.dispatchCommand(cmd, val)
		editor.focus()
	}

	const handleAttachment = async () => {
		try {
			const selected = await open({
				multiple: true,
				title: 'Select attachments',
			})

			if (!selected) return

			const paths = Array.isArray(selected) ? selected : [selected]

			for (const path of paths) {
				if (!path) continue
				try {
					const attachment = await invoke<EmailAttachment>('add_attachment', { path })
					const isDuplicate = currentDraft?.attachments?.some(a => a.hash === attachment.hash)
					
					if (isDuplicate) {
						setPendingAttachment({ attachment, path })
						setDialogOpen(true)
						// For simplicity in this session, we stop at the first duplicate. 
						break;
					} else {
						addAttachment({ ...attachment, path })
					}
				} catch (err) {
					console.error('Failed to add attachment:', err)
				}
			}
		} catch (err) {
			console.error('Failed to open file picker:', err)
		}
	}

	const confirmDuplicate = () => {
		if (pendingAttachment) {
			addAttachment({ ...pendingAttachment.attachment, path: pendingAttachment.path })
			setPendingAttachment(null)
		}
		setDialogOpen(false)
	}

	return (
		<div className='flex items-center gap-2'>
			{editorMode === 'rich-text' && (
				<>
					<Button
						variant='ghost'
						size='icon'
						className='h-9 w-9 text-zinc-400 hover:bg-zinc-800'
						onClick={handleAttachment}>
						<Paperclip className='h-5 w-5' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.bold ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400'}`}
						onClick={() => exec(FORMAT_TEXT_COMMAND, 'bold')}>
						<Bold className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.italic ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400'}`}
						onClick={() => exec(FORMAT_TEXT_COMMAND, 'italic')}>
						<Italic className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.underline ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400'}`}
						onClick={() => exec(FORMAT_TEXT_COMMAND, 'underline')}>
						<Underline className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.strikethrough ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400'}`}
						onClick={() => exec(FORMAT_TEXT_COMMAND, 'strikethrough')}>
						<Strikethrough className='h-4 w-4' />
					</Button>
					<div className='mx-1 h-4 w-px bg-zinc-800' />
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.unordered ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400'}`}
						onClick={() => exec(INSERT_UNORDERED_LIST_COMMAND)}>
						<ListIcon className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className={`h-9 w-9 ${formats.ordered ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400'}`}
						onClick={() => exec(INSERT_ORDERED_LIST_COMMAND)}>
						<ListOrdered className='h-4 w-4' />
					</Button>
					<LinkPopover editor={editor} formats={formats} linkData={linkData} />
				</>
			)}

			<div className='mx-1 h-4 w-px bg-zinc-800' />

			<Button
				variant='ghost'
				size='icon'
				title={editorMode === 'rich-text' ? 'Switch to Source' : 'Switch to Rich Text'}
				className={`h-9 w-9 text-zinc-400 hover:bg-zinc-800 ${editorMode === 'source' ? 'bg-zinc-800 text-blue-400' : ''}`}
				onClick={() => setEditorMode(editorMode === 'rich-text' ? 'source' : 'rich-text')}>
				{editorMode === 'rich-text' ? <Code className='h-4 w-4' /> : <FileType className='h-4 w-4' />}
			</Button>

			<ConfirmationDialog
				open={dialogOpen}
				onOpenChange={setDialogOpen}
				title={t('compose.duplicateTitle')}
				description={t('compose.duplicateMessage', { filename: pendingAttachment?.attachment.filename })}
				confirmLabel={t('compose.addAnyway')}
				cancelLabel={t('actions.cancel')}
				onConfirm={confirmDuplicate}
			/>
		</div>
	)
}

export default memo(EditorToolbar)
