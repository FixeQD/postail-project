import { useRef, useCallback } from 'react'
import { Virtuoso, type VirtuosoHandle } from 'react-virtuoso'
import { useInfiniteQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { format, isToday, isYesterday, isThisYear } from 'date-fns'
import { motion } from 'framer-motion'
import { Paperclip, Mail } from 'lucide-react'
import type { MailHeader } from '../../types/mail'
import type { AccountMeta } from '../../types/accounts'

interface MessageListProps {
	account: AccountMeta
	mailbox: string
	onMessageClick: (messageId: number) => void
}

const BATCH_SIZE = 50

export const MessageList = ({ account, mailbox, onMessageClick }: MessageListProps) => {
	const virtuosoRef = useRef<VirtuosoHandle>(null)

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
				anchor, // Use last UID as anchor
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
    
    // Flatten pages into a single array
	const allMessages = data?.pages.flatMap((page) => page) ?? []

    const formatDate = (dateString: string) => {
        const date = new Date(dateString)
        if (isToday(date)) return format(date, 'HH:mm')
        if (isYesterday(date)) return 'Yesterday'
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
                Failed to load messages
             </div>
        )
    }

    if (allMessages.length === 0) {
        return (
            <div className="flex h-full flex-col items-center justify-center text-slate-500">
                <Mail className="mb-4 h-12 w-12 opacity-20" />
                <p>No messages in {mailbox}</p>
            </div>
        )
    }

	return (
		<div className='h-full flex-1'>
			<Virtuoso
				ref={virtuosoRef}
				data={allMessages}
				endReached={loadMore}
				overscan={200}
				itemContent={(_index, message) => (
					<motion.div
                        initial={{ opacity: 0, y: 5 }}
                        animate={{ opacity: 1, y: 0 }}
						onClick={() => onMessageClick(message.uid)}
						className={`group flex cursor-pointer items-center border-b border-slate-800/50 bg-slate-950/30 px-4 py-3 transition-colors hover:bg-slate-900 ${
                            !message.flags.includes('\\Seen') ? 'bg-slate-900 font-semibold' : ''
                        }`}
					>
                        {/* Avatar/Selection placeholder */}
                        <div className="mr-4 flex h-8 w-8 items-center justify-center rounded-full bg-slate-800 text-xs font-bold text-slate-400 group-hover:bg-slate-700 group-hover:text-slate-200">
                            {message.from[0]?.charAt(0).toUpperCase() || '?'}
                        </div>

						<div className='min-w-0 flex-1 pr-4'>
							<div className='flex items-center justify-between'>
								<span className={`truncate text-sm ${!message.flags.includes('\\Seen') ? 'text-slate-100' : 'text-slate-300'}`}>
                                    {message.from.join(', ')}
                                </span>
								<span className={`text-xs ${!message.flags.includes('\\Seen') ? 'text-orange-400' : 'text-slate-500'}`}>
                                    {formatDate(message.internal_date)}
                                </span>
							</div>
							<div className='flex items-center justify-between'>
                                <span className={`truncate text-sm ${!message.flags.includes('\\Seen') ? 'text-slate-200' : 'text-slate-400'}`}>
                                    {message.subject || '(No Subject)'}
                                </span>
                                {message.has_attachments && <Paperclip className="ml-2 h-3 w-3 text-slate-500" />}
                            </div>
							<p className='truncate text-xs text-slate-500'>
                                {message.snippet}
                            </p>
						</div>
					</motion.div>
				)}
                components={{
                    Footer: () => isFetchingNextPage ? (
                        <div className="flex justify-center py-4">
                             <div className="h-5 w-5 animate-spin rounded-full border-2 border-slate-600 border-t-orange-500" />
                        </div>
                    ) : null
                }}
			/>
		</div>
	)
}
