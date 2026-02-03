import { useHotkeys } from 'react-hotkeys-hook'

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

/**
 * Register Gmail-style keyboard shortcuts for inbox actions.
 *
 * Supported shortcuts:
 * - J: Next message
 * - K: Previous message
 * - Enter / Space: Open selected message
 * - Delete / #: Move to trash
 * - R: Reply
 * - Shift+R: Reply all
 * - F: Forward
 * - N: New message
 * - U: Toggle read/unread
 * - Shift+U: Mark as unread
 * - S: Toggle star/flag
 * - /: Focus search
 *
 * @param enabled - When `true` the shortcuts are active; when `false` all shortcuts are disabled
 */
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
	// J - Next message
	useHotkeys(
		'j',
		(e) => {
			e.preventDefault()
			onNextMessage()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onNextMessage]
	)

	// K - Previous message
	useHotkeys(
		'k',
		(e) => {
			e.preventDefault()
			onPrevMessage()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onPrevMessage]
	)

	// Enter/Space - Open message
	useHotkeys(
		'enter, space',
		(e) => {
			e.preventDefault()
			onOpenMessage()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onOpenMessage]
	)

	// Delete/# - Move to trash
	useHotkeys(
		'delete, #',
		(e) => {
			e.preventDefault()
			onDeleteMessage()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onDeleteMessage]
	)

	// R - Reply
	useHotkeys(
		'r',
		(e) => {
			e.preventDefault()
			onReply()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onReply]
	)

	// Shift+R - Reply all
	useHotkeys(
		'shift+r',
		(e) => {
			e.preventDefault()
			onReplyAll()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onReplyAll]
	)

	// F - Forward
	useHotkeys(
		'f',
		(e) => {
			e.preventDefault()
			onForward()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onForward]
	)

	// N - New message (Gmail style, in addition to Ctrl+N)
	useHotkeys(
		'n',
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

	// U - Toggle read/unread
	useHotkeys(
		'u',
		(e) => {
			e.preventDefault()
			onToggleRead()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onToggleRead]
	)

	// Shift+U - Mark as unread
	useHotkeys(
		'shift+u',
		(e) => {
			e.preventDefault()
			onMarkUnread()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onMarkUnread]
	)

	// S - Toggle star
	useHotkeys(
		's',
		(e) => {
			e.preventDefault()
			onToggleStar()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onToggleStar]
	)

	// / - Focus search
	useHotkeys(
		'/',
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
}