import { useHotkeys } from 'react-hotkeys-hook'
import { useShortcutKeys } from './useShortcutKeys'

interface UseInboxShortcutsProps {
	onNextMessage: () => void
	onPrevMessage: () => void
	onOpenMessage: () => void
	onDeleteMessage: () => void
	onReply: () => void
	onReplyAll: () => void
	onForward: () => void
	onNewMessage: () => void
	onToggleRead: () => void
	onMarkUnread: () => void
	onToggleStar: () => void
	onFocusSearch: () => void
	onArchive: () => void
	enabled?: boolean
}

export function useInboxShortcuts({
	onNextMessage,
	onPrevMessage,
	onOpenMessage,
	onDeleteMessage,
	onReply,
	onReplyAll,
	onForward,
	onNewMessage,
	onToggleRead,
	onMarkUnread,
	onToggleStar,
	onFocusSearch,
	onArchive,
	enabled = true,
}: UseInboxShortcutsProps) {
	const getKey = useShortcutKeys()

	useHotkeys(
		getKey('inbox', 'next_message', 'j'),
		(e) => {
			e.preventDefault()
			onNextMessage()
		},
		{ enabled, enableOnContentEditable: true },
		[onNextMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'prev_message', 'k'),
		(e) => {
			e.preventDefault()
			onPrevMessage()
		},
		{ enabled, enableOnContentEditable: true },
		[onPrevMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'open_message', 'enter, space'),
		(e) => {
			e.preventDefault()
			onOpenMessage()
		},
		{ enabled, enableOnContentEditable: true },
		[onOpenMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'delete_message', 'delete, #'),
		(e) => {
			e.preventDefault()
			onDeleteMessage()
		},
		{ enabled, enableOnContentEditable: true },
		[onDeleteMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'reply', 'r'),
		(e) => {
			e.preventDefault()
			onReply()
		},
		{ enabled, enableOnContentEditable: true },
		[onReply, getKey]
	)
	useHotkeys(
		getKey('inbox', 'reply_all', 'shift+r'),
		(e) => {
			e.preventDefault()
			onReplyAll()
		},
		{ enabled, enableOnContentEditable: true },
		[onReplyAll, getKey]
	)
	useHotkeys(
		getKey('inbox', 'forward', 'f'),
		(e) => {
			e.preventDefault()
			onForward()
		},
		{ enabled, enableOnContentEditable: true },
		[onForward, getKey]
	)
	useHotkeys(
		getKey('inbox', 'new_message', 'n'),
		(e) => {
			e.preventDefault()
			onNewMessage()
		},
		{ enabled, enableOnContentEditable: true },
		[onNewMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'toggle_read', 'u'),
		(e) => {
			e.preventDefault()
			onToggleRead()
		},
		{ enabled, enableOnContentEditable: true },
		[onToggleRead, getKey]
	)
	useHotkeys(
		getKey('inbox', 'mark_unread', 'shift+u'),
		(e) => {
			e.preventDefault()
			onMarkUnread()
		},
		{ enabled, enableOnContentEditable: true },
		[onMarkUnread, getKey]
	)
	useHotkeys(
		getKey('inbox', 'toggle_star', 's'),
		(e) => {
			e.preventDefault()
			onToggleStar()
		},
		{ enabled, enableOnContentEditable: true },
		[onToggleStar, getKey]
	)
	useHotkeys(
		getKey('inbox', 'focus_search', '/'),
		(e) => {
			e.preventDefault()
			onFocusSearch()
		},
		{ enabled, enableOnContentEditable: true },
		[onFocusSearch, getKey]
	)
	useHotkeys(
		getKey('inbox', 'archive', 'e'),
		(e) => {
			e.preventDefault()
			onArchive()
		},
		{ enabled, enableOnContentEditable: true },
		[onArchive, getKey]
	)
}
