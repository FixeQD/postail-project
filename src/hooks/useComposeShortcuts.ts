import { useHotkeys } from 'react-hotkeys-hook'

interface UseComposeShortcutsProps {
	onSend: () => void
	onSaveDraft: () => void
	onClose: () => void
	onAttachFile: () => void
	onInsertLink: () => void
	onToggleCc: () => void
	onToggleBcc: () => void
	enabled?: boolean
}

/**
 * Register keyboard shortcuts for common compose-screen actions.
 *
 * Shortcuts:
 * - Ctrl/Cmd+Enter → send
 * - Ctrl/Cmd+S → save draft
 * - Esc → close/discard
 * - Ctrl/Cmd+Shift+A → attach file
 * - Ctrl/Cmd+K → insert link
 * - Ctrl/Cmd+Shift+C → toggle Cc field
 * - Ctrl/Cmd+Shift+B → toggle Bcc field
 *
 * @param onSend - Called when the send shortcut is triggered
 * @param onSaveDraft - Called when the save-draft shortcut is triggered
 * @param onClose - Called when the close/discard shortcut is triggered
 * @param onAttachFile - Called when the attach-file shortcut is triggered
 * @param onInsertLink - Called when the insert-link shortcut is triggered
 * @param onToggleCc - Called when the toggle-Cc shortcut is triggered
 * @param onToggleBcc - Called when the toggle-Bcc shortcut is triggered
 * @param enabled - Whether the shortcuts are active (defaults to `true`)
 */
export function useComposeShortcuts({
	onSend,
	onSaveDraft,
	onClose,
	onAttachFile,
	onInsertLink,
	onToggleCc,
	onToggleBcc,
	enabled = true,
}: UseComposeShortcutsProps) {
	// Ctrl/Cmd+Enter - Send message
	useHotkeys(
		'ctrl+enter, meta+enter',
		(e) => {
			e.preventDefault()
			onSend()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onSend]
	)

	// Ctrl/Cmd+S - Save draft
	useHotkeys(
		'ctrl+s, meta+s',
		(e) => {
			e.preventDefault()
			onSaveDraft()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onSaveDraft]
	)

	// Esc - Close/Discard
	useHotkeys(
		'esc',
		(e) => {
			e.preventDefault()
			onClose()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onClose]
	)

	// Ctrl/Cmd+Shift+A - Attach file
	useHotkeys(
		'ctrl+shift+a, meta+shift+a',
		(e) => {
			e.preventDefault()
			onAttachFile()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onAttachFile]
	)

	// Ctrl/Cmd+K - Insert link
	useHotkeys(
		'ctrl+k, meta+k',
		(e) => {
			e.preventDefault()
			onInsertLink()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onInsertLink]
	)

	// Ctrl/Cmd+Shift+C - Toggle Cc field
	useHotkeys(
		'ctrl+shift+c, meta+shift+c',
		(e) => {
			e.preventDefault()
			onToggleCc()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onToggleCc]
	)

	// Ctrl/Cmd+Shift+B - Toggle Bcc field
	useHotkeys(
		'ctrl+shift+b, meta+shift+b',
		(e) => {
			e.preventDefault()
			onToggleBcc()
		},
		{
			enabled,
			enableOnFormTags: true,
			enableOnContentEditable: true,
		},
		[onToggleBcc]
	)
}