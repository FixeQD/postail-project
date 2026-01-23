import { useState, useEffect } from 'react'
import { Sidebar } from '../Layout/Sidebar'
import { MessageList } from './MessageList'
import type { AccountMeta } from '../../types/accounts'

interface InboxScreenProps {
	accounts: AccountMeta[]
	activeAccount: AccountMeta | null
	setActiveAccount: (account: AccountMeta) => void
	onOpenSettings: () => void
}

export const InboxScreen = ({
	accounts,
	activeAccount,
	setActiveAccount,
	onOpenSettings,
}: InboxScreenProps) => {
	const [activeMailbox, setActiveMailbox] = useState('INBOX')

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
		<div className='flex h-full overflow-hidden bg-slate-950'>
			<Sidebar
				activeAccount={activeAccount}
				activeMailbox={activeMailbox}
				onMailboxSelect={setActiveMailbox}
				onOpenSettings={onOpenSettings}
				onLogout={() => {}}
			/>
			<div className='flex flex-1 flex-col overflow-hidden'>
				<MessageList
					account={activeAccount}
					mailbox={activeMailbox}
					onMessageClick={(uid) => console.log('Message clicked:', uid)}
				/>
			</div>
		</div>
	)
}
