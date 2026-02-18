
import { useEffect } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { AlertCircle, Mail } from 'lucide-react'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
import { useMessageViewStore } from '@/stores/messageViewStore'
import type { MessageFull } from '@/types/mail'
import { MessageViewHeader } from './MessageViewHeader'
import { MessageViewMeta } from './MessageViewMeta'
import { MessageViewBody } from './MessageViewBody'
import { toast } from '@/stores/toastStore'

interface MessageViewProps {
	accountId: string
	mailbox: string
	uid: number
	onBack: () => void
}

export const MessageView = ({
	accountId,
	mailbox,
	uid,
	onBack,
}: MessageViewProps) => {
	const { t } = useTypedTranslation(['common', 'inbox'])
	const accentColor = useThemeStore((s) => s.accentColor)
	const queryClient = useQueryClient()
	const { viewMode, toggleViewMode } = useMessageViewStore()

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

	const handleReply = () => {
		console.warn('Reply not implemented')
		toast.info('Reply not implemented yet')
	}

	const handleReplyAll = () => {
		console.warn('Reply All not implemented')
		toast.info('Reply All not implemented yet')
	}

	const handleForward = () => {
		console.warn('Forward not implemented')
		toast.info('Forward not implemented yet')
	}

	const handleDelete = async () => {
		try {
			await invoke('delete_messages', {
				accountId,
				mailbox,
				uids: [uid],
			})
			queryClient.invalidateQueries({
				queryKey: ['messages', accountId, mailbox],
			})
			onBack()
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

	if (isLoading) {
		return (
			<div className='flex h-full items-center justify-center'>
				<div className='flex flex-col items-center gap-3'>
					<div className='relative h-10 w-10'>
						<div
							className='absolute inset-0 animate-spin rounded-full border-2 border-transparent'
							style={{ borderTopColor: accentColor }}
						/>
						<div
							className='absolute inset-1 animate-spin rounded-full border-2 border-transparent'
							style={{
								borderBottomColor: `rgba(var(--accent-rgb), 0.3)`,
								animationDirection: 'reverse',
								animationDuration: '1.5s',
							}}
						/>
					</div>
					<span className='text-sm text-slate-500'>
						{t('inbox:messageView.loading')}
					</span>
				</div>
			</div>
		)
	}

	if (error) {
		return (
			<div className='flex h-full items-center justify-center'>
				<div className='flex flex-col items-center gap-3 text-red-400'>
					<div className='flex h-12 w-12 items-center justify-center rounded-full bg-red-500/10 ring-1 ring-red-500/20'>
						<AlertCircle className='h-5 w-5' />
					</div>
					<p className='text-sm font-medium'>
						{t('inbox:messageView.error')}
					</p>
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
				viewMode={viewMode}
				onToggleViewMode={toggleViewMode}
			/>

			<div className='message-view-body flex-1 overflow-y-auto px-6 py-4'>
				<MessageViewMeta header={data.header} />
				<div className='mt-6'>
					<MessageViewBody
						htmlContent={data.body_html_safe}
						plainContent={data.body_plain}
						viewMode={viewMode}
					/>
				</div>
			</div>
		</div>
	)
}
