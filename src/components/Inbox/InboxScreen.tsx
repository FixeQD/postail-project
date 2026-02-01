import { useState, useEffect, useCallback } from 'react'
import { Sidebar } from '../Layout/Sidebar'
import { MessageList } from './MessageList'
import { DraftsList } from './DraftsList'
import { ComposeScreen } from '../Compose/ComposeScreen'
import { useDraftStore } from '@/stores/draftStore'
import { useInboxShortcuts } from '@/hooks/useInboxShortcuts'
import type { AccountMeta } from '../../types/accounts'
import type { ComposeDraft } from '../../types/compose'

interface InboxScreenProps {
	accounts: AccountMeta[]
	activeAccount: AccountMeta | null
	setActiveAccount: (account: AccountMeta) => void
	activeMailbox: string
	setActiveMailbox: (mailbox: string) => void
	onOpenSettings: () => void
}

export const InboxScreen = ({
	accounts,
	activeAccount,
	setActiveAccount,
	activeMailbox,
	setActiveMailbox,
}: InboxScreenProps) => {
	const [isComposeOpen, setIsComposeOpen] = useState(false)
	const { loadDraft } = useDraftStore()

	const handleNextMessage = useCallback(() => {
		console.log('Next message (J)')
	}, [])

	const handlePrevMessage = useCallback(() => {
		console.log('Previous message (K)')
	}, [])

	const handleOpenMessage = useCallback(() => {
		console.log('Open message (Enter)')
	}, [])

	const handleDeleteMessage = useCallback(() => {
		console.log('Delete message (Del/#)')
	}, [])

	const handleReply = useCallback(() => {
		console.log('Reply (R)')
	}, [])

	const handleReplyAll = useCallback(() => {
		console.log('Reply all (Shift+R)')
	}, [])

	const handleForward = useCallback(() => {
		console.log('Forward (F)')
	}, [])

	const handleNewMessage = useCallback(() => {
		setIsComposeOpen(true)
	}, [])

	const handleToggleRead = useCallback(() => {
		console.log('Toggle read (U)')
	}, [])

	const handleMarkUnread = useCallback(() => {
		console.log('Mark unread (Shift+U)')
	}, [])

	const handleToggleStar = useCallback(() => {
		console.log('Toggle star (S)')
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

	useEffect(() => {
		if (!activeAccount && accounts.length > 0) {
			setActiveAccount(accounts[0])
		}
	}, [accounts, activeAccount, setActiveAccount])

	if (!activeAccount) {
		return (
			<div className='flex h-full items-center justify-center text-slate-400'>
				No accounts configured.
			</div>
		)
	}

	return (
		<>
			<div className='flex h-full overflow-hidden bg-slate-950'>
				<Sidebar
					activeAccount={activeAccount}
					activeMailbox={activeMailbox}
					onMailboxSelect={setActiveMailbox}
					onCompose={() => setIsComposeOpen(true)}
				/>
				<div className='flex flex-1 flex-col overflow-hidden'>
					{activeMailbox === 'Drafts' ? (
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
							onMessageClick={(uid) => console.log('Message clicked:', uid)}
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
