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
}

interface OutboxState {
	items: OutboxItem[]
	isLoading: boolean
	selectedAccountId: string | null

	// Actions
	loadOutbox: (accountId: string) => Promise<void>
	retryMessage: (outboxId: string) => Promise<void>
	cancelMessage: (outboxId: string) => Promise<void>
	updateItemStatus: (outboxId: string, status: OutboxItem['status'], error?: string) => void
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
			set({ isLoading: false })
		}
	},

	retryMessage: async (outboxId: string) => {
		try {
			await invoke('retry_sending', { outboxId })
			get().updateItemStatus(outboxId, 'PENDING')
		} catch (error) {}
	},

	cancelMessage: async (outboxId: string) => {
		try {
			await invoke('cancel_sending', { outboxId })
			// Remove from list
			set((state) => ({
				items: state.items.filter((item) => item.id !== outboxId),
			}))
		} catch (error) {}
	},

	updateItemStatus: (outboxId: string, status: OutboxItem['status'], error?: string) => {
		set((state) => ({
			items: state.items.map((item) =>
				item.id === outboxId
					? {
							...item,
							status,
							lastError: error,
							attempts: status === 'RETRY' ? item.attempts + 1 : item.attempts,
						}
					: item
			),
		}))
	},

	setSelectedAccount: (accountId: string | null) => {
		set({ selectedAccountId: accountId })
	},
}))

/**
 * Registers listeners for outbox lifecycle events and updates the outbox store accordingly.
 *
 * @param onProcessing - Called when a message transitions to processing with `(outboxId, accountId)`.
 * @param onSent - Called when a message is sent with `(outboxId, accountId)`.
 * @param onRetry - Called when a message is scheduled for retry with `(outboxId, accountId, details)` where `details` includes `error`, `attempts`, and `nextRetry`.
 * @param onFailed - Called when a message fails permanently with `(outboxId, accountId, details)` where `details` includes `error` and `attempts`.
 * @returns A cleanup function that unsubscribes all registered listeners.
 */
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
	const listeners: Promise<() => void>[] = []

	// Processing
	listeners.push(
		listen(
			'outbox:message:processing',
			(event: Event<{ outboxId: string; accountId: string }>) => {
				const { outboxId, accountId } = event.payload
				useOutboxStore.getState().updateItemStatus(outboxId, 'PROCESSING')
				onProcessing?.(outboxId, accountId)
			}
		)
	)

	// Sent
	listeners.push(
		listen('outbox:message:sent', (event: Event<{ outboxId: string; accountId: string }>) => {
			const { outboxId, accountId } = event.payload
			useOutboxStore.getState().updateItemStatus(outboxId, 'SENT')
			onSent?.(outboxId, accountId)
		})
	)

	// Retry
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
				useOutboxStore.getState().updateItemStatus(outboxId, 'RETRY', details.error)
				onRetry?.(outboxId, accountId, details)
			}
		)
	)

	// Failed
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
				useOutboxStore.getState().updateItemStatus(outboxId, 'FAILED', details.error)
				onFailed?.(outboxId, accountId, details)
			}
		)
	)

	// Return cleanup function
	return async () => {
		const cleanupFns = await Promise.all(listeners)
		cleanupFns.forEach((fn) => fn())
	}
}