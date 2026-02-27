import { useState, useEffect, useRef } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
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
import { useDraftStore } from '@/stores/draftStore'
import type { MessageFull } from '@/types/mail'
import { MessageViewHeader } from './MessageViewHeader'
import { MessageViewMeta } from './MessageViewMeta'
import { MessageViewBody } from './MessageViewBody'
import { MessageViewAttachments } from './MessageViewAttachments'
import { MessageViewErrorBoundary } from './MessageViewErrorBoundary'
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
	// viewMode passed to MessageViewBody below
	const [allowExternalResources, setAllowExternalResources] = useState(false)
	const [cspBlocked, setCspBlocked] = useState(false)
	const [noReplyAction, setNoReplyAction] = useState<'reply' | 'replyAll' | null>(null)

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

	// auto-mark as read
	useEffect(() => {
		if (!data) return
		const isUnread = !data.header.flags.includes('\\Seen')
		if (isUnread) {
			invoke('mark_read', {
				accountId,
				mailbox,
				uids: [uid],
				read: true,
			}).catch((err) => console.error('Failed to mark as read:', err))

			queryClient.invalidateQueries({
				queryKey: ['messages', accountId, mailbox],
			})
		}
	}, [data, accountId, mailbox, uid, queryClient])

	const isNoReply = (email: string) => /^no-reply@/i.test(email) || /^noreply@/i.test(email)

	const doReply = () => {
		if (!data) return
		const { startReply, isComposing } = useDraftStore.getState()
		if (isComposing) {
			toast.info('Please finish or discard current draft first')
			return
		}
		startReply(accountId, data)
		window.dispatchEvent(new CustomEvent('compose:reply'))
	}

	const doReplyAll = () => {
		if (!data) return
		const { startReplyAll, isComposing } = useDraftStore.getState()
		if (isComposing) {
			toast.info('Please finish or discard current draft first')
			return
		}
		startReplyAll(accountId, data)
		window.dispatchEvent(new CustomEvent('compose:reply'))
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
		const { startForward, isComposing } = useDraftStore.getState()
		if (isComposing) {
			toast.info('Please finish or discard current draft first')
			return
		}
		startForward(accountId, data)
		window.dispatchEvent(new CustomEvent('compose:forward'))
	}

	const handleDelete = async () => {
		// Optimistic update
		onBack()

		try {
			await invoke('delete_messages', {
				accountId,
				mailbox,
				uids: [uid],
			})
			queryClient.invalidateQueries({
				queryKey: ['messages', accountId, mailbox],
			})
			toast.success(t('inbox:messageView.deleted'))
		} catch (error) {
			console.error('Failed to delete message:', error)
			toast.error(t('inbox:messageView.deleteError'))
		}
	}

	const handleMarkUnread = async () => {
		try {
			await invoke('mark_read', {
				accountId,
				mailbox,
				uids: [uid],
				read: false,
			})
			queryClient.invalidateQueries({
				queryKey: ['messages', accountId, mailbox],
			})
			onBack()
		} catch (error) {
			console.error('Failed to mark as unread:', error)
			toast.error(t('inbox:messageView.markUnreadError'))
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
		setCspBlocked(false)
		setAllowExternalResources(false)
	}, [uid, setLoading])

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
						className='rounded-lg px-4 py-1.5 text-xs font-medium text-slate-300 ring-1 ring-white/[0.08] transition-colors hover:bg-white/[0.04]'>
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
						className='mt-1 text-xs text-slate-400 underline underline-offset-2 hover:text-slate-300'>
						{t('inbox:messageView.back')}
					</button>
				</div>
			</div>
		)
	}

	return (
		<div className='message-view-container flex h-full flex-col bg-slate-900'>
			<MessageViewHeader
				onBack={onBack}
				onReply={handleReply}
				onReplyAll={handleReplyAll}
				onForward={handleForward}
				onDelete={handleDelete}
				onMarkUnread={handleMarkUnread}
			/>

			<div ref={scrollContainerRef} className='message-view-body flex-1 overflow-y-auto'>
				<MessageViewMeta header={data.header} />

				{/* CSP-triggered banner - appears only when browser actually blocked something */}
				{cspBlocked && !allowExternalResources && (
					<div className='mx-5 mb-2 flex items-center justify-between gap-2 rounded-lg bg-zinc-900/50 px-3 py-2 ring-1 ring-white/[0.04]'>
						<div className='flex items-center gap-2'>
							<div className='h-1.5 w-1.5 shrink-0 rounded-full bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.5)]' />
							<span className='text-[11px] font-medium tracking-tight text-zinc-500 uppercase'>
								{t('inbox:messageView.cspBlocked.label')}
							</span>
						</div>
						<button
							type='button'
							onClick={() => setAllowExternalResources(true)}
							className='text-[11px] font-medium text-zinc-300 transition-colors hover:text-white'>
							{t('inbox:messageView.cspBlocked.allow')}
						</button>
					</div>
				)}

				<div className='border-t border-white/[0.04]'>
					<MessageViewErrorBoundary
						onFallback={() => toggleViewMode()}
						title={t('inbox:messageView.renderError.title')}
						description={t('inbox:messageView.renderError.description')}
						fallbackText={t('inbox:messageView.renderError.fallback')}>
						<MessageViewBody
							htmlContent={data.body_html_safe}
							plainContent={data.body_plain}
							viewMode={viewMode}
							allowExternalResources={allowExternalResources}
							inline_images={data.inline_images}
							onCspBlocked={() => setCspBlocked(true)}
						/>
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
			</div>
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
