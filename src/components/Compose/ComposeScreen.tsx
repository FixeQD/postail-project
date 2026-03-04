import { useRef, useCallback, useMemo, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { LexicalComposer } from '@lexical/react/LexicalComposer'
import { EditorState, LexicalEditor } from 'lexical'
import { HeadingNode, QuoteNode } from '@lexical/rich-text'
import { ListNode, ListItemNode } from '@lexical/list'
import { LinkNode } from '@lexical/link'
import { motion, AnimatePresence } from 'framer-motion'
import { toast } from '../ui/custom/Toaster'
import { useToastStore } from '@/stores/toastStore'

import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useDraftStore } from '@/stores/draftStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { useComposeShortcuts } from '@/hooks/useComposeShortcuts'
import { useDragging, useLinkTooltip } from './useCompose'
import EditorContent from './Editor/EditorContent'
import { lexicalToHtml } from './Editor/utils/conversion'
import { ImageNode } from './Editor/Nodes/ImageNode'
import { CompatibilityPanel } from './CompatibilityPanel'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { ComposeHeader } from './ComposeHeader'
import { ComposeInputs } from './ComposeInputs'
import { ComposeFooter } from './ComposeFooter'
import type { ComposeScreenProps } from '@/types/components/compose'

export function ComposeScreen({ open, onOpenChange, accountId }: ComposeScreenProps) {
	const { t } = useTranslation()
	const animationsEnabled = useAnimationsEnabled()
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
		isSending,
	} = useDraftStore()

	const editorRef = useRef<HTMLDivElement>(null)
	const { position, size, isDragging, isResizing, startDrag, handleResizeMouseDown } =
		useDragging()

	const tooltipData = useLinkTooltip(editorRef)

	const htmlRef = useRef('')
	const isHydratingRef = useRef(false)
	const [changeCount, setChangeCount] = useState(0)
	const [autoFixKey, setAutoFixKey] = useState(0)
	const [showDiscardDialog, setShowDiscardDialog] = useState(false)
	const [isFixing, setIsFixing] = useState(false)
	const [isFlying, setIsFlying] = useState(false)
	const [isCountingDown, setIsCountingDown] = useState(false)
	const closedDuringCountdownRef = useRef(false)
	const [frozenLayout, setFrozenLayout] = useState<{
		position: { x: number; y: number }
		size: { width: number; height: number }
		target: { x: number; y: number }
	} | null>(null)

	const calculateFlyTarget = useCallback(
		(currentPos: { x: number; y: number }, currentSize: { width: number; height: number }) => {
			if (typeof window === 'undefined') return { x: 0, y: 0 }
			const targetX = window.innerWidth - 120
			const targetY = 28
			const centerX = currentPos.x + currentSize.width / 2
			const centerY = currentPos.y + currentSize.height / 2
			return {
				x: targetX - centerX,
				y: targetY - centerY,
			}
		},
		[]
	)

	const handleClose = useCallback(() => {
		if (isCountingDown) {
			closedDuringCountdownRef.current = true
			onOpenChange(false)
			return
		}
		if (isDirty) {
			setShowDiscardDialog(true)
		} else {
			saveDraft(htmlRef.current)
			onOpenChange(false)
			stopComposing()
		}
	}, [isCountingDown, isDirty, saveDraft, onOpenChange, stopComposing])

	const { settings } = useSettingsStore()

	const handleSend = useCallback(async () => {
		const delaySeconds = settings['undo-send-delay'] ?? 0

		// ── Strategic Delay ────────────────────────────────────────────────────
		if (delaySeconds > 0) {
			const TOAST_ID = 'send-countdown'
			const delayMs = delaySeconds * 1000

			closedDuringCountdownRef.current = false
			setIsCountingDown(true)

			const cancelled = await new Promise<boolean>((resolve) => {
				toast.loading(t('compose.sendingIn', 'Sending…'), {
					id: TOAST_ID,
					duration: delayMs,
					withCountdown: true,
					cancelFn: () => resolve(true),
				})
				setTimeout(() => resolve(false), delayMs)
			})

			setIsCountingDown(false)
			useToastStore.getState().removeToast(TOAST_ID)

			if (cancelled) {
				// If user closed compose during countdown, reopen it with draft intact
				if (closedDuringCountdownRef.current) {
					closedDuringCountdownRef.current = false
					onOpenChange(true)
				}
				toast.info(t('compose.sendCancelled', 'Send cancelled'), { duration: 2500 })
				return
			}
		}

		// ── Proceed with actual send ───────────────────────────────────────────
		try {
			const target = calculateFlyTarget(position, size)
			setFrozenLayout({ position, size, target })
			setIsFlying(true)

			await sendDraft(htmlRef.current)
			toast.success(t('compose.sendSuccess', 'Message sent successfully'))
			onOpenChange(false)
		} catch (error) {
			setIsFlying(false)
			setFrozenLayout(null)
			console.error('Send failed', error)
			toast.error(t('compose.sendError', 'Failed to send message'))
		}
	}, [sendDraft, onOpenChange, t, position, size, calculateFlyTarget, settings])

	useEffect(() => {
		if (open) {
			setIsFlying(false)
			setFrozenLayout(null)
			setIsCountingDown(false)
		}
	}, [open])

	const handleSaveDraft = useCallback(() => {
		saveDraft(htmlRef.current)
	}, [saveDraft])

	const handleAttachFile = useCallback(() => {
		useDraftStore.getState().triggerAttachFile()
	}, [])

	const handleInsertLink = useCallback(() => {
		useDraftStore.getState().triggerInsertLink()
	}, [])

	// Keyboard shortcuts control state
	const [showCc, setShowCc] = useState(false)
	const [showBcc, setShowBcc] = useState(false)

	const handleToggleCc = useCallback(() => setShowCc((prev) => !prev), [])
	const handleToggleBcc = useCallback(() => setShowBcc((prev) => !prev), [])

	const handleEditorMount = useCallback(() => {
		if (isFixing) {
			setTimeout(() => {
				setIsFixing(false)
			}, 300)
		}
	}, [isFixing])

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

	const handleDiscard = useCallback(async () => {
		if (currentDraft?.id) {
			await useDraftStore.getState().deleteDraft(currentDraft.id)
		}
		stopComposing()
		onOpenChange(false)
		setShowDiscardDialog(false)
	}, [currentDraft, stopComposing, onOpenChange])

	const isValid = useMemo(() => {
		if (!currentDraft) return false
		const hasRecipients = currentDraft.to && currentDraft.to.length > 0
		const hasSubject = currentDraft.subject && currentDraft.subject.trim() !== ''
		const bodyContent = currentDraft.body
		const hasBody = bodyContent && bodyContent.trim() !== '' && bodyContent !== '<p><br></p>'
		return !!(hasRecipients && hasSubject && hasBody)
	}, [currentDraft])

	const activePosition = isFlying && frozenLayout ? frozenLayout.position : position
	const activeSize = isFlying && frozenLayout ? frozenLayout.size : size
	const activeTarget = isFlying && frozenLayout ? frozenLayout.target : { x: 0, y: 0 }

	// Validate tooltip URL schemes to avoid rendering javascript: or other unsafe protocols. Only allow http:, https:, mailto:.
	const isSafeUrl = (u: string) => {
		try {
			const parsed = new URL(u)
			return (
				parsed.protocol === 'http:' ||
				parsed.protocol === 'https:' ||
				parsed.protocol === 'mailto:'
			)
		} catch (e) {
			return false
		}
	}

	return (
		<AnimatePresence>
			{open && (
				<motion.div
					{...(animationsEnabled
						? {
								initial: { opacity: 0, y: 50, scale: 0.95 },
								animate:
									isSending || isFlying
										? {
												opacity: 0,
												scale: 0.1,
												x: activeTarget.x,
												y: activeTarget.y,
												transition: { duration: 0.5, ease: 'easeInOut' },
											}
										: isCountingDown
											? {
													opacity: 0.58,
													y: 0,
													scale: 1,
													x: 0,
													transition: { duration: 0.35, ease: 'easeOut' },
												}
											: { opacity: 1, y: 0, scale: 1, x: 0 },
								exit: isFlying
									? {
											opacity: 0,
											scale: 0.1,
											x: activeTarget.x,
											y: activeTarget.y,
											transition: { duration: 0.5, ease: 'easeInOut' },
										}
									: {
											opacity: 0,
											y: 20,
											scale: 0.95,
											transition: { duration: 0.2 },
										},
								transition: { type: 'spring', duration: 0.4, bounce: 0.3 },
							}
						: {})}
					className={`compose-drag-root fixed z-50 flex flex-col rounded-t-xl bg-zinc-950 text-zinc-100 shadow-2xl ring-1 ${isCountingDown ? 'ring-blue-500/30' : 'ring-zinc-800'} ${isDragging ? 'shadow-blue-900/20' : ''}`}
					style={{
						left: `${activePosition.x}px`,
						top: `${activePosition.y}px`,
						width: `${activeSize.width}px`,
						height: `${activeSize.height}px`,
						cursor: isDragging ? 'grabbing' : 'auto',
						pointerEvents: 'auto',
						boxShadow: isCountingDown
							? '0 0 0 1px rgba(59,130,246,0.25), 0 25px 60px rgba(0,0,0,0.7)'
							: undefined,
					}}>
					{/* Countdown overlay - blocks all interaction */}
					{isCountingDown && (
						<div
							className='absolute inset-0 z-20 rounded-t-xl'
							style={{
								background:
									'linear-gradient(135deg, rgba(59,130,246,0.06) 0%, rgba(139,92,246,0.04) 60%, transparent 100%)',
								cursor: 'not-allowed',
							}}
						/>
					)}
					{/* Drag/resize interaction shield */}
					{(isDragging || isResizing) && (
						<div className='absolute inset-0 z-[9999] cursor-grabbing' />
					)}
					<ComposeHeader
						isDragging={isDragging}
						onMouseDown={startDrag}
						onClose={handleClose}
						isCountingDown={isCountingDown}
					/>

					<ComposeInputs
						to={currentDraft?.to || []}
						cc={currentDraft?.cc || []}
						bcc={currentDraft?.bcc || []}
						subject={currentDraft?.subject || ''}
						showCc={showCc}
						showBcc={showBcc}
						setShowCc={setShowCc}
						setShowBcc={setShowBcc}
						onUpdate={updateCurrentDraft}
						onAddRecipient={addRecipient}
						onRemoveRecipient={removeRecipient}
					/>

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
								setIsFixing(true)

								const fixedHtml = await applyAutoFix(htmlRef.current)
								htmlRef.current = fixedHtml
								setAutoFixKey((k) => k + 1) // Force re-render SourceEditor
							}
						}}
						hasIssues={compatibilityIssues.length > 0}
						composeX={position.x}
						composeY={position.y}
						composeHeight={size.height}
					/>

					<LexicalComposer initialConfig={initialConfig}>
						<EditorContent
							editorRef={editorRef}
							htmlRef={htmlRef}
							isHydratingRef={isHydratingRef}
							handleEditorChange={handleEditorChange}
							attachments={currentDraft?.attachments || []}
							onRemoveAttachment={removeAttachment}
							onSourceChange={triggerValidation}
							autoFixKey={autoFixKey}
							isFixing={isFixing}
							onEditorMount={handleEditorMount}
						/>

						<ComposeFooter
							onSend={handleSend}
							onDiscard={() => setShowDiscardDialog(true)}
							isValid={isValid}
						/>
					</LexicalComposer>

					{tooltipData.visible && tooltipData.rect && isSafeUrl(tooltipData.url) && (
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
						title={String(t('validation:sendWarning.title'))}
						description={String(
							t('validation:sendWarning.description', {
								count: compatibilityIssues.length,
							})
						)}
						confirmLabel={String(t('validation:sendWarning.confirm'))}
						cancelLabel={String(t('validation:sendWarning.cancel'))}
						onConfirm={() => {
							dismissValidationWarning()
							handleSend()
						}}
					/>

					{/* Discard Draft Dialog */}
					<ConfirmationDialog
						open={showDiscardDialog}
						onOpenChange={setShowDiscardDialog}
						title={String(t('compose.discard.title'))}
						description={String(t('compose.discard.description'))}
						confirmLabel={String(t('compose.discard.confirm'))}
						cancelLabel={String(t('compose.discard.cancel'))}
						onConfirm={handleDiscard}
					/>
				</motion.div>
			)}
		</AnimatePresence>
	)
}
