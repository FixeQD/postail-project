import { useState, useEffect, useRef, useCallback } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { invokeWithErrorLog } from '@/lib/tauri'
import { AlertCircle, Mail } from 'lucide-react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'

import { Button } from '@/components/ui/button'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'

import { useMessageViewStore } from '@/stores/messageViewStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { useDraftStore } from '@/stores/draftStore'
import type { MessageFull, ThreadView as ThreadViewType } from '@/types/mail'
import { MessageViewHeader } from './MessageViewHeader'
import { MessageViewMeta } from './MessageViewMeta'
import { MessageViewBody } from './MessageViewBody'
import { MessageViewAttachments } from './MessageViewAttachments'
import { MessageViewErrorBoundary } from './MessageViewErrorBoundary'
import { ThreadView } from './ThreadView'
import { useInboxShortcuts } from '@/hooks/useInboxShortcuts'
import { toast } from '@/stores/toastStore'
import { MessageViewSkeleton } from './MessageViewSkeleton'
import type { MessageViewProps } from '@/types/components/shared'

export const MessageView = ({
	accountId,
	mailbox,
	uid,
	onBack,
	onNext,
	onPrev,
}: MessageViewProps) => {
	const { t } = useTypedTranslation(['common', 'inbox'])
	const queryClient = useQueryClient()
	const { viewMode, toggleViewMode, setTitleMeta, setLoading } = useMessageViewStore()
	const blockExternalImages = useSettingsStore((s) => s.settings['block-external-images'])
	const blockReadReceipts = useSettingsStore((s) => s.settings['block-read-receipts'])
	const markAsReadDelay = useSettingsStore((s) => s.settings['mark-as-read-delay'])
	const threadViewEnabled = useSettingsStore((s) => s.settings['thread-view'])

	const [allowExternalResources, setAllowExternalResources] = useState(!blockExternalImages)
	const [hasExternalResources, setHasExternalResources] = useState(false)
	const [loadingExternal, setLoadingExternal] = useState(false)
	const [receiptDismissed, setReceiptDismissed] = useState(false)
	const [isSendingReceipt, setIsSendingReceipt] = useState(false)
	const [noReplyAction, setNoReplyAction] = useState<'reply' | 'replyAll' | null>(null)
	const [rawEml, setRawEml] = useState<string | null>(null)
	const [rawEmlLoading, setRawEmlLoading] = useState(false)
	const [sourceOpen, setSourceOpen] = useState(false)
	const [thread, setThread] = useState<ThreadViewType | null>(null)
	const [threadLoading, setThreadLoading] = useState(false)

	const { data, isLoading, error, refetch } = useQuery<MessageFull | null>({
		queryKey: ['message', accountId, mailbox, uid],
		queryFn: () =>
			invoke<MessageFull | null>('fetch_message_full', {
				accountId,
				mailbox,
				uid,
			}),
		staleTime: 5 * 60 * 1000,
		retry: 1,
	})

	// Load thread when threadViewEnabled
	useEffect(() => {
		if (!threadViewEnabled || !data) {
			setThread(null)
			return
		}

		setThreadLoading(true)
		invoke<ThreadViewType>('fetch_thread', { accountId, mailbox, uid })
			.then(setThread)
			.catch((err) => {
				console.error('Failed to load thread:', err)
				setThread(null)
			})
			.finally(() => setThreadLoading(false))
	}, [threadViewEnabled, data, accountId, mailbox, uid])

	// auto-mark as read - respects mark-as-read-delay setting
	// -1 = manual only, 0 = immediate, 2/5 = delayed seconds
	useEffect(() => {
		if (!data) return
		if (markAsReadDelay === -1) return

		const isUnread = !data.header.flags.includes('\\Seen')
		if (!isUnread) return

		const doMark = () => {
			invokeWithErrorLog(
				'mark_read',
				{ accountId, mailbox, uids: [uid], read: true },
				'mark_read'
			)
			queryClient.invalidateQueries({
				queryKey: ['messages', accountId, mailbox],
			})
		}

		if (markAsReadDelay === 0) {
			doMark()
			return
		}

		const timer = setTimeout(doMark, markAsReadDelay * 1000)
		return () => clearTimeout(timer)
	}, [data, accountId, mailbox, uid, queryClient, markAsReadDelay])

	const isNoReply = (email: string) => /^no-reply@/i.test(email) || /^noreply@/i.test(email)

	const guardComposing = (fn: () => void) => {
		if (useDraftStore.getState().isComposing) {
			toast.info('Please finish or discard current draft first')
			return
		}
		fn()
	}

	const doReply = () => {
		if (!data) return
		guardComposing(() => {
			useDraftStore.getState().startReply(accountId, data)
			window.dispatchEvent(new CustomEvent('compose:reply'))
		})
	}

	const doReplyAll = () => {
		if (!data) return
		guardComposing(() => {
			useDraftStore.getState().startReplyAll(accountId, data)
			window.dispatchEvent(new CustomEvent('compose:reply'))
		})
	}

	const handleReply = () => {
		if (!data) return
		const fromEmail = data.header.from?.[0] || ''
		if (isNoReply(fromEmail)) {
			setNoReplyAction('reply')
			return
		}
		doReply()
	}

	const handleReplyAll = () => {
		if (!data) return
		const fromEmail = data.header.from?.[0] || ''
		if (isNoReply(fromEmail)) {
			setNoReplyAction('replyAll')
			return
		}
		doReplyAll()
	}

	const handleForward = () => {
		if (!data) return
		guardComposing(() => {
			useDraftStore.getState().startForward(accountId, data)
			window.dispatchEvent(new CustomEvent('compose:forward'))
		})
	}

	const handleDelete = async () => {
		// Optimistic update
		onBack()

		try {
			await invokeWithErrorLog(
				'delete_messages',
				{ accountId, mailbox, uids: [uid] },
				'delete_messages'
			)
			queryClient.invalidateQueries({
				queryKey: ['messages', accountId, mailbox],
			})
			toast.success(t('inbox:messageView.deleted'))
		} catch (error) {
			toast.error(t('inbox:messageView.deleteError'))
		}
	}

	const handleMarkUnread = async () => {
		const result = await invokeWithErrorLog(
			'mark_read',
			{ accountId, mailbox, uids: [uid], read: false },
			'mark_unread'
		)
		if (result === null) {
			toast.error(t('inbox:messageView.markUnreadError'))
			return
		}
		queryClient.invalidateQueries({
			queryKey: ['messages', accountId, mailbox],
		})
		onBack()
	}

	const handleToggleStar = async () => {
		try {
			await invoke('toggle_starred', { accountId, mailbox, uid })
			queryClient.invalidateQueries({ queryKey: ['message', accountId, mailbox, uid] })
			queryClient.invalidateQueries({ queryKey: ['messages', accountId, mailbox] })
		} catch (error) {
			toast.error('Failed to toggle star', { description: String(error) })
		}
	}

	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key === 'Escape') {
				onBack()
			}
		}
		window.addEventListener('keydown', handleKeyDown)
		return () => window.removeEventListener('keydown', handleKeyDown)
	}, [onBack])

	const isComposing = useDraftStore((s) => s.isComposing)

	// Connect Gmail-style shortcuts
	useInboxShortcuts({
		onNextMessage: onNext || (() => {}),
		onPrevMessage: onPrev || (() => {}),
		onOpenMessage: () => {},
		onDeleteMessage: handleDelete,
		onReply: handleReply,
		onReplyAll: handleReplyAll,
		onForward: handleForward,
		onNewMessage: () => {}, // Sidebar/InboxScreen handles this
		onToggleRead: () => {},
		onMarkUnread: handleMarkUnread,
		onToggleStar: () => {},
		onFocusSearch: () => {
			const searchInput = document.querySelector('[data-search-input]') as HTMLElement
			searchInput?.focus()
		},
		enabled: !isComposing,
	})

	const scrollContainerRef = useRef<HTMLDivElement>(null)

	useEffect(() => {
		if (scrollContainerRef.current) {
			scrollContainerRef.current.scrollTo({ top: 0, behavior: 'auto' })
		}
		setLoading(true)
		setHasExternalResources(false)
		setLoadingExternal(false)
		setReceiptDismissed(false)
		setIsSendingReceipt(false)
		setAllowExternalResources(!blockExternalImages)
		setRawEml(null)
		setSourceOpen(false)
	}, [uid, setLoading, blockExternalImages])

	const handleViewSource = useCallback(async () => {
		setSourceOpen(true)
		if (rawEml !== null) return
		setRawEmlLoading(true)
		try {
			const eml = await invoke<string>('fetch_raw_eml_text', { accountId, mailbox, uid })
			setRawEml(eml)
		} catch {
			setRawEml('// Failed to load raw EML')
		} finally {
			setRawEmlLoading(false)
		}
	}, [accountId, mailbox, uid, rawEml])

	// Keep TitleBar in sync with current message subject + navigation
	useEffect(() => {
		if (!data) return
		setLoading(false)
		setTitleMeta({
			subject: data.header.subject || '',
			onNext: onNext,
			onPrev: onPrev,
		})
		return () => {
			setTitleMeta(null)
			setLoading(false)
		}
	}, [data, onNext, onPrev, setTitleMeta, setLoading])

	if (isLoading) {
		return <MessageViewSkeleton />
	}

	if (error) {
		return (
			<div className='flex h-full items-center justify-center'>
				<div className='flex flex-col items-center gap-3 text-red-400'>
					<div className='flex h-12 w-12 items-center justify-center rounded-full bg-red-500/10 ring-1 ring-red-500/20'>
						<AlertCircle className='h-5 w-5' />
					</div>
					<p className='text-sm font-medium'>{t('inbox:messageView.error')}</p>
					<button
						onClick={() => refetch()}
						className='rounded-lg px-4 py-1.5 text-xs font-medium text-[var(--text-secondary)] ring-1 ring-[var(--border-subtle)] transition-colors hover:bg-[var(--surface-hover)]'>
						{t('inbox:messageView.errorRetry')}
					</button>
				</div>
			</div>
		)
	}

	if (!data) {
		return (
			<div className='flex h-full items-center justify-center'>
				<div className='flex flex-col items-center gap-2 text-slate-500'>
					<Mail className='h-8 w-8' />
					<p className='text-sm'>{t('inbox:messageView.notFound')}</p>
					<button
						onClick={onBack}
						className='mt-1 text-xs text-[var(--text-secondary)] underline underline-offset-2 hover:text-[var(--text-primary)]'>
						{t('inbox:messageView.back')}
					</button>
				</div>
			</div>
		)
	}

	return (
		<div className='message-view-container flex h-full flex-col bg-[var(--surface-panel)]'>
			<MessageViewHeader
				onBack={onBack}
				onReply={handleReply}
				onReplyAll={handleReplyAll}
				onForward={handleForward}
				onDelete={handleDelete}
				onMarkUnread={handleMarkUnread}
				onToggleStar={handleToggleStar}
				isStarred={data?.header.starred ?? false}
				onViewSource={handleViewSource}
			/>

			<div
				ref={scrollContainerRef}
				className='message-view-body flex-1 overflow-y-auto'
				style={{ willChange: 'transform' }}>
				{threadViewEnabled && thread && thread.messages.length > 1 && !threadLoading ? (
					<ThreadView
						thread={thread}
						currentUid={uid}
						blockExternalImages={blockExternalImages}
						viewMode={viewMode}
					/>
				) : threadViewEnabled && threadLoading ? (
					<div className='flex h-full items-center justify-center'>
						<div className='text-sm text-[var(--text-secondary)]'>Loading thread…</div>
					</div>
				) : (
					<>
						<MessageViewMeta header={data.header} />

						{/* Read receipt request banner */}
						{data.read_receipt_to && !blockReadReceipts && !receiptDismissed && (
							<div className='mx-5 mb-2 flex items-center justify-between gap-2 rounded-lg bg-[var(--surface-active)] px-3 py-2 ring-1 ring-[var(--border-faint)]'>
								<div className='flex items-center gap-2'>
									<div className='h-1.5 w-1.5 shrink-0 rounded-full bg-sky-400 shadow-[0_0_8px_rgba(56,189,248,0.5)]' />
									<span className='text-[11px] font-medium tracking-tight text-[var(--text-tertiary)] uppercase'>
										{t('inbox:messageView.readReceipt.label')}
									</span>
								</div>
								<div className='flex items-center gap-3'>
									<button
										type='button'
										disabled={isSendingReceipt}
										onClick={async () => {
											setIsSendingReceipt(true)
											try {
												await invoke('send_read_receipt', {
													accountId,
													toAddress: data.read_receipt_to,
													originalMessageId:
														data.header.message_id ?? null,
													originalSubject: data.header.subject ?? null,
												})
												toast.success(
													t('inbox:messageView.readReceipt.sent')
												)
												setReceiptDismissed(true)
											} catch {
												toast.error(
													t('inbox:messageView.readReceipt.error')
												)
												setIsSendingReceipt(false)
											}
										}}
										className='text-[11px] font-medium text-sky-400 transition-colors hover:text-sky-300 disabled:opacity-50'>
										{isSendingReceipt
											? t('inbox:messageView.readReceipt.sending')
											: t('inbox:messageView.readReceipt.send')}
									</button>
									<button
										type='button'
										disabled={isSendingReceipt}
										onClick={() => setReceiptDismissed(true)}
										className='text-[11px] font-medium text-[var(--text-primary)] transition-colors hover:text-[var(--text-primary)] disabled:opacity-50'>
										{t('inbox:messageView.readReceipt.dismiss')}
									</button>
								</div>
							</div>
						)}

						{/* External resources banner — shown when Rust detected external URLs and user hasn't loaded them yet */}
						{hasExternalResources && !allowExternalResources && (
							<div className='mx-5 mb-2 flex items-center justify-between gap-2 rounded-lg bg-[var(--surface-active)] px-3 py-2 ring-1 ring-[var(--border-faint)]'>
								<div className='flex items-center gap-2'>
									<div className='h-1.5 w-1.5 shrink-0 rounded-full bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.5)]' />
									<span className='text-[11px] font-medium tracking-tight text-[var(--text-tertiary)] uppercase'>
										{t('inbox:messageView.cspBlocked.label')}
									</span>
								</div>
								<button
									type='button'
									disabled={loadingExternal}
									onClick={() => setAllowExternalResources(true)}
									className='text-[11px] font-medium text-[var(--text-primary)] transition-colors hover:text-[var(--text-primary)] disabled:opacity-50'>
									{loadingExternal
										? '…'
										: t('inbox:messageView.cspBlocked.allow')}
								</button>
							</div>
						)}

						<div className='border-t border-[var(--border-faint)]'>
							<MessageViewErrorBoundary
								onFallback={() => toggleViewMode()}
								title={t('inbox:messageView.renderError.title')}
								description={t('inbox:messageView.renderError.description')}
								fallbackText={t('inbox:messageView.renderError.fallback')}>
								<div className='relative'>
									{loadingExternal && (
										<div className='absolute bottom-8 left-1/2 z-10 -translate-x-1/2'>
											<div className='flex items-center gap-2 rounded-full bg-[var(--surface-panel)] px-3.5 py-2 shadow-lg ring-1 ring-[var(--border-faint)]'>
												<svg
													className='h-3.5 w-3.5 animate-spin text-[var(--text-tertiary)]'
													viewBox='0 0 24 24'
													fill='none'>
													<circle
														className='opacity-25'
														cx='12'
														cy='12'
														r='10'
														stroke='currentColor'
														strokeWidth='3'
													/>
													<path
														className='opacity-75'
														fill='currentColor'
														d='M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z'
													/>
												</svg>
												<span className='text-[11px] font-medium text-[var(--text-tertiary)]'>
													{t('inbox:messageView.loadingExternal')}
												</span>
											</div>
										</div>
									)}
									<MessageViewBody
										htmlContent={data.body_html_safe}
										plainContent={data.body_plain}
										viewMode={viewMode}
										allowExternalResources={allowExternalResources}
										inline_images={data.inline_images}
										onExternalDetected={() => setHasExternalResources(true)}
										onLoadingChange={(loading) => setLoadingExternal(loading)}
									/>
								</div>
							</MessageViewErrorBoundary>
							{data.attachments.length > 0 && (
								<MessageViewAttachments
									attachments={data.attachments}
									accountId={accountId}
									mailbox={mailbox}
									uid={uid}
								/>
							)}
						</div>
					</>
				)}
			</div>

			{/* Raw EML source viewer */}
			<Dialog open={sourceOpen} onOpenChange={setSourceOpen}>
				<DialogContent className='flex max-h-[80vh] w-full max-w-3xl flex-col gap-0 p-0'>
					<DialogHeader className='shrink-0 border-b border-[var(--border-faint)] px-5 py-4'>
						<DialogTitle className='font-mono text-sm'>
							{data?.header.subject || 'Raw EML'}
						</DialogTitle>
						<DialogDescription className='font-mono text-xs text-[var(--text-tertiary)]'>
							{accountId} / {mailbox} / uid:{uid}
						</DialogDescription>
					</DialogHeader>

					<div className='min-h-0 flex-1 overflow-y-auto'>
						<pre className='p-5 font-mono text-[11px] leading-relaxed break-all whitespace-pre-wrap text-[var(--text-secondary)]'>
							{rawEmlLoading ? 'Loading…' : (rawEml ?? '')}
						</pre>
					</div>

					<div className='flex shrink-0 items-center justify-end gap-2 border-t border-[var(--border-faint)] px-5 py-3'>
						<button
							type='button'
							onClick={async () => {
								if (!rawEml) return
								await navigator.clipboard.writeText(rawEml)
								toast.success('Copied to clipboard')
							}}
							disabled={!rawEml}
							className='rounded-md px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)] ring-1 ring-[var(--border-subtle)] transition-colors hover:bg-[var(--surface-hover)] disabled:opacity-40'>
							Copy
						</button>
						<button
							type='button'
							onClick={() => setSourceOpen(false)}
							className='rounded-md px-3 py-1.5 text-xs font-medium text-[var(--text-primary)] ring-1 ring-[var(--border-subtle)] transition-colors hover:bg-[var(--surface-hover)]'>
							Close
						</button>
					</div>
				</DialogContent>
			</Dialog>

			{/* No-reply warning dialog */}
			<Dialog
				open={noReplyAction !== null}
				onOpenChange={(o) => !o && setNoReplyAction(null)}>
				<DialogContent className='sm:max-w-sm'>
					<DialogHeader>
						<DialogTitle>{t('inbox:messageView.noReply.title')}</DialogTitle>
						<DialogDescription>
							{t('inbox:messageView.noReply.description')}
						</DialogDescription>
					</DialogHeader>
					<DialogFooter className='gap-2'>
						<Button variant='ghost' onClick={() => setNoReplyAction(null)}>
							{t('inbox:messageView.noReply.cancel')}
						</Button>
						<Button
							variant='default'
							onClick={() => {
								if (noReplyAction === 'reply') doReply()
								else doReplyAll()
								setNoReplyAction(null)
							}}>
							{t('inbox:messageView.noReply.confirm')}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</div>
	)
}
