import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen, Event } from '@tauri-apps/api/event'

export interface OutboxItem {
	id: string
	subject?: string
	recipient?: string
	status: 'PENDING' | 'PROCESSING' | 'SENT' | 'RETRY' | 'FAILED'
	attempts: number
	lastError?: string
	nextRetry?: number
}

interface OutboxState {
	items: OutboxItem[]
	isLoading: boolean
	selectedAccountId: string | null

	loadOutbox: (accountId: string) => Promise<void>
	retryMessage: (outboxId: string) => Promise<{ ok: true } | { ok: false; error: string }>
	cancelMessage: (outboxId: string) => Promise<{ ok: true } | { ok: false; error: string }>
	updateItemStatus: (
		outboxId: string,
		status: OutboxItem['status'],
		extra?: Partial<Pick<OutboxItem, 'lastError' | 'attempts' | 'nextRetry'>>
	) => void
	removeItem: (outboxId: string) => void
	setSelectedAccount: (accountId: string | null) => void
}

export const useOutboxStore = create<OutboxState>((set, get) => ({
	items: [],
	isLoading: false,
	selectedAccountId: null,

	loadOutbox: async (accountId: string) => {
		set({ isLoading: true, selectedAccountId: accountId })
		try {
			const items = await invoke<OutboxItem[]>('list_outbox', { accountId })
			set({ items, isLoading: false })
		} catch (error) {
			console.error('[OutboxStore] Failed to load outbox:', error)
			set({ isLoading: false })
		}
	},

	retryMessage: async (outboxId: string) => {
		try {
			await invoke('retry_sending', { outboxId })
			get().updateItemStatus(outboxId, 'PENDING', {
				lastError: undefined,
				nextRetry: undefined,
			})
			return { ok: true }
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error)
			console.error('[OutboxStore] Failed to retry message:', message)
			return { ok: false, error: message }
		}
	},

	cancelMessage: async (outboxId: string) => {
		try {
			await invoke('cancel_sending', { outboxId })
			get().removeItem(outboxId)
			return { ok: true }
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error)
			console.error('[OutboxStore] Failed to cancel message:', message)
			return { ok: false, error: message }
		}
	},

	updateItemStatus: (outboxId, status, extra = {}) => {
		set((state) => ({
			items: state.items.map((item) =>
				item.id === outboxId ? { ...item, status, ...extra } : item
			),
		}))
	},

	removeItem: (outboxId: string) => {
		set((state) => ({
			items: state.items.filter((item) => item.id !== outboxId),
		}))
	},

	setSelectedAccount: (accountId: string | null) => {
		set({ selectedAccountId: accountId })
	},
}))

export function setupOutboxListeners(
	onProcessing?: (outboxId: string, accountId: string) => void,
	onSent?: (outboxId: string, accountId: string) => void,
	onRetry?: (
		outboxId: string,
		accountId: string,
		details: { error: string; attempts: number; nextRetry: number }
	) => void,
	onFailed?: (
		outboxId: string,
		accountId: string,
		details: { error: string; attempts: number }
	) => void
) {
	const store = useOutboxStore.getState()
	const listeners: Promise<() => void>[] = []

	// PROCESSING
	listeners.push(
		listen(
			'outbox:message:processing',
			(event: Event<{ outboxId: string; accountId: string }>) => {
				const { outboxId, accountId } = event.payload
				store.updateItemStatus(outboxId, 'PROCESSING')
				onProcessing?.(outboxId, accountId)
			}
		)
	)

	// SENT
	listeners.push(
		listen('outbox:message:sent', (event: Event<{ outboxId: string; accountId: string }>) => {
			const { outboxId, accountId } = event.payload
			store.updateItemStatus(outboxId, 'SENT')
			onSent?.(outboxId, accountId)
		})
	)

	// RETRY
	listeners.push(
		listen(
			'outbox:message:retry',
			(
				event: Event<{
					outboxId: string
					accountId: string
					details: { error: string; attempts: number; nextRetry: number }
				}>
			) => {
				const { outboxId, accountId, details } = event.payload
				store.updateItemStatus(outboxId, 'RETRY', {
					lastError: details.error,
					attempts: details.attempts,
					nextRetry: details.nextRetry,
				})
				onRetry?.(outboxId, accountId, details)
			}
		)
	)

	// FAILED
	listeners.push(
		listen(
			'outbox:message:failed',
			(
				event: Event<{
					outboxId: string
					accountId: string
					details: { error: string; attempts: number }
				}>
			) => {
				const { outboxId, accountId, details } = event.payload
				store.updateItemStatus(outboxId, 'FAILED', {
					lastError: details.error,
					attempts: details.attempts,
				})
				onFailed?.(outboxId, accountId, details)
			}
		)
	)

	// PENDING (after manual retry)
	listeners.push(
		listen(
			'outbox:message:pending',
			(event: Event<{ outboxId: string; accountId: string }>) => {
				const { outboxId } = event.payload
				store.updateItemStatus(outboxId, 'PENDING', {
					lastError: undefined,
					nextRetry: undefined,
				})
			}
		)
	)

	// CANCELLED
	listeners.push(
		listen(
			'outbox:message:cancelled',
			(event: Event<{ outboxId: string; accountId: string }>) => {
				useOutboxStore.getState().removeItem(event.payload.outboxId)
			}
		)
	)

	return async () => {
		const cleanupFns = await Promise.all(listeners)
		cleanupFns.forEach((fn) => fn())
	}
}
