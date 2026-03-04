import { useState, useCallback } from 'react'

export function useAsyncState() {
	const [isLoading, setIsLoading] = useState(false)

	const run = useCallback(async (fn: () => Promise<void>) => {
		setIsLoading(true)
		try {
			await fn()
		} finally {
			setIsLoading(false)
		}
	}, [])

	return { isLoading, setIsLoading, run }
}
