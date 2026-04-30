import { create } from 'zustand'
import type { AdvancedSearchQuery } from '@/types/search'

interface SearchBarState {
	rawInput: string
	setRawInput: (val: string) => void
	query: AdvancedSearchQuery
	setQuery: (
		val: AdvancedSearchQuery | ((prev: AdvancedSearchQuery) => AdvancedSearchQuery)
	) => void
	hasActiveSearch: boolean
	setHasActiveSearch: (val: boolean) => void
}

export const useSearchBarStore = create<SearchBarState>((set) => ({
	rawInput: '',
	setRawInput: (val) => set({ rawInput: val }),
	query: {},
	setQuery: (val) =>
		set((state) => ({ query: typeof val === 'function' ? val(state.query) : val })),
	hasActiveSearch: false,
	setHasActiveSearch: (val) => set({ hasActiveSearch: val }),
}))
