import { useHotkeys } from 'react-hotkeys-hook'

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

/**
 * Register global keyboard shortcuts and invoke the provided callbacks when those shortcuts are triggered.
 *
 * @param onNewMessage - Called when the new-message shortcut (Ctrl/Cmd+N) is pressed
 * @param onFocusSearch - Called when the focus-search shortcut (Ctrl/Cmd+F) is pressed
 * @param onRefresh - Called when the refresh shortcut (Ctrl/Cmd+R) is pressed
 * @param onGoToInbox - Called when the go-to-inbox shortcut (Ctrl/Cmd+1) is pressed
 * @param onGoToOutbox - Called when the go-to-outbox shortcut (Ctrl/Cmd+2) is pressed
 * @param onGoToDrafts - Called when the go-to-drafts shortcut (Ctrl/Cmd+3) is pressed
 * @param onGoToAccounts - Called when the go-to-accounts shortcut (Ctrl/Cmd+4) is pressed
 * @param onOpenSettings - Called when the open-settings shortcut (Ctrl/Cmd+,) is pressed
 * @param enabled - Whether shortcuts are active; when false, no shortcuts will be triggered
 */
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
	// Ctrl/Cmd+N - New message
	useHotkeys(
		'ctrl+n, meta+n',
		(e) => {
			e.preventDefault()
			onNewMessage()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onNewMessage]
	)

	// Ctrl/Cmd+F - Focus search
	useHotkeys(
		'ctrl+f, meta+f',
		(e) => {
			e.preventDefault()
			onFocusSearch()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onFocusSearch]
	)

	// Ctrl/Cmd+R - Refresh
	useHotkeys(
		'ctrl+r, meta+r',
		(e) => {
			e.preventDefault()
			onRefresh()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onRefresh]
	)

	// Ctrl/Cmd+1 - Go to Inbox
	useHotkeys(
		'ctrl+1, meta+1',
		(e) => {
			e.preventDefault()
			onGoToInbox()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onGoToInbox]
	)

	// Ctrl/Cmd+2 - Go to Outbox
	useHotkeys(
		'ctrl+2, meta+2',
		(e) => {
			e.preventDefault()
			onGoToOutbox()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onGoToOutbox]
	)

	// Ctrl/Cmd+3 - Go to Drafts
	useHotkeys(
		'ctrl+3, meta+3',
		(e) => {
			e.preventDefault()
			onGoToDrafts()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onGoToDrafts]
	)

	// Ctrl/Cmd+4 - Go to Accounts
	useHotkeys(
		'ctrl+4, meta+4',
		(e) => {
			e.preventDefault()
			onGoToAccounts()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onGoToAccounts]
	)

	// Ctrl/Cmd+Comma - Open settings
	useHotkeys(
		'ctrl+comma, meta+comma',
		(e) => {
			e.preventDefault()
			onOpenSettings()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onOpenSettings]
	)
}