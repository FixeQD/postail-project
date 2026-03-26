import { useHotkeys } from 'react-hotkeys-hook'
import { useShortcutKeys } from './useShortcutKeys'

interface UseGlobalShortcutsProps {
	onNewMessage: () => void
	onFocusSearch: () => void
	onRefresh: () => void
	onGoToInbox: () => void
	onGoToOutbox: () => void
	onGoToDrafts: () => void
	onGoToAccounts: () => void
	onOpenSettings: () => void
	enabled?: boolean
}

export function useGlobalShortcuts({
	onNewMessage,
	onFocusSearch,
	onRefresh,
	onGoToInbox,
	onGoToOutbox,
	onGoToDrafts,
	onGoToAccounts,
	onOpenSettings,
	enabled = true,
}: UseGlobalShortcutsProps) {
	const getKey = useShortcutKeys()

	useHotkeys(
		getKey('global', 'new_message', 'ctrl+n, meta+n'),
		(e) => {
			e.preventDefault()
			onNewMessage()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onNewMessage, getKey]
	)
	useHotkeys(
		getKey('global', 'focus_search', 'ctrl+f, meta+f'),
		(e) => {
			e.preventDefault()
			onFocusSearch()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onFocusSearch, getKey]
	)
	useHotkeys(
		getKey('global', 'refresh', 'ctrl+r, meta+r'),
		(e) => {
			e.preventDefault()
			onRefresh()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onRefresh, getKey]
	)
	useHotkeys(
		getKey('global', 'go_inbox', 'ctrl+1, meta+1'),
		(e) => {
			e.preventDefault()
			onGoToInbox()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onGoToInbox, getKey]
	)
	useHotkeys(
		getKey('global', 'go_outbox', 'ctrl+2, meta+2'),
		(e) => {
			e.preventDefault()
			onGoToOutbox()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onGoToOutbox, getKey]
	)
	useHotkeys(
		getKey('global', 'go_drafts', 'ctrl+3, meta+3'),
		(e) => {
			e.preventDefault()
			onGoToDrafts()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onGoToDrafts, getKey]
	)
	useHotkeys(
		getKey('global', 'go_accounts', 'ctrl+4, meta+4'),
		(e) => {
			e.preventDefault()
			onGoToAccounts()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onGoToAccounts, getKey]
	)
	useHotkeys(
		getKey('global', 'open_settings', 'ctrl+comma, meta+comma'),
		(e) => {
			e.preventDefault()
			onOpenSettings()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onOpenSettings, getKey]
	)
}
