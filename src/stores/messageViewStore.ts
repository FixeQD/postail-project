import { create } from 'zustand'

interface MessageViewState {
	selectedMessage: {
		accountId: string
		mailbox: string
		uid: number
	} | null

	viewMode: 'html' | 'plain'

	openMessage: (accountId: string, mailbox: string, uid: number) => void
	closeMessage: () => void
	toggleViewMode: () => void
	setViewMode: (mode: 'html' | 'plain') => void
}

export const useMessageViewStore = create<MessageViewState>((set) => ({
	selectedMessage: null,
	viewMode: 'html',

	openMessage: (accountId, mailbox, uid) =>
		set({
			selectedMessage: { accountId, mailbox, uid },
			viewMode: 'html',
		}),

	closeMessage: () =>
		set({
			selectedMessage: null,
			viewMode: 'html',
		}),

	toggleViewMode: () =>
		set((state) => ({
			viewMode: state.viewMode === 'html' ? 'plain' : 'html',
		})),

	// for when we need to force plain (e.g. no html content)
	setViewMode: (mode) => set({ viewMode: mode }),
}))
