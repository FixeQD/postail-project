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
import { useComposeShortcuts } from '@/hooks/useComposeShortcuts'
import { useDragging, useLinkTooltip } from './useCompose'
import EditorContent from './Editor/EditorContent'
import { lexicalToHtml } from './Editor/utils/conversion'
import { AddressInput } from './Inputs/AddressInput'
import { SubjectInput } from './Inputs/SubjectInput'
import { ImageNode } from './Editor/Nodes/ImageNode'
import { CompatibilityPanel } from './CompatibilityPanel'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'

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
		editorMode,
		compatibilityPanelOpen,
		compatibilityPanelWidth,
		compatibilityIssues,
		isValidating,
		showSendWarning,
		setSubject,
		updateCurrentDraft,
		startComposing,
		stopComposing,
		saveDraft,
		markDirty,
		addRecipient,
		removeRecipient,
		removeAttachment,
		validateCompatibility,
		applyAutoFix,
		toggleCompatibilityPanel,
		setCompatibilityPanelWidth,
		dismissValidationWarning,
		setShowSendWarning,
		sendDraft,
	} = useDraftStore()

	const editorRef = useRef<HTMLDivElement>(null)
	const { position, size, isDragging, isResizing, startDrag, handleResizeMouseDown } =
		useDragging()

	// Disable all interactions during drag/resize
	useEffect(() => {
		if (isDragging || isResizing) {
			document.body.style.pointerEvents = 'none'
			document.body.style.userSelect = 'none'
		} else {
			document.body.style.pointerEvents = ''
			document.body.style.userSelect = ''
		}
		return () => {
			document.body.style.pointerEvents = ''
			document.body.style.userSelect = ''
		}
	}, [isDragging, isResizing])
	const tooltipData = useLinkTooltip(editorRef)

	const htmlRef = useRef('')
	const isHydratingRef = useRef(false)
	const [changeCount, setChangeCount] = useState(0)
	const [autoFixKey, setAutoFixKey] = useState(0)
	const [showCc, setShowCc] = useState(false)
	const [showBcc, setShowBcc] = useState(false)
	const [showDiscardDialog, setShowDiscardDialog] = useState(false)

	const handleClose = useCallback(() => {
		if (isDirty) {
			setShowDiscardDialog(true)
		} else {
			saveDraft(htmlRef.current)
			onOpenChange(false)
			stopComposing()
		}
	}, [isDirty, saveDraft, onOpenChange, stopComposing])

	const handleSend = useCallback(async () => {
		try {
			await sendDraft(htmlRef.current)
			onOpenChange(false)
		} catch {
			// Error handling is in sendDraft
		}
	}, [sendDraft, onOpenChange])

	const handleSaveDraft = useCallback(() => {
		saveDraft(htmlRef.current)
	}, [saveDraft])

	const handleAttachFile = useCallback(() => {
		useDraftStore.getState().triggerAttachFile()
	}, [])

	const handleInsertLink = useCallback(() => {
		useDraftStore.getState().triggerInsertLink()
	}, [])

	const handleToggleCc = useCallback(() => {
		setShowCc((prev) => !prev)
	}, [])

	const handleToggleBcc = useCallback(() => {
		setShowBcc((prev) => !prev)
	}, [])

	// Register them
	useComposeShortcuts({
		onSend: handleSend,
		onSaveDraft: handleSaveDraft,
		onClose: handleClose,
		onAttachFile: handleAttachFile,
		onInsertLink: handleInsertLink,
		onToggleCc: handleToggleCc,
		onToggleBcc: handleToggleBcc,
		enabled: open,
	})

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

	const triggerValidation = useCallback(() => {
		setChangeCount((c) => c + 1)
		markDirty()
	}, [markDirty])

	// Lexical initial config
	const initialConfig = useMemo(
		() => ({
			namespace: 'ComposeEditor',
			theme: {
				text: {
					bold: 'font-bold',
					italic: 'italic',
					underline: 'underline',
					strikethrough: 'line-through',
				},
				list: {
					listitem: '!ml-4',
					nested: { listitem: '!ml-8' },
					ol: '!list-decimal !ml-4',
					ul: '!list-disc !ml-4',
				},
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
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [isDirty, currentDraft, saveDraft, changeCount])

	useEffect(() => {
		if (editorMode !== 'source') return

		const timer = setTimeout(() => {
			validateCompatibility(htmlRef.current || '')
		}, 800)

		return () => clearTimeout(timer)
	}, [editorMode, changeCount, validateCompatibility])

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
				pointerEvents: 'auto',
			}}>
			<div
				className='flex w-full items-center justify-between bg-zinc-900 px-4 py-3 select-none'
				onMouseDown={startDrag}
				style={{ cursor: isDragging ? 'grabbing' : 'grab' }}>
				<h2 className='text-sm font-medium text-zinc-300'>{t('compose.newMessage')}</h2>
				<div className='flex items-center gap-1'>
					<Button variant='ghost' size='icon' className='h-7 w-7 text-zinc-400'>
						<Minimize2 className='h-4 w-4' />
					</Button>
					<Button
						variant='ghost'
						size='icon'
						className='h-7 w-7 text-zinc-400 hover:text-zinc-100'
						onClick={handleClose}>
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
						<div className='mr-2 flex gap-2'>
							{!showCc && (
								<button
									type='button'
									onClick={() => setShowCc(true)}
									className='text-xs font-medium text-zinc-500 transition-colors hover:text-zinc-300'>
									{t('compose.cc')}
								</button>
							)}
							{!showBcc && (
								<button
									type='button'
									onClick={() => setShowBcc(true)}
									className='text-xs font-medium text-zinc-500 transition-colors hover:text-zinc-300'>
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
								className='mr-2 text-zinc-500 transition-colors hover:text-zinc-300'>
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
								className='mr-2 text-zinc-500 transition-colors hover:text-zinc-300'>
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

			<CompatibilityPanel
				isOpen={compatibilityPanelOpen && editorMode === 'source'}
				onClose={toggleCompatibilityPanel}
				width={compatibilityPanelWidth}
				onWidthChange={setCompatibilityPanelWidth}
				issues={compatibilityIssues}
				isLoading={isValidating}
				onCheckAgain={() => {
					if (htmlRef.current) {
						validateCompatibility(htmlRef.current, true)
					}
				}}
				onAutoFix={async () => {
					if (htmlRef.current) {
						const fixedHtml = await applyAutoFix(htmlRef.current)
						htmlRef.current = fixedHtml
						setAutoFixKey((k) => k + 1) // Force SourceEditor re-render with new HTML
					}
				}}
				hasIssues={compatibilityIssues.length > 0}
				composeX={position.x}
				composeY={position.y}
				composeHeight={size.height}
			/>

			<LexicalComposer initialConfig={initialConfig}>
				<EditorContent
					onOpenChange={onOpenChange}
					editorRef={editorRef}
					htmlRef={htmlRef}
					isHydratingRef={isHydratingRef}
					handleEditorChange={handleEditorChange}
					attachments={currentDraft?.attachments || []}
					onRemoveAttachment={removeAttachment}
					onSourceChange={triggerValidation}
					autoFixKey={autoFixKey}
				/>
			</LexicalComposer>

			{tooltipData.visible && tooltipData.rect && (
				<div
					className='bg-popover text-popover-foreground fixed z-50 max-w-md truncate rounded-md px-3 py-1.5 text-xs'
					style={{
						left: `${tooltipData.rect.left + tooltipData.rect.width / 2}px`,
						top: `${tooltipData.rect.top > 40 ? tooltipData.rect.top - 8 : tooltipData.rect.bottom + 8}px`,
						transform:
							tooltipData.rect.top > 40
								? 'translate(-50%, -100%)'
								: 'translate(-50%, 0)',
					}}>
					{tooltipData.url.length > 120
						? tooltipData.url.slice(0, 116) + '…'
						: tooltipData.url}
				</div>
			)}

			<div
				className='absolute right-0 bottom-0 h-4 w-4 cursor-se-resize'
				onMouseDown={handleResizeMouseDown}
			/>

			{/* Send Warning Dialog */}
			<ConfirmationDialog
				open={showSendWarning}
				onOpenChange={setShowSendWarning}
				title={t('validation:sendWarning.title')}
				description={t('validation:sendWarning.description', {
					count: compatibilityIssues.length,
				})}
				confirmLabel={t('validation:sendWarning.confirm')}
				cancelLabel={t('validation:sendWarning.cancel')}
				onConfirm={() => {
					dismissValidationWarning()
					sendDraft().then(() => {
						onOpenChange(false)
					})
				}}
			/>

			{/* Discard Draft Dialog */}
			<ConfirmationDialog
				open={showDiscardDialog}
				onOpenChange={setShowDiscardDialog}
				title={t('compose.discard.title')}
				description={t('compose.discard.description')}
				confirmLabel={t('compose.discard.confirm')}
				cancelLabel={t('compose.discard.cancel')}
				onConfirm={() => {
					onOpenChange(false)
					stopComposing()
				}}
			/>
		</div>
	)
}
