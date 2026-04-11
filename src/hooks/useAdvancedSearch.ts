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

export function useAdvancedSearch(
	accountId: string | undefined,
	activeMailbox: string | undefined
) {
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
	}, [accountId, activeMailbox])

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
				// 1. Fetch local results first
				const localResults = await invoke<SearchResult[]>('search_messages_advanced', {
					accountId,
					mailbox: query.folder ?? activeMailbox ?? null,
					query: rawQueryString,
					bodyQuery: query.body?.trim() ?? null,
					limit: 200,
				})

				if (currentSearchId !== searchIdRef.current) return

				// Client-side filter helper
				const filterResults = (res: SearchResult[]) =>
					res.filter((r) => {
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
					results: filterResults(localResults),
					// Keep loading true since IMAP search is next
					isLoading: true,
					error: null,
				}))

				// 2. Fetch from IMAP in background
				const targetMailbox = query.folder ?? activeMailbox ?? 'INBOX'
				try {
					const headers = await invoke<any[]>('imap_search_messages', {
						accountId,
						mailbox: targetMailbox,
						criteria: {
							from: query.from?.trim() || null,
							to: query.to?.trim() || null,
							subject: query.subject?.trim() || null,
							body: query.body?.trim() || query.rawQuery?.trim() || null,
							since: query.dateFrom || null,
							before: query.dateTo || null,
							has_attachment: query.hasAttachment || null,
						},
					})

					if (currentSearchId !== searchIdRef.current) return

					const imapResults: SearchResult[] = headers.map((h: any) => ({
						message_id: h.message_id || 0,
						account_id: accountId,
						mailbox: h.mailbox,
						uid: h.uid,
						subject: h.subject,
						from_addr: h.from?.[0] ?? '',
						snippet: h.snippet,
						rank: 0,
						has_attachments: h.has_attachments,
						date: new Date(h.internal_date).getTime() / 1000,
					}))

					setState((prev) => {
						// Merge local and imap results, preferring IMAP order
						const existingKeys = new Set(
							imapResults.map((r) => `${r.account_id}:${r.mailbox}:${r.uid}`)
						)
						const combined = [
							...imapResults,
							...prev.results.filter(
								(r) => !existingKeys.has(`${r.account_id}:${r.mailbox}:${r.uid}`)
							),
						]
						// Sort by date desc
						combined.sort((a, b) => b.date - a.date)

						return {
							...prev,
							results: filterResults(combined),
							isLoading: false,
						}
					})
				} catch (err) {
					console.warn('IMAP search failed or skipped', err)
					if (currentSearchId !== searchIdRef.current) return
					setState((prev) => ({
						...prev,
						isLoading: false,
					}))
				}
			} catch (e) {
				if (currentSearchId !== searchIdRef.current) return
				setState((prev) => ({
					...prev,
					isLoading: false,
					error: e instanceof Error ? e.message : String(e),
				}))
			}
		},
		[accountId, activeMailbox]
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
