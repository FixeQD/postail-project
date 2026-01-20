import { useRef, useCallback, useState } from 'react'
import { Virtuoso, type VirtuosoHandle } from 'react-virtuoso'
import { useInfiniteQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { format, isToday, isYesterday, isThisYear } from 'date-fns'
import { Star, Archive, Trash2, MailOpen, Mail } from 'lucide-react'
import type { MailHeader } from '../../types/mail'
import type { AccountMeta } from '../../types/accounts'
import { useTypedTranslation } from '../../hooks/useTypedTranslation'

interface MessageListProps {
	account: AccountMeta
	mailbox: string
	onMessageClick: (messageId: number) => void
}

const BATCH_SIZE = 50

export const MessageList = ({ account, mailbox, onMessageClick }: MessageListProps) => {
    const { t } = useTypedTranslation()
	const virtuosoRef = useRef<VirtuosoHandle>(null)
    const [hoveredMessageId, setHoveredMessageId] = useState<number | null>(null)

	const {
		data,
		fetchNextPage,
		hasNextPage,
		isFetchingNextPage,
		isLoading,
        error
	} = useInfiniteQuery({
		queryKey: ['messages', account.id, mailbox],
		queryFn: async ({ pageParam }) => {
			const anchor = pageParam as number | undefined
			const messages = await invoke<MailHeader[]>('fetch_headers', {
				accountId: account.id,
				mailbox,
				anchor,
				limit: BATCH_SIZE,
			})
			return messages
		},
		initialPageParam: undefined as number | undefined,
		getNextPageParam: (lastPage: MailHeader[]) => {
			if (lastPage.length < BATCH_SIZE) return undefined
			const lastMessage = lastPage[lastPage.length - 1]
			return lastMessage?.uid
		},
	})
    
	const allMessages = data?.pages.flatMap((page) => page) ?? []

    const formatDate = (dateString: string) => {
        const date = new Date(dateString)
        if (isToday(date)) return format(date, 'HH:mm')
        if (isYesterday(date)) return t('inbox:messageList.date.yesterday')
        if (isThisYear(date)) return format(date, 'MMM d')
        return format(date, 'dd/MM/yyyy')
    }

	const loadMore = useCallback(() => {
        if (hasNextPage && !isFetchingNextPage) {
            fetchNextPage()
        }
	}, [hasNextPage, isFetchingNextPage, fetchNextPage])

    if (isLoading) {
        return (
            <div className="flex h-full items-center justify-center text-slate-500">
                <div className="h-8 w-8 animate-spin rounded-full border-2 border-slate-600 border-t-orange-500" />
            </div>
        )
    }

    if (error) {
        return (
             <div className="flex h-full items-center justify-center text-red-400">
                {t('inbox:messageList.error')}
             </div>
        )
    }

    if (allMessages.length === 0) {
        return (
            <div className="flex h-full flex-col items-center justify-center text-slate-500">
                <div className="flex h-24 w-24 items-center justify-center rounded-full bg-slate-900/50">
                    <Mail className="h-10 w-10 opacity-20" />
                </div>
                <p className="mt-4 font-medium">{t('inbox:messageList.empty.title')}</p>
                <p className="text-sm opacity-50">{t('inbox:messageList.empty.subtitle')}</p>
            </div>
        )
    }

	return (
		<div className='flex h-full flex-1 flex-col bg-slate-950'>
			<Virtuoso
				ref={virtuosoRef}
				data={allMessages}
				endReached={loadMore}
				overscan={200}
				itemContent={(_index, message) => {
                    const isUnread = !message.flags.includes('\\Seen')
                    const isHovered = hoveredMessageId === message.uid

                    return (
                        <div
                            onMouseEnter={() => setHoveredMessageId(message.uid)}
                            onMouseLeave={() => setHoveredMessageId(null)}
                            onClick={() => onMessageClick(message.uid)}
                            className={`group relative flex cursor-pointer items-center border-b border-slate-900 px-4 py-3 transition-colors ${
                                isUnread ? 'bg-slate-900/40 hover:bg-slate-900' : 'bg-transparent hover:bg-slate-900/50'
                            }`}
                        >
                            {/* Checkbox & Star */}
                            <div className="flex items-center gap-3 pr-4">
                                <input 
                                    type="checkbox" 
                                    className="h-4 w-4 rounded border-slate-700 bg-slate-950/50 text-orange-500 focus:ring-orange-500/20"
                                    onClick={(e) => e.stopPropagation()}
                                />
                                <button 
                                    className="text-slate-600 hover:text-yellow-500 focus:outline-none"
                                    onClick={(e) => e.stopPropagation()}
                                >
                                    <Star className="h-4 w-4" />
                                </button>
                            </div>

                            {/* Main Content */}
                            <div className="flex min-w-0 flex-1 items-baseline gap-4">
                                {/* Sender */}
                                <div className={`w-48 shrink-0 truncate text-sm ${isUnread ? 'font-bold text-white' : 'font-medium text-slate-300'}`}>
                                    {message.from[0]?.replace(/<.*>/, '').trim() || message.from.join(', ')}
                                </div>

                                {/* Subject & Snippet */}
                                <div className="flex min-w-0 flex-1 items-baseline gap-2">
                                    <span className={`truncate text-sm ${isUnread ? 'font-semibold text-slate-200' : 'text-slate-400'}`}>
                                        {message.subject || '(No Subject)'}
                                    </span>
                                    <span className="truncate text-xs text-slate-500">
                                        - {message.snippet}
                                    </span>
                                </div>
                            </div>

                            {/* Date or Actions */}
                            <div className="ml-4 flex w-24 shrink-0 justify-end">
                                {isHovered ? (
                                    <div className="flex items-center gap-1">
                                        <ActionBtn icon={<Archive className="h-4 w-4" />} tooltip={t('inbox:messageList.actions.archive')} />
                                        <ActionBtn icon={<Trash2 className="h-4 w-4" />} tooltip={t('inbox:messageList.actions.delete')} />
                                        <ActionBtn icon={isUnread ? <MailOpen className="h-4 w-4" /> : <Mail className="h-4 w-4" />} tooltip={isUnread ? t('inbox:messageList.actions.markRead') : t('inbox:messageList.actions.markUnread')} />
                                    </div>
                                ) : (
                                    <span className={`text-xs ${isUnread ? 'font-medium text-slate-300' : 'text-slate-500'}`}>
                                        {formatDate(message.internal_date)}
                                    </span>
                                )}
                            </div>
                        </div>
                    )
                }}
			/>
		</div>
	)
}

const ActionBtn = ({ icon, tooltip }: { icon: React.ReactNode, tooltip: string }) => (
    <button 
        className="rounded p-1.5 text-slate-400 hover:bg-slate-800 hover:text-slate-200"
        title={tooltip}
        onClick={(e) => e.stopPropagation()}
    >
        {icon}
    </button>
)
