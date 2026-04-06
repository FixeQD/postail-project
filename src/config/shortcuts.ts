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
	{ key: 'e', action: 'archive', scope: 'inbox' },
]

export const shortcutsByScope = {
	global: defaultShortcuts.filter((s) => s.scope === 'global'),
	compose: defaultShortcuts.filter((s) => s.scope === 'compose'),
	inbox: defaultShortcuts.filter((s) => s.scope === 'inbox'),
}

export function getShortcutsForScope(scope: 'global' | 'compose' | 'inbox'): ShortcutDefinition[] {
	return shortcutsByScope[scope]
}

// ─── Persistence ──────────────────────────────────────────────────────────────

const STORAGE_KEY = 'postail_custom_shortcuts'

/** Load overrides: { action -> key } */
export function loadShortcutOverrides(): Record<string, string> {
	try {
		const raw = localStorage.getItem(STORAGE_KEY)
		if (!raw) return {}
		return JSON.parse(raw) as Record<string, string>
	} catch {
		return {}
	}
}

/** Save a single override */
export function saveShortcutOverride(action: string, key: string): void {
	const overrides = loadShortcutOverrides()
	overrides[action] = key
	localStorage.setItem(STORAGE_KEY, JSON.stringify(overrides))
}

/** Remove a single override (revert to default) */
export function resetShortcutOverride(action: string): void {
	const overrides = loadShortcutOverrides()
	delete overrides[action]
	localStorage.setItem(STORAGE_KEY, JSON.stringify(overrides))
}

/** Clear all overrides */
export function resetAllShortcutOverrides(): void {
	localStorage.removeItem(STORAGE_KEY)
}

/** Merge defaults with saved overrides */
export function loadShortcutsConfig(): ShortcutDefinition[] {
	const overrides = loadShortcutOverrides()
	return defaultShortcuts.map((s) =>
		overrides[s.action] ? { ...s, key: overrides[s.action] } : s
	)
}

// ─── Key capture ──────────────────────────────────────────────────────────────

const IGNORED_KEYS = new Set([
	'Control',
	'Meta',
	'Shift',
	'Alt',
	'CapsLock',
	'NumLock',
	'ScrollLock',
	'Unidentified',
])

/** Convert a KeyboardEvent to a shortcut string like "ctrl+shift+k" */
export function eventToShortcutKey(e: KeyboardEvent): string | null {
	if (IGNORED_KEYS.has(e.key)) return null

	const parts: string[] = []
	if (e.ctrlKey || e.metaKey) parts.push('ctrl')
	if (e.shiftKey) parts.push('shift')
	if (e.altKey) parts.push('alt')

	let key = e.key.toLowerCase()
	if (key === 'escape') key = 'esc'
	else if (key === 'enter') key = 'enter'
	else if (key === ' ') key = 'space'
	else if (key === 'backspace') key = 'backspace'
	else if (key === 'delete') key = 'delete'
	else if (key === ',') key = 'comma'
	else if (key === 'arrowup') key = 'up'
	else if (key === 'arrowdown') key = 'down'
	else if (key === 'arrowleft') key = 'left'
	else if (key === 'arrowright') key = 'right'

	parts.push(key)
	return parts.join('+')
}

// ─── Descriptions & formatting ────────────────────────────────────────────────

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
	archive: 'Archive message',
}

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
			if (trimmed === 'backspace') return '⌫'
			if (trimmed === 'comma') return ','
			if (trimmed === 'up') return '↑'
			if (trimmed === 'down') return '↓'
			if (trimmed === 'left') return '←'
			if (trimmed === 'right') return '→'
			if (trimmed === '#') return '#'
			return trimmed.charAt(0).toUpperCase() + trimmed.slice(1)
		})
		.join('+')
}
