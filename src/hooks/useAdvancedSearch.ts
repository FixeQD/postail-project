import { useState, useCallback, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { SearchResult, AdvancedSearchQuery } from '@/types/search'

interface SearchState {
	results: SearchResult[]
	isLoading: boolean
	error: string | null
	activeQuery: AdvancedSearchQuery | null
	rawQueryString: string
	displayQueryString: string
}

export function useAdvancedSearch(accountId: string | undefined) {
	const searchIdRef = useRef(0)
	const [state, setState] = useState<SearchState>({
		results: [],
		isLoading: false,
		error: null,
		activeQuery: null,
		rawQueryString: '',
		displayQueryString: '',
	})

	useEffect(() => {
		searchIdRef.current += 1
		setState({
			results: [],
			isLoading: false,
			error: null,
			activeQuery: null,
			rawQueryString: '',
			displayQueryString: '',
		})
	}, [accountId])

	const search = useCallback(
		async (query: AdvancedSearchQuery) => {
			if (!accountId) return

			const currentSearchId = ++searchIdRef.current

			const parts: string[] = []
			if (query.rawQuery?.trim()) parts.push(query.rawQuery.trim())
			if (query.subject?.trim())
				parts.push(`subject:"${query.subject.trim().replace(/"/g, '""')}"`)
			if (query.from?.trim())
				parts.push(`from_addr:"${query.from.trim().replace(/"/g, '""')}"`)
			if (query.to?.trim()) parts.push(`to_json:"${query.to.trim().replace(/"/g, '""')}"`)
			const rawQueryString = parts.join(' ')

			const displayParts: string[] = []
			if (query.rawQuery?.trim()) displayParts.push(query.rawQuery.trim())
			if (query.subject?.trim()) displayParts.push(`subject:"${query.subject.trim()}"`)
			if (query.from?.trim()) displayParts.push(`from:"${query.from.trim()}"`)
			if (query.to?.trim()) displayParts.push(`to:"${query.to.trim()}"`)
			if (query.body?.trim()) displayParts.push(`body:"${query.body.trim()}"`)
			const displayQueryString = displayParts.join(' ')

			setState((prev) => ({
				...prev,
				isLoading: true,
				error: null,
				activeQuery: query,
				rawQueryString,
				displayQueryString,
			}))

			try {
				const results = await invoke<SearchResult[]>('search_messages_advanced', {
					accountId,
					mailbox: query.folder ?? null,
					query: rawQueryString,
					bodyQuery: query.body?.trim() ?? null,
					limit: 200,
				})

				if (currentSearchId !== searchIdRef.current) return

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
				if (currentSearchId !== searchIdRef.current) return
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
		searchIdRef.current += 1
		setState({
			results: [],
			isLoading: false,
			error: null,
			activeQuery: null,
			rawQueryString: '',
			displayQueryString: '',
		})
	}, [])

	return {
		results: state.results,
		isLoading: state.isLoading,
		error: state.error,
		activeQuery: state.activeQuery,
		rawQueryString: state.rawQueryString,
		displayQueryString: state.displayQueryString,
		isActive: state.activeQuery !== null,
		search,
		clear,
	}
}
