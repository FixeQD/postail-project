import { useRef, useCallback, useMemo, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { LexicalComposer } from '@lexical/react/LexicalComposer'
import { EditorState, LexicalEditor } from 'lexical'
import { HeadingNode, QuoteNode } from '@lexical/rich-text'
import { ListNode, ListItemNode } from '@lexical/list'
import { LinkNode } from '@lexical/link'
import { X, Minimize2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { useDraftStore } from '@/stores/draftStore'
import { useDragging, useLinkTooltip } from './useCompose'
import EditorContent from './Editor/EditorContent'
import { lexicalToHtml } from './Editor/utils/conversion'
import { AddressInput } from './Inputs/AddressInput'
import { SubjectInput } from './Inputs/SubjectInput'
import { ImageNode } from './Editor/Nodes/ImageNode'

interface ComposeScreenProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	accountId?: string
}

export function ComposeScreen({ open, onOpenChange, accountId }: ComposeScreenProps) {
	const { t } = useTranslation()
	const {
		currentDraft,
		isComposing,
		isDirty,
		setSubject,
		updateCurrentDraft,
		startComposing,
		stopComposing,
		saveDraft,
		markDirty,
		addRecipient,
		removeRecipient,
		removeAttachment,
	} = useDraftStore()

	const editorRef = useRef<HTMLDivElement>(null)
	const { position, size, isDragging, startDrag, handleResizeMouseDown } = useDragging()
	const tooltipData = useLinkTooltip(editorRef)

	const htmlRef = useRef('')
	const isHydratingRef = useRef(false)
	const [changeCount, setChangeCount] = useState(0)
	const [showCc, setShowCc] = useState(false)
	const [showBcc, setShowBcc] = useState(false)

	const handleEditorChange = useCallback(
		(editorState: EditorState, editor: LexicalEditor) => {
			if (isHydratingRef.current) return
			editorState.read(() => {
				htmlRef.current = lexicalToHtml(editor)
				setChangeCount((c) => c + 1)
				markDirty()
			})
		},
		[markDirty]
	)

	// Lexical initial config
	const initialConfig = useMemo(
		() => ({
			namespace: 'ComposeEditor',
			theme: {
				text: { bold: 'font-bold', italic: 'italic', underline: 'underline', strikethrough: 'line-through' },
				list: { listitem: '!ml-4', nested: { listitem: '!ml-8' }, ol: '!list-decimal !ml-4', ul: '!list-disc !ml-4' },
				link: 'underline text-cyan-400',
			},
			nodes: [HeadingNode, QuoteNode, ListNode, ListItemNode, LinkNode, ImageNode],
			onError: (err: Error) => console.error(err),
		}),
		[]
	)

	useEffect(() => {
		if (open && !isComposing && accountId) startComposing(accountId)
	}, [open, isComposing, startComposing, accountId])

	useEffect(() => {
		if (!isDirty || !currentDraft || htmlRef.current === currentDraft.body) return
		const timer = setTimeout(() => saveDraft(htmlRef.current), 3000)
		return () => clearTimeout(timer)
	}, [isDirty, currentDraft, saveDraft, changeCount])

	// Automatically show Cc/Bcc fields if they have recipients
	useEffect(() => {
		if (currentDraft?.cc && currentDraft.cc.length > 0) setShowCc(true)
		if (currentDraft?.bcc && currentDraft.bcc.length > 0) setShowBcc(true)
	}, [currentDraft])

	if (!open) return null

	return (
		<div
			className={`fixed z-50 flex flex-col overflow-hidden rounded-t-xl bg-zinc-950 text-zinc-100 shadow-2xl ring-1 ring-zinc-800 ${isDragging ? 'shadow-blue-900/20' : ''}`}
			style={{
				left: `${position.x}px`,
				top: `${position.y}px`,
				width: `${size.width}px`,
				height: `${size.height}px`,
				cursor: isDragging ? 'grabbing' : 'auto',
			}}>
			<div
				className='flex w-full items-center justify-between bg-zinc-900 px-4 py-3 select-none'
				onMouseDown={startDrag}
				style={{ cursor: isDragging ? 'grabbing' : 'grab' }}>
				<h2 className='text-sm font-medium text-zinc-300'>{t('compose.newMessage')}</h2>
				<div className='flex items-center gap-1'>
					<Button variant='ghost' size='icon' className='h-7 w-7 text-zinc-400'><Minimize2 className='h-4 w-4' /></Button>
					<Button
						variant='ghost'
						size='icon'
						className='h-7 w-7 text-zinc-400 hover:text-zinc-100'
						onClick={() => {
							saveDraft(htmlRef.current)
							onOpenChange(false)
							stopComposing()
						}}>
						<X className='h-4 w-4' />
					</Button>
				</div>
			</div>

			<div className='flex flex-col px-4 pt-1'>
				<AddressInput
					label={t('compose.to')}
					recipients={currentDraft?.to || []}
					onAdd={(recipient) => addRecipient('to', recipient)}
					onRemove={(email) => removeRecipient('to', email)}
					placeholder={t('compose.recipients')}
					rightElement={
						<div className='flex gap-2 mr-2'>
							{!showCc && (
								<button
									type='button'
									onClick={() => setShowCc(true)}
									className='text-xs font-medium text-zinc-500 hover:text-zinc-300 transition-colors'>
									{t('compose.cc')}
								</button>
							)}
							{!showBcc && (
								<button
									type='button'
									onClick={() => setShowBcc(true)}
									className='text-xs font-medium text-zinc-500 hover:text-zinc-300 transition-colors'>
									{t('compose.bcc')}
								</button>
							)}
						</div>
					}
				/>
				{showCc && (
					<AddressInput
						label={t('compose.cc')}
						recipients={currentDraft?.cc || []}
						onAdd={(recipient) => addRecipient('cc', recipient)}
						onRemove={(email) => removeRecipient('cc', email)}
						rightElement={
							<button
								type='button'
								onClick={() => {
									setShowCc(false)
									updateCurrentDraft({ cc: [] })
								}}
								className='mr-2 text-zinc-500 hover:text-zinc-300 transition-colors'>
								<X className='h-3.5 w-3.5' />
							</button>
						}
					/>
				)}
				{showBcc && (
					<AddressInput
						label={t('compose.bcc')}
						recipients={currentDraft?.bcc || []}
						onAdd={(recipient) => addRecipient('bcc', recipient)}
						onRemove={(email) => removeRecipient('bcc', email)}
						rightElement={
							<button
								type='button'
								onClick={() => {
									setShowBcc(false)
									updateCurrentDraft({ bcc: [] })
								}}
								className='mr-2 text-zinc-500 hover:text-zinc-300 transition-colors'>
								<X className='h-3.5 w-3.5' />
							</button>
						}
					/>
				)}
				<SubjectInput
					placeholder={t('compose.subject')}
					value={currentDraft?.subject || ''}
					onChange={setSubject}
				/>
			</div>


			<LexicalComposer initialConfig={initialConfig}>
				<EditorContent
					onOpenChange={onOpenChange}
					editorRef={editorRef}
					htmlRef={htmlRef}
					isHydratingRef={isHydratingRef}
					handleEditorChange={handleEditorChange}
					attachments={currentDraft?.attachments || []}
					onRemoveAttachment={removeAttachment}
				/>
			</LexicalComposer>

			{tooltipData.visible && tooltipData.rect && (
				<div
					className='bg-popover text-popover-foreground fixed z-50 max-w-md truncate rounded-md px-3 py-1.5 text-xs'
					style={{
						left: `${tooltipData.rect.left + tooltipData.rect.width / 2}px`,
						top: `${tooltipData.rect.top > 40 ? tooltipData.rect.top - 8 : tooltipData.rect.bottom + 8}px`,
						transform: tooltipData.rect.top > 40 ? 'translate(-50%, -100%)' : 'translate(-50%, 0)',
					}}>
					{tooltipData.url.length > 120 ? tooltipData.url.slice(0, 116) + '…' : tooltipData.url}
				</div>
			)}

			<div className='absolute right-0 bottom-0 h-4 w-4 cursor-se-resize' onMouseDown={handleResizeMouseDown} />
		</div>
	)
}
