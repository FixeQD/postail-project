import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

interface AppSettings {
	// General - Interface
	'zen-mode': boolean
	// General - Reading
	'mark-as-read-delay': number // seconds: 0=immediate, 2, 5, -1=manual
	'thread-view': boolean
	'preview-lines': number // 1, 2, 3
	// General - Behavior
	'undo-send-delay': number
	'confirm-before-delete': boolean
	// General - Startup
	'minimize-to-tray': boolean
	// General - Storage
	'data-path': string
	// Security
	'auto-lock-timeout': number
	'encrypt-attachments': boolean
	'clear-clipboard-delay': number // seconds: 0=disabled, 30, 60
	// Privacy
	'block-external-images': boolean
	'block-read-receipts': boolean
	'disable-link-preview': boolean
	'strip-attachment-metadata': boolean
	// Appearance
	'compact-mode': boolean
	'show-avatars': boolean
	// Composing
	'read-receipts-enabled': boolean
	'auto-save-drafts': boolean
	'spell-check': boolean
	'default-reply-position': 'top' | 'bottom'
	'signature-enabled': boolean
	'signature-content': string
	'warn-large-attachment-mb': number // 0=disabled, 10, 25, 50
}

interface SettingsState {
	settings: AppSettings
	isLoading: boolean
	setSetting: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => Promise<void>
	loadSettings: () => Promise<void>
}

const DEFAULT_SETTINGS: AppSettings = {
	'zen-mode': false,
	'mark-as-read-delay': 2,
	'thread-view': true,
	'preview-lines': 2,
	'undo-send-delay': 10,
	'confirm-before-delete': true,
	'minimize-to-tray': false,
	'data-path': '',
	'auto-lock-timeout': 0,
	'encrypt-attachments': false,
	'clear-clipboard-delay': 0,
	'block-external-images': true,
	'block-read-receipts': true,
	'disable-link-preview': false,
	'strip-attachment-metadata': false,
	'compact-mode': false,
	'show-avatars': true,
	'read-receipts-enabled': false,
	'auto-save-drafts': true,
	'spell-check': true,
	'default-reply-position': 'top',
	'signature-enabled': false,
	'signature-content': '',
	'warn-large-attachment-mb': 25,
}

function parseBool(val: string): boolean {
	return val === 'true'
}

function parseNum(val: string): number {
	return parseInt(val, 10)
}

export const useSettingsStore = create<SettingsState>((set) => ({
	settings: DEFAULT_SETTINGS,
	isLoading: false,

	setSetting: async (key, value) => {
		try {
			await invoke('set_setting', { key, value: String(value) })
			set((state) => ({ settings: { ...state.settings, [key]: value } }))
		} catch (error) {
			console.error(`Failed to set setting ${key}:`, error)
		}
	},

	loadSettings: async () => {
		set({ isLoading: true })
		try {
			const raw = await invoke<Record<string, string>>('get_all_settings')
			const s = { ...DEFAULT_SETTINGS }

			if ('zen-mode' in raw) s['zen-mode'] = parseBool(raw['zen-mode'])
			if ('mark-as-read-delay' in raw)
				s['mark-as-read-delay'] = parseNum(raw['mark-as-read-delay'])
			if ('thread-view' in raw) s['thread-view'] = parseBool(raw['thread-view'])
			if ('preview-lines' in raw) s['preview-lines'] = parseNum(raw['preview-lines'])
			if ('undo-send-delay' in raw) s['undo-send-delay'] = parseNum(raw['undo-send-delay'])
			if ('confirm-before-delete' in raw)
				s['confirm-before-delete'] = parseBool(raw['confirm-before-delete'])
			if ('minimize-to-tray' in raw)
				s['minimize-to-tray'] = parseBool(raw['minimize-to-tray'])
			if ('data-path' in raw) s['data-path'] = raw['data-path']
			if ('auto-lock-timeout' in raw)
				s['auto-lock-timeout'] = parseNum(raw['auto-lock-timeout'])
			if ('encrypt-attachments' in raw)
				s['encrypt-attachments'] = parseBool(raw['encrypt-attachments'])
			if ('clear-clipboard-delay' in raw)
				s['clear-clipboard-delay'] = parseNum(raw['clear-clipboard-delay'])
			if ('block-external-images' in raw)
				s['block-external-images'] = parseBool(raw['block-external-images'])
			if ('block-read-receipts' in raw)
				s['block-read-receipts'] = parseBool(raw['block-read-receipts'])
			if ('disable-link-preview' in raw)
				s['disable-link-preview'] = parseBool(raw['disable-link-preview'])
			if ('strip-attachment-metadata' in raw)
				s['strip-attachment-metadata'] = parseBool(raw['strip-attachment-metadata'])
			if ('compact-mode' in raw) s['compact-mode'] = parseBool(raw['compact-mode'])
			if ('show-avatars' in raw) s['show-avatars'] = parseBool(raw['show-avatars'])
			if ('read-receipts-enabled' in raw)
				s['read-receipts-enabled'] = parseBool(raw['read-receipts-enabled'])
			if ('auto-save-drafts' in raw)
				s['auto-save-drafts'] = parseBool(raw['auto-save-drafts'])
			if ('spell-check' in raw) s['spell-check'] = parseBool(raw['spell-check'])
			if ('default-reply-position' in raw) {
				const pos = raw['default-reply-position']
				if (pos === 'top' || pos === 'bottom') s['default-reply-position'] = pos
			}
			if ('signature-enabled' in raw)
				s['signature-enabled'] = parseBool(raw['signature-enabled'])
			if ('signature-content' in raw) s['signature-content'] = raw['signature-content']
			if ('warn-large-attachment-mb' in raw)
				s['warn-large-attachment-mb'] = parseNum(raw['warn-large-attachment-mb'])

			set({ settings: s })
		} catch (error) {
			console.error('Failed to load settings:', error)
		} finally {
			set({ isLoading: false })
		}
	},
}))
