import { useEffect } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { AlertCircle, Mail } from 'lucide-react'
import { motion } from 'framer-motion'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import type { MessageFull } from '@/types/mail'

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
	const animationsEnabled = useAnimationsEnabled()
	const queryClient = useQueryClient()

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

	// placeholder
	return (
		<div className='message-view-container'>
			<div className='p-6'>
				<button
					onClick={onBack}
					className='mb-4 text-sm text-slate-400 hover:text-slate-200'>
					← {t('inbox:messageView.back')}
				</button>
				<h2 className='text-lg font-semibold text-slate-100'>
					{data.header.subject || t('inbox:messageView.noSubject')}
				</h2>
				<p className='mt-1 text-sm text-slate-400'>
					{data.header.from[0]}
				</p>
				<div className='mt-4 rounded-lg bg-white/[0.02] p-4 text-sm text-slate-300'>
					{data.body_plain
						? data.body_plain.slice(0, 500)
						: 'No content'}
					{data.body_plain && data.body_plain.length > 500 && '...'}
				</div>
			</div>
		</div>
	)
}
