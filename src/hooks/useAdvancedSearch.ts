import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { SearchResult, AdvancedSearchQuery } from '@/types/search'

interface SearchState {
	results: SearchResult[]
	isLoading: boolean
	error: string | null
	activeQuery: AdvancedSearchQuery | null
	rawQueryString: string
}

export function useAdvancedSearch(accountId: string | undefined) {
	const [state, setState] = useState<SearchState>({
		results: [],
		isLoading: false,
		error: null,
		activeQuery: null,
		rawQueryString: '',
	})

	useEffect(() => {
		setState({
			results: [],
			isLoading: false,
			error: null,
			activeQuery: null,
			rawQueryString: '',
		})
	}, [accountId])

	const search = useCallback(
		async (query: AdvancedSearchQuery) => {
			if (!accountId) return

			const parts: string[] = []
			if (query.rawQuery?.trim()) parts.push(query.rawQuery.trim())
			if (query.subject?.trim()) parts.push(`subject:"${query.subject.trim()}"`)
			if (query.from?.trim()) parts.push(`from_addr:"${query.from.trim()}"`)
			if (query.to?.trim()) parts.push(`"${query.to.trim()}"`)
			const rawQueryString = parts.join(' ')

			setState((prev) => ({
				...prev,
				isLoading: true,
				error: null,
				activeQuery: query,
				rawQueryString,
			}))

			try {
				const results = await invoke<SearchResult[]>('search_messages_advanced', {
					accountId,
					mailbox: query.folder ?? null,
					query: rawQueryString || '""',
					bodyQuery: query.body?.trim() ?? null,
					limit: 200,
				})

				// Client-side date + attachment filter
				const filtered = results.filter((r) => {
					if (query.hasAttachment === true && !r.has_attachments) return false
					if (query.dateFrom) {
						const from = new Date(query.dateFrom).getTime()
						if (r.date * 1000 < from) return false
					}
					if (query.dateTo) {
						const to = new Date(query.dateTo).getTime() + 86_400_000
						if (r.date * 1000 > to) return false
					}
					return true
				})

				setState((prev) => ({
					...prev,
					results: filtered,
					isLoading: false,
					error: null,
				}))
			} catch (e) {
				setState((prev) => ({
					...prev,
					isLoading: false,
					error: e instanceof Error ? e.message : String(e),
				}))
			}
		},
		[accountId]
	)

	const clear = useCallback(() => {
		setState({
			results: [],
			isLoading: false,
			error: null,
			activeQuery: null,
			rawQueryString: '',
		})
	}, [])

	return {
		results: state.results,
		isLoading: state.isLoading,
		error: state.error,
		activeQuery: state.activeQuery,
		rawQueryString: state.rawQueryString,
		isActive: state.activeQuery !== null,
		search,
		clear,
	}
}
