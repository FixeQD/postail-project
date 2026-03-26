import { useHotkeys } from 'react-hotkeys-hook'
import { useShortcutKeys } from './useShortcutKeys'

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
	const getKey = useShortcutKeys()

	useHotkeys(
		getKey('compose', 'send', 'ctrl+enter, meta+enter'),
		(e) => {
			e.preventDefault()
			onSend()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onSend, getKey]
	)
	useHotkeys(
		getKey('compose', 'save_draft', 'ctrl+s, meta+s'),
		(e) => {
			e.preventDefault()
			onSaveDraft()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onSaveDraft, getKey]
	)
	useHotkeys(
		getKey('compose', 'close', 'esc'),
		(e) => {
			e.preventDefault()
			onClose()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onClose, getKey]
	)
	useHotkeys(
		getKey('compose', 'attach_file', 'ctrl+shift+a, meta+shift+a'),
		(e) => {
			e.preventDefault()
			onAttachFile()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onAttachFile, getKey]
	)
	useHotkeys(
		getKey('compose', 'insert_link', 'ctrl+k, meta+k'),
		(e) => {
			e.preventDefault()
			onInsertLink()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onInsertLink, getKey]
	)
	useHotkeys(
		getKey('compose', 'toggle_cc', 'ctrl+shift+c, meta+shift+c'),
		(e) => {
			e.preventDefault()
			onToggleCc()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onToggleCc, getKey]
	)
	useHotkeys(
		getKey('compose', 'toggle_bcc', 'ctrl+shift+b, meta+shift+b'),
		(e) => {
			e.preventDefault()
			onToggleBcc()
		},
		{ enabled, enableOnFormTags: true, enableOnContentEditable: true },
		[onToggleBcc, getKey]
	)
}
