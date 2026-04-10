import { useState, useCallback } from 'react'

const STORAGE_KEY = 'postail.search_history'
const MAX_HISTORY = 20

function readHistory(): string[] {
	try {
		const raw = localStorage.getItem(STORAGE_KEY)
		return raw ? (JSON.parse(raw) as string[]) : []
	} catch {
		return []
	}
}

function writeHistory(queries: string[]): void {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(queries))
	} catch {
		// storage full or unavailable – silently skip
	}
}

export function useSearchHistory() {
	const [queries, setQueries] = useState<string[]>(() => readHistory())

	const addQuery = useCallback((query: string) => {
		const trimmed = query.trim()
		if (!trimmed) return
		setQueries((prev) => {
			const deduped = [trimmed, ...prev.filter((q) => q !== trimmed)].slice(0, MAX_HISTORY)
			writeHistory(deduped)
			return deduped
		})
	}, [])

	const removeQuery = useCallback((query: string) => {
		setQueries((prev) => {
			const next = prev.filter((q) => q !== query)
			writeHistory(next)
			return next
		})
	}, [])

	const clearHistory = useCallback(() => {
		writeHistory([])
		setQueries([])
	}, [])

	return { queries, addQuery, removeQuery, clearHistory }
}
