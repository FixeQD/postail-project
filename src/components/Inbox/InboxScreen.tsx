import { useState, useEffect, useCallback } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { Sidebar } from '../Layout/Sidebar'
import { MessageList } from './MessageList'
import { MessageView } from './MessageView'
import { DraftsList } from './DraftsList'
import { ComposeScreen } from '../Compose/ComposeScreen'
import { useDraftStore } from '@/stores/draftStore'
import { useInboxShortcuts } from '@/hooks/useInboxShortcuts'
import { useMessageViewStore } from '@/stores/messageViewStore'
import type { ComposeDraft } from '@/types/compose'
import type { MailHeader } from '@/types/mail'
import type { InboxScreenProps } from '@/types/components/inbox'

import { useAccountStore } from '@/stores/accountStore'

export const InboxScreen = ({}: InboxScreenProps) => {
	const { accounts, activeAccount, setActiveAccount, activeMailbox, setActiveMailbox } =
		useAccountStore()
	const [isComposeOpen, setIsComposeOpen] = useState(false)
	const [selectedMessage, setSelectedMessage] = useState<{
		uid: number
		mailbox: string
	} | null>(null)
	const [focusedUid, setFocusedUid] = useState<number | null>(null)
	const { loadDraft } = useDraftStore()
	const { openMessage: openMessageInStore, closeMessage: closeMessageInStore } =
		useMessageViewStore()
	const queryClient = useQueryClient()

	const getMessagesList = useCallback(() => {
		if (!activeAccount || !activeMailbox) return []
		const data = queryClient.getQueryData<{ pages: MailHeader[][] }>([
			'messages',
			activeAccount.id,
			activeMailbox,
		])
		return data?.pages.flatMap((page) => page) || []
	}, [activeAccount, activeMailbox, queryClient])

	const navigateMessage = useCallback(
		(direction: 'next' | 'prev') => {
			const messages = getMessagesList()
			if (messages.length === 0) return

			const currentUid = selectedMessage?.uid ?? focusedUid
			const currentIndex = currentUid ? messages.findIndex((m) => m.uid === currentUid) : -1

			let newIndex = -1
			if (direction === 'next') {
				newIndex = currentIndex + 1
				if (newIndex >= messages.length) return
			} else {
				newIndex = currentIndex === -1 ? 0 : currentIndex - 1
				if (newIndex < 0) return
			}

			const newMessage = messages[newIndex]
			if (!newMessage) return

			if (selectedMessage) {
				setSelectedMessage({ uid: newMessage.uid, mailbox: activeMailbox! })
				openMessageInStore(activeAccount!.id, activeMailbox!, newMessage.uid)
			} else {
				setFocusedUid(newMessage.uid)
			}
		},
		[getMessagesList, selectedMessage, focusedUid, activeMailbox]
	)

	const canGoNext = useCallback(() => {
		const messages = getMessagesList()
		if (!messages.length) return false
		const idx = messages.findIndex((m) => m.uid === selectedMessage?.uid)
		return idx !== -1 && idx < messages.length - 1
	}, [getMessagesList, selectedMessage])

	const canGoPrev = useCallback(() => {
		const messages = getMessagesList()
		if (!messages.length) return false
		const idx = messages.findIndex((m) => m.uid === selectedMessage?.uid)
		return idx > 0
	}, [getMessagesList, selectedMessage])

	const handleNextMessage = useCallback(() => {
		navigateMessage('next')
	}, [navigateMessage])

	const handlePrevMessage = useCallback(() => {
		navigateMessage('prev')
	}, [navigateMessage])

	const handleOpenMessage = useCallback(() => {
		if (focusedUid && !selectedMessage) {
			setSelectedMessage({ uid: focusedUid, mailbox: activeMailbox! })
			openMessageInStore(activeAccount!.id, activeMailbox!, focusedUid)
		}
	}, [focusedUid, selectedMessage, activeMailbox])

	const handleDeleteMessage = useCallback(() => {
		// handle delete message
	}, [])

	const handleReply = useCallback(() => {
		// handle reply
	}, [])

	const handleReplyAll = useCallback(() => {
		// handle reply all
	}, [])

	const handleForward = useCallback(() => {
		// handle forward
	}, [])

	const handleNewMessage = useCallback(() => {
		setIsComposeOpen(true)
	}, [])

	const handleToggleRead = useCallback(() => {
		// handle toggle read
	}, [])

	const handleMarkUnread = useCallback(() => {
		// handle mark unread
	}, [])

	const handleToggleStar = useCallback(() => {
		// handle toggle star
	}, [])

	const handleFocusSearch = useCallback(() => {
		const searchInput = document.querySelector('[data-search-input]') as HTMLElement
		searchInput?.focus()
	}, [])

	// Register them
	useInboxShortcuts({
		onNextMessage: handleNextMessage,
		onPrevMessage: handlePrevMessage,
		onOpenMessage: handleOpenMessage,
		onDeleteMessage: handleDeleteMessage,
		onReply: handleReply,
		onReplyAll: handleReplyAll,
		onForward: handleForward,
		onNewMessage: handleNewMessage,
		onToggleRead: handleToggleRead,
		onMarkUnread: handleMarkUnread,
		onToggleStar: handleToggleStar,
		onFocusSearch: handleFocusSearch,
		enabled: !isComposeOpen,
	})

	// Listen for global new message shortcut
	useEffect(() => {
		const handleNewMessage = () => {
			setIsComposeOpen(true)
		}
		window.addEventListener('compose:new', handleNewMessage)
		return () => window.removeEventListener('compose:new', handleNewMessage)
	}, [])

	// Listen for reply events
	useEffect(() => {
		const handleReply = () => {
			setIsComposeOpen(true)
		}
		window.addEventListener('compose:reply', handleReply)
		return () => window.removeEventListener('compose:reply', handleReply)
	}, [])

	// Listen for forward events
	useEffect(() => {
		const handleForwardEvent = () => {
			setIsComposeOpen(true)
		}
		window.addEventListener('compose:forward', handleForwardEvent)
		return () => window.removeEventListener('compose:forward', handleForwardEvent)
	}, [])

	useEffect(() => {
		if (!activeAccount && accounts.length > 0) {
			setActiveAccount(accounts[0])
		}
	}, [accounts, activeAccount, setActiveAccount])

	// reset selected message when mailbox changes
	useEffect(() => {
		setSelectedMessage(null)
		closeMessageInStore()
	}, [activeMailbox])

	if (!activeAccount) {
		return (
			<div className='flex h-full items-center justify-center text-slate-400'>
				No accounts configured.
			</div>
		)
	}

	return (
		<>
			<div className='flex h-full overflow-hidden'>
				<Sidebar
					activeAccount={activeAccount}
					activeMailbox={activeMailbox}
					onMailboxSelect={setActiveMailbox}
					onCompose={() => setIsComposeOpen(true)}
				/>
				<div className='flex flex-1 flex-col overflow-hidden'>
					{selectedMessage ? (
						<MessageView
							accountId={activeAccount.id}
							mailbox={selectedMessage.mailbox}
							uid={selectedMessage.uid}
							onBack={() => {
								setSelectedMessage(null)
								closeMessageInStore()
							}}
							onNext={canGoNext() ? handleNextMessage : undefined}
							onPrev={canGoPrev() ? handlePrevMessage : undefined}
						/>
					) : activeMailbox === 'Drafts' ? (
						<DraftsList
							accountId={activeAccount.id}
							onDraftClick={(draft: ComposeDraft) => {
								loadDraft(draft)
								setIsComposeOpen(true)
							}}
						/>
					) : (
						<MessageList
							account={activeAccount}
							mailbox={activeMailbox}
							focusedUid={focusedUid}
							onMessageClick={(uid: number) => {
								setSelectedMessage({ uid, mailbox: activeMailbox })
								openMessageInStore(activeAccount!.id, activeMailbox!, uid)
								setFocusedUid(uid)
							}}
						/>
					)}
				</div>
			</div>
			<ComposeScreen
				open={isComposeOpen}
				onOpenChange={setIsComposeOpen}
				accountId={activeAccount?.id}
			/>
		</>
	)
}
