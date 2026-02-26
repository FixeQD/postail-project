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

		if (account) {
			invoke('set_setting', { key: 'postail.last_account', value: account.id }).catch(console.error)
			invoke('set_setting', { key: `postail.last_mailbox.${account.id}`, value: nextMailbox }).catch(console.error)
		}
	},
	
	setActiveMailbox: (mailbox) => {
		const { activeAccount, lastMailboxes } = get()
		if (activeAccount) {
			set({ 
				activeMailbox: mailbox,
				lastMailboxes: { ...lastMailboxes, [activeAccount.id]: mailbox }
			})
			invoke('set_setting', { key: `postail.last_mailbox.${activeAccount.id}`, value: mailbox }).catch(console.error)
		} else {
			set({ activeMailbox: mailbox })
		}
	},

	fetchAccounts: async () => {
		set({ isLoading: true })
		try {
			const fetchedAccounts = await invoke<AccountMeta[]>('list_accounts')
			set({ accounts: fetchedAccounts, isLoading: false })
			
			// Try to restore last account and its last mailbox
			const { activeAccount } = get()
			if (fetchedAccounts.length > 0 && !activeAccount) {
				try {
					const lastAccountId = await invoke<string | null>('get_setting', { key: 'postail.last_account' })
					let targetAccount = fetchedAccounts.find(a => a.id === lastAccountId)

					if (!targetAccount) {
						targetAccount = fetchedAccounts[0]
					}
					
					const lastAccountMailbox = await invoke<string | null>('get_setting', { key: `postail.last_mailbox.${targetAccount.id}` })
					const initialMailbox = lastAccountMailbox || 'INBOX'

					set({ 
						activeAccount: targetAccount, 
						activeMailbox: initialMailbox,
						lastMailboxes: { [targetAccount.id]: initialMailbox } 
					})
				} catch (err) {
					console.error('Failed to restore last active account', err)
					set({ activeAccount: fetchedAccounts[0] })
				}
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
