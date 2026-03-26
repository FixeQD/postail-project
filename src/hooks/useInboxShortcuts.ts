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
	enabled = true,
}: UseInboxShortcutsProps) {
	const getKey = useShortcutKeys()

	useHotkeys(
		getKey('inbox', 'next_message', 'j'),
		(e) => {
			e.preventDefault()
			onNextMessage()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onNextMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'prev_message', 'k'),
		(e) => {
			e.preventDefault()
			onPrevMessage()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onPrevMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'open_message', 'enter, space'),
		(e) => {
			e.preventDefault()
			onOpenMessage()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onOpenMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'delete_message', 'delete, #'),
		(e) => {
			e.preventDefault()
			onDeleteMessage()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onDeleteMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'reply', 'r'),
		(e) => {
			e.preventDefault()
			onReply()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onReply, getKey]
	)
	useHotkeys(
		getKey('inbox', 'reply_all', 'shift+r'),
		(e) => {
			e.preventDefault()
			onReplyAll()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onReplyAll, getKey]
	)
	useHotkeys(
		getKey('inbox', 'forward', 'f'),
		(e) => {
			e.preventDefault()
			onForward()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onForward, getKey]
	)
	useHotkeys(
		getKey('inbox', 'new_message', 'n'),
		(e) => {
			e.preventDefault()
			onNewMessage()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onNewMessage, getKey]
	)
	useHotkeys(
		getKey('inbox', 'toggle_read', 'u'),
		(e) => {
			e.preventDefault()
			onToggleRead()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onToggleRead, getKey]
	)
	useHotkeys(
		getKey('inbox', 'mark_unread', 'shift+u'),
		(e) => {
			e.preventDefault()
			onMarkUnread()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onMarkUnread, getKey]
	)
	useHotkeys(
		getKey('inbox', 'toggle_star', 's'),
		(e) => {
			e.preventDefault()
			onToggleStar()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onToggleStar, getKey]
	)
	useHotkeys(
		getKey('inbox', 'focus_search', '/'),
		(e) => {
			e.preventDefault()
			onFocusSearch()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onFocusSearch, getKey]
	)
}
