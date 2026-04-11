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
import type { FilterRule } from '@/types/filters'

import { useAccountStore } from '@/stores/accountStore'
import { SuggestRuleDialog } from '@/components/Inbox/SuggestRuleDialog'
import { useAdvancedSearch } from '@/hooks/useAdvancedSearch'
import { SearchResultsList } from '@/components/Inbox/Search/SearchResultsList'
import type { AdvancedSearchQuery } from '@/types/search'

export const InboxScreen = (props: InboxScreenProps) => {
	const [suggestedRules, setSuggestedRules] = useState<FilterRule[]>([])
	const activeAccount = useAccountStore((s) => s.activeAccount)

	useEffect(() => {
		const handleSuggest = (e: Event) => {
			const customEvent = e as CustomEvent<FilterRule[]>
			setSuggestedRules(customEvent.detail)
		}
		window.addEventListener('postail:suggestRules', handleSuggest)
		return () => window.removeEventListener('postail:suggestRules', handleSuggest)
	}, [])

	return (
		<>
			<InboxScreenInner {...props} />
			{suggestedRules.length > 0 && activeAccount && (
				<SuggestRuleDialog
					rules={suggestedRules}
					accountId={activeAccount.id}
					onClose={() => setSuggestedRules([])}
				/>
			)}
		</>
	)
}

const InboxScreenInner = ({}: InboxScreenProps) => {
	const accounts = useAccountStore((s) => s.accounts)
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const setActiveAccount = useAccountStore((s) => s.setActiveAccount)
	const activeMailbox = useAccountStore((s) => s.activeMailbox)
	const setActiveMailbox = useAccountStore((s) => s.setActiveMailbox)
	const [isComposeOpen, setIsComposeOpen] = useState(false)
	const [selectedMessage, setSelectedMessage] = useState<{
		uid: number
		mailbox: string
	} | null>(null)
	const [focusedUid, setFocusedUid] = useState<number | null>(null)
	const { loadDraft } = useDraftStore()
	const openMessageInStore = useMessageViewStore((s) => s.openMessage)
	const closeMessageInStore = useMessageViewStore((s) => s.closeMessage)
	const queryClient = useQueryClient()

	const {
		results,
		isLoading: searchLoading,
		error: searchError,
		displayQueryString,
		isActive: isSearchActive,
		search,
		clear: clearSearch,
	} = useAdvancedSearch(activeAccount?.id, activeMailbox)

	// Listen for search events from SearchBar
	useEffect(() => {
		const handleSearch = (e: Event) => {
			const query = (e as CustomEvent<AdvancedSearchQuery | null>).detail
			if (!query) {
				clearSearch()
			} else {
				search(query)
			}
		}
		window.addEventListener('postail:search', handleSearch)
		return () => window.removeEventListener('postail:search', handleSearch)
	}, [search, clearSearch])

	// Listen for saved search activation from SearchBar
	useEffect(() => {
		const handleActivateSavedSearch = (e: Event) => {
			const { query } = (
				e as CustomEvent<{ id: string; name: string; query: AdvancedSearchQuery }>
			).detail
			search(query)
		}
		window.addEventListener('postail:activateSavedSearch', handleActivateSavedSearch)
		return () =>
			window.removeEventListener('postail:activateSavedSearch', handleActivateSavedSearch)
	}, [search])

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
				setSelectedMessage({ uid: newMessage.uid, mailbox: newMessage.mailbox })
				openMessageInStore(activeAccount!.id, newMessage.mailbox, newMessage.uid)
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
			const messages = getMessagesList()
			const msg = messages.find((m) => m.uid === focusedUid)
			if (msg) {
				setSelectedMessage({ uid: focusedUid, mailbox: msg.mailbox })
				openMessageInStore(activeAccount!.id, msg.mailbox, focusedUid)
			}
		}
	}, [focusedUid, selectedMessage, activeAccount, getMessagesList, openMessageInStore])

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
	const handleArchive = useCallback(async () => {
		// archive requires knowing selected message – no-op at list level;
		// MessageView wires its own handler via useInboxShortcuts
	}, [])

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
		onArchive: handleArchive,
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
			<div className='text-muted-foreground flex h-full items-center justify-center'>
				No accounts configured.
			</div>
		)
	}

	return (
		<>
			<div className='flex h-full overflow-hidden'>
				<Sidebar
					activeAccount={activeAccount}
					activeMailbox={isSearchActive ? '' : activeMailbox}
					onMailboxSelect={(mailbox) => {
						if (isSearchActive) {
							clearSearch()
							window.dispatchEvent(new CustomEvent('postail:search:clear'))
						}
						setActiveMailbox(mailbox)
					}}
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
					) : isSearchActive ? (
						<SearchResultsList
							results={results}
							isLoading={searchLoading}
							error={searchError}
							query={displayQueryString}
							onMessageClick={(uid: number, mailbox: string) => {
								setSelectedMessage({ uid, mailbox })
								openMessageInStore(activeAccount!.id, mailbox, uid)
								setFocusedUid(uid)
							}}
						/>
					) : (
						<MessageList
							account={activeAccount}
							mailbox={activeMailbox}
							focusedUid={focusedUid}
							onMessageClick={(uid: number, mailbox: string) => {
								setSelectedMessage({ uid, mailbox })
								openMessageInStore(activeAccount!.id, mailbox, uid)
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
