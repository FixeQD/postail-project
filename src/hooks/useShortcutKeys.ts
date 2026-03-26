import { useState, useEffect, useCallback } from 'react'
import { loadShortcutOverrides } from '@/config/shortcuts'

export const SHORTCUTS_UPDATED_EVENT = 'postail:shortcuts-updated'

/** Dispatch this after saving any override so hooks re-register */
export function dispatchShortcutsUpdated() {
	window.dispatchEvent(new CustomEvent(SHORTCUTS_UPDATED_EVENT))
}

/**
 * Returns a function that resolves the current key for a given scope:action.
 * Re-renders whenever the user saves an override.
 */
export function useShortcutKeys() {
	const [overrides, setOverrides] = useState<Record<string, string>>(() =>
		loadShortcutOverrides(),
	)

	useEffect(() => {
		const handler = () => setOverrides(loadShortcutOverrides())
		window.addEventListener(SHORTCUTS_UPDATED_EVENT, handler)
		return () => window.removeEventListener(SHORTCUTS_UPDATED_EVENT, handler)
	}, [])

	const getKey = useCallback(
		(scope: 'global' | 'compose' | 'inbox', action: string, defaultKey: string): string => {
			return overrides[`${scope}:${action}`] ?? defaultKey
		},
		[overrides],
	)

	return getKey
}
