import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen, Event } from '@tauri-apps/api/event'
import { formatDistanceToNow } from 'date-fns'

export interface AccountSyncStatus {
	accountId: string
	accountEmail: string
	status: 'idle' | 'syncing' | 'error'
	mailbox?: string
	progress?: { current: number; total: number }
	mailboxProgress?: { currentMailbox: number; totalMailboxes: number }
	lastSync?: number
	error?: string
}

interface SyncState {
	statuses: Map<string, AccountSyncStatus>
	isLoading: boolean

	// Actions
	setStatus: (accountId: string, status: AccountSyncStatus) => void
	updateStatus: (accountId: string, updates: Partial<AccountSyncStatus>) => void
	getStatus: (accountId: string) => AccountSyncStatus | undefined
	getAllStatuses: () => AccountSyncStatus[]
	getFormattedLastSync: (accountId: string) => string
	cancelSync: (accountId: string) => Promise<void>
	retrySync: (accountId: string) => Promise<void>
	loadInitialStatuses: (accounts: { id: string; email: string }[]) => Promise<void>
}

export const useSyncStore = create<SyncState>((set, get) => ({
	statuses: new Map(),
	isLoading: false,

	setStatus: (accountId: string, status: AccountSyncStatus) => {
		set((state) => {
			const newStatuses = new Map(state.statuses)
			newStatuses.set(accountId, status)
			return { statuses: newStatuses }
		})
	},

	updateStatus: (accountId: string, updates: Partial<AccountSyncStatus>) => {
		set((state) => {
			const newStatuses = new Map(state.statuses)
			const current = newStatuses.get(accountId)
			if (current) {
				newStatuses.set(accountId, { ...current, ...updates })
			}
			return { statuses: newStatuses }
		})
	},

	getStatus: (accountId: string) => {
		return get().statuses.get(accountId)
	},

	getAllStatuses: () => {
		return Array.from(get().statuses.values())
	},

	getFormattedLastSync: (accountId: string) => {
		const status = get().statuses.get(accountId)
		if (!status?.lastSync) return 'Never'
		return formatDistanceToNow(status.lastSync * 1000, { addSuffix: true })
	},

	cancelSync: async (accountId: string) => {
		try {
			await invoke('stop_sync', { accountId })
			get().updateStatus(accountId, { status: 'idle' })
		} catch (error) {
			console.error('Failed to cancel sync:', error)
		}
	},

	retrySync: async (accountId: string) => {
		try {
			await invoke('start_sync', { accountId })
		} catch (error) {
			console.error('Failed to retry sync:', error)
		}
	},

	loadInitialStatuses: async (accounts: { id: string; email: string }[]) => {
		set({ isLoading: true })
		try {
			for (const account of accounts) {
				try {
					const status = await invoke<{
						Idle?: null
						Syncing?: null
						Error?: string
					}>('get_sync_status', { accountId: account.id })

					let normalizedStatus: 'idle' | 'syncing' | 'error' = 'idle'
					let errorMessage: string | undefined

					if (status.Error !== undefined) {
						normalizedStatus = 'error'
						errorMessage = status.Error
					} else if (status.Syncing !== undefined) {
						normalizedStatus = 'syncing'
					}

					const existing = get().statuses.get(account.id)
					if (!existing) {
						get().setStatus(account.id, {
							accountId: account.id,
							accountEmail: account.email,
							status: normalizedStatus,
							error: errorMessage,
							mailbox: undefined,
							progress: undefined,
							lastSync: undefined,
						})
					}
				} catch (e) {
					console.error(`Failed to load sync status for ${account.id}:`, e)
				}
			}
		} finally {
			set({ isLoading: false })
		}
	},
}))

export function setupSyncListeners(
	onStarted?: (accountId: string) => void,
	onProgress?: (
		accountId: string,
		details: { mailbox: string; current: number; total: number }
	) => void,
	onCompleted?: (accountId: string, timestamp: number) => void,
	onError?: (accountId: string, error: string) => void
) {
	const listeners: Promise<() => void>[] = []

	// Sync started
	listeners.push(
		listen('sync:started', (event: Event<{ accountId: string; accountEmail: string }>) => {
			const { accountId, accountEmail } = event.payload
			useSyncStore.getState().setStatus(accountId, {
				accountId,
				accountEmail,
				status: 'syncing',
			})
			onStarted?.(accountId)
		})
	)

	// Sync progress
	listeners.push(
		listen(
			'sync:progress',
			(
				event: Event<{
					accountId: string
					mailbox: string
					current: number
					total: number
					mailboxProgress?: { currentMailbox: number; totalMailboxes: number }
				}>
			) => {
				const { accountId, mailbox, current, total, mailboxProgress } = event.payload
				useSyncStore.getState().updateStatus(accountId, {
					status: 'syncing',
					mailbox,
					progress: { current, total },
					mailboxProgress,
				})
				onProgress?.(accountId, { mailbox, current, total })
			}
		)
	)

	// Sync completed
	listeners.push(
		listen('sync:completed', (event: Event<{ accountId: string; timestamp: number }>) => {
			const { accountId, timestamp } = event.payload
			useSyncStore.getState().updateStatus(accountId, {
				status: 'idle',
				mailbox: undefined,
				progress: undefined,
				lastSync: timestamp,
			})
			onCompleted?.(accountId, timestamp)
		})
	)

	// Sync error
	listeners.push(
		listen('sync:error', (event: Event<{ accountId: string; error: string }>) => {
			const { accountId, error } = event.payload
			useSyncStore.getState().updateStatus(accountId, {
				status: 'error',
				error,
				mailbox: undefined,
				progress: undefined,
			})
			onError?.(accountId, error)
		})
	)

	// Return cleanup function
	return async () => {
		const cleanupFns = await Promise.all(listeners)
		for (const fn of cleanupFns) {
			fn()
		}
	}
}
