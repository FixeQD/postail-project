import { create } from 'zustand'

interface MessageViewState {
	selectedMessage: {
		accountId: string
		mailbox: string
		uid: number
	} | null

	viewMode: 'html' | 'plain'

	isLoading: boolean

	// Set by MessageView so TitleBar can show subject
	titleMeta: {
		subject: string
		onNext?: () => void
		onPrev?: () => void
	} | null

	openMessage: (accountId: string, mailbox: string, uid: number) => void
	closeMessage: () => void
	setLoading: (loading: boolean) => void
	toggleViewMode: () => void
	setViewMode: (mode: 'html' | 'plain') => void
	setTitleMeta: (meta: MessageViewState['titleMeta']) => void
}

export const useMessageViewStore = create<MessageViewState>((set) => ({
	selectedMessage: null,
	viewMode: 'html',
	titleMeta: null,
	isLoading: false,

	openMessage: (accountId, mailbox, uid) =>
		set({ selectedMessage: { accountId, mailbox, uid }, viewMode: 'html', isLoading: true }),

	setLoading: (loading) => set({ isLoading: loading }),

	closeMessage: () =>
		set({ selectedMessage: null, viewMode: 'html', titleMeta: null, isLoading: false }),

	toggleViewMode: () =>
		set((state) => ({ viewMode: state.viewMode === 'html' ? 'plain' : 'html' })),

	setViewMode: (mode) => set({ viewMode: mode }),

	setTitleMeta: (meta) => set({ titleMeta: meta }),
}))
