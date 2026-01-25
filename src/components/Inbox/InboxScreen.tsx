import { useState, useEffect } from 'react'
import { Sidebar } from '../Layout/Sidebar'
import { MessageList } from './MessageList'
import { DraftsList } from './DraftsList'
import { ComposeScreen } from '../Compose/ComposeScreen'
import { useDraftStore } from '@/stores/draftStore'
import type { AccountMeta } from '../../types/accounts'
import type { ComposeDraft } from '../../types/compose'

interface InboxScreenProps {
	accounts: AccountMeta[]
	activeAccount: AccountMeta | null
	setActiveAccount: (account: AccountMeta) => void
	onOpenSettings: () => void
}

export const InboxScreen = ({ accounts, activeAccount, setActiveAccount }: InboxScreenProps) => {
	const [activeMailbox, setActiveMailbox] = useState('INBOX')
	const [isComposeOpen, setIsComposeOpen] = useState(false)
	const { loadDraft } = useDraftStore()

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
