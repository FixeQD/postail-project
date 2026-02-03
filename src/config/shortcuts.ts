/**
 * Central keyboard shortcuts configuration
 */

export interface ShortcutDefinition {
	key: string
	action: string
	scope: 'global' | 'compose' | 'inbox'
	preventDefault?: boolean
	stopPropagation?: boolean
}

export const defaultShortcuts: ShortcutDefinition[] = [
	// === GLOBAL SHORTCUTS ===
	{ key: 'ctrl+n, meta+n', action: 'new_message', scope: 'global', preventDefault: true },
	{ key: 'ctrl+f, meta+f', action: 'focus_search', scope: 'global', preventDefault: true },
	{ key: 'ctrl+r, meta+r', action: 'refresh', scope: 'global', preventDefault: true },
	{ key: 'ctrl+1, meta+1', action: 'go_inbox', scope: 'global', preventDefault: true },
	{ key: 'ctrl+2, meta+2', action: 'go_outbox', scope: 'global', preventDefault: true },
	{ key: 'ctrl+3, meta+3', action: 'go_drafts', scope: 'global', preventDefault: true },
	{ key: 'ctrl+4, meta+4', action: 'go_accounts', scope: 'global', preventDefault: true },
	{
		key: 'ctrl+comma, meta+comma',
		action: 'open_settings',
		scope: 'global',
		preventDefault: true,
	},

	// === COMPOSE SHORTCUTS ===
	{ key: 'ctrl+enter, meta+enter', action: 'send', scope: 'compose', preventDefault: true },
	{ key: 'ctrl+s, meta+s', action: 'save_draft', scope: 'compose', preventDefault: true },
	{ key: 'esc', action: 'close', scope: 'compose', preventDefault: true },
	{
		key: 'ctrl+shift+a, meta+shift+a',
		action: 'attach_file',
		scope: 'compose',
		preventDefault: true,
	},
	{ key: 'ctrl+k, meta+k', action: 'insert_link', scope: 'compose', preventDefault: true },
	{
		key: 'ctrl+shift+c, meta+shift+c',
		action: 'toggle_cc',
		scope: 'compose',
		preventDefault: true,
	},
	{
		key: 'ctrl+shift+b, meta+shift+b',
		action: 'toggle_bcc',
		scope: 'compose',
		preventDefault: true,
	},

	// === INBOX SHORTCUTS ===
	{ key: 'j', action: 'next_message', scope: 'inbox' },
	{ key: 'k', action: 'prev_message', scope: 'inbox' },
	{ key: 'enter, space', action: 'open_message', scope: 'inbox' },
	{ key: 'delete, #', action: 'delete_message', scope: 'inbox' },
	{ key: 'r', action: 'reply', scope: 'inbox' },
	{ key: 'shift+r', action: 'reply_all', scope: 'inbox' },
	{ key: 'f', action: 'forward', scope: 'inbox' },
	{ key: 'n', action: 'new_message', scope: 'inbox' },
	{ key: 'u', action: 'toggle_read', scope: 'inbox' },
	{ key: 'shift+u', action: 'mark_unread', scope: 'inbox' },
	{ key: 's', action: 'toggle_star', scope: 'inbox' },
	{ key: '/', action: 'focus_search', scope: 'inbox' },
]

export const shortcutsByScope = {
	global: defaultShortcuts.filter((s) => s.scope === 'global'),
	compose: defaultShortcuts.filter((s) => s.scope === 'compose'),
	inbox: defaultShortcuts.filter((s) => s.scope === 'inbox'),
}

/**
 * Retrieve the list of shortcut definitions for a given UI scope.
 *
 * @param scope - One of 'global', 'compose', or 'inbox'
 * @returns The array of ShortcutDefinition objects associated with `scope`
 */
export function getShortcutsForScope(scope: 'global' | 'compose' | 'inbox'): ShortcutDefinition[] {
	return shortcutsByScope[scope]
}

/**
 * Load the application's keyboard shortcut configuration.
 *
 * @returns An array of `ShortcutDefinition` objects representing the active keyboard shortcuts; currently returns the built-in defaults
 */
export function loadShortcutsConfig(): ShortcutDefinition[] {
	// TODO: Load from localStorage or settings DB or idk
	return defaultShortcuts
}

export const shortcutDescriptions: Record<string, string> = {
	new_message: 'New message',
	focus_search: 'Focus search',
	refresh: 'Refresh / Sync',
	go_inbox: 'Go to Inbox',
	go_outbox: 'Go to Outbox',
	go_drafts: 'Go to Drafts',
	go_accounts: 'Go to Accounts',
	open_settings: 'Open settings',
	send: 'Send message',
	save_draft: 'Save draft',
	close: 'Close / Discard',
	attach_file: 'Attach file',
	insert_link: 'Insert link',
	toggle_cc: 'Toggle Cc field',
	toggle_bcc: 'Toggle Bcc field',
	next_message: 'Next message',
	prev_message: 'Previous message',
	open_message: 'Open message',
	delete_message: 'Move to trash',
	reply: 'Reply',
	reply_all: 'Reply all',
	forward: 'Forward',
	toggle_read: 'Toggle read/unread',
	mark_unread: 'Mark as unread',
	toggle_star: 'Star / Flag message',
}

/**
 * Format a keyboard shortcut string into a human-readable display form.
 *
 * @param key - Raw shortcut string containing key tokens separated by commas, plus signs, or spaces (e.g., "ctrl+enter", "shift, a")
 * @returns A display-friendly shortcut where modifiers and common keys are normalized (e.g., "Ctrl+↵", "Shift+A")
 */
export function formatShortcutKey(key: string): string {
	return key
		.split(/[+,]/)
		.map((part) => {
			const trimmed = part.trim()
			if (trimmed === 'ctrl' || trimmed === 'meta') return 'Ctrl'
			if (trimmed === 'shift') return 'Shift'
			if (trimmed === 'alt' || trimmed === 'opt') return 'Alt'
			if (trimmed === 'enter') return '↵'
			if (trimmed === 'esc') return 'Esc'
			if (trimmed === 'space') return 'Space'
			if (trimmed === 'delete') return 'Del'
			if (trimmed === 'comma') return ','
			if (trimmed === '#') return '#'
			return trimmed.charAt(0).toUpperCase() + trimmed.slice(1)
		})
		.join('+')
}