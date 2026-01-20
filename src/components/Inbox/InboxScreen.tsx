import { useState } from 'react'
import { Sidebar } from '../Layout/Sidebar'
import { MessageList } from './MessageList'
import type { AccountMeta } from '../../types/accounts'

interface InboxScreenProps {
	accounts: AccountMeta[]
    onOpenSettings: () => void
}

export const InboxScreen = ({ accounts, onOpenSettings }: InboxScreenProps) => {
    // Default to first account if available
	const [activeAccount, _setActiveAccount] = useState<AccountMeta | null>(accounts[0] || null)
	const [activeMailbox, setActiveMailbox] = useState('INBOX')

    if (!activeAccount) {
        return (
            <div className="flex h-full items-center justify-center text-slate-400">
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
                onLogout={() => {}} // TODO: Implement logout/lock
			/>
			<div className='flex flex-1 flex-col overflow-hidden'>
                {/* Top Bar for Context (Optional, e.g. Search) */}
                <div className="flex h-12 items-center border-b border-slate-800 bg-slate-900/30 px-4">
                    <h3 className="font-semibold text-slate-200">{activeMailbox}</h3>
                </div>

				<MessageList
					account={activeAccount}
					mailbox={activeMailbox}
					onMessageClick={(uid) => console.log('Message clicked:', uid)}
				/>
			</div>
		</div>
	)
}
