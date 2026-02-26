import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import type { AccountMeta } from '../types/accounts'
import { useMessageViewStore } from './messageViewStore'

interface AccountState {
	accounts: AccountMeta[]
	activeAccount: AccountMeta | null
	activeMailbox: string
	lastMailboxes: Record<string, string>
	isLoading: boolean

	setAccounts: (accounts: AccountMeta[]) => void
	setActiveAccount: (account: AccountMeta | null) => void
	setActiveMailbox: (mailbox: string) => void
	fetchAccounts: () => Promise<AccountMeta[]>
	removeAccount: (id: string) => Promise<void>
}

export const useAccountStore = create<AccountState>((set, get) => ({
	accounts: [],
	activeAccount: null,
	activeMailbox: 'INBOX',
	lastMailboxes: {},
	isLoading: false,

	setAccounts: (accounts) => set({ accounts }),
	
	setActiveAccount: (account) => {
		useMessageViewStore.getState().closeMessage()
		
		const { lastMailboxes } = get()
		const nextMailbox = account ? (lastMailboxes[account.id] || 'INBOX') : 'INBOX'
		
		set({ activeAccount: account, activeMailbox: nextMailbox })
	},
	
	setActiveMailbox: (mailbox) => {
		const { activeAccount, lastMailboxes } = get()
		if (activeAccount) {
			set({ 
				activeMailbox: mailbox,
				lastMailboxes: { ...lastMailboxes, [activeAccount.id]: mailbox }
			})
		} else {
			set({ activeMailbox: mailbox })
		}
	},

	fetchAccounts: async () => {
		set({ isLoading: true })
		try {
			const fetchedAccounts = await invoke<AccountMeta[]>('list_accounts')
			set({ accounts: fetchedAccounts, isLoading: false })
			
			// Auto-set active account if none selected
			const { activeAccount } = get()
			if (fetchedAccounts.length > 0 && !activeAccount) {
				set({ activeAccount: fetchedAccounts[0] })
			}
			
			return fetchedAccounts
		} catch (error) {
			console.error('Failed to fetch accounts:', error)
			set({ isLoading: false })
			return []
		}
	},

	removeAccount: async (id: string) => {
		try {
			await invoke('remove_account', { id })
			const { accounts, activeAccount } = get()
			const updatedAccounts = accounts.filter((a) => a.id !== id)
			
			set({ 
				accounts: updatedAccounts,
				activeAccount: activeAccount?.id === id ? (updatedAccounts[0] || null) : activeAccount
			})
		} catch (error) {
			console.error('Failed to remove account:', error)
			throw error
		}
	}
}))
