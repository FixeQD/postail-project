import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

interface AppSettings {
	'zen-mode': boolean
	'undo-send-delay': number
	'data-path': string
	'auto-lock-timeout': number
	'block-external-images': boolean
}

interface SettingsState {
	settings: AppSettings
	isLoading: boolean

	// Actions
	setSetting: (key: keyof AppSettings, value: any) => Promise<void>
	loadSettings: () => Promise<void>
}

const DEFAULT_SETTINGS: AppSettings = {
	'zen-mode': false,
	'undo-send-delay': 10,
	'data-path': '',
	'auto-lock-timeout': 0, // Disabled
	'block-external-images': true,
}

export const useSettingsStore = create<SettingsState>((set) => ({
	settings: DEFAULT_SETTINGS,
	isLoading: false,

	setSetting: async (key, value) => {
		try {
			// Persist to backend
			await invoke('set_setting', { key, value: String(value) })
			
			// Update local state
			set((state) => ({
				settings: { ...state.settings, [key]: value },
			}))
		} catch (error) {
			console.error(`Failed to set setting ${key}:`, error)
		}
	},

	loadSettings: async () => {
		set({ isLoading: true })
		try {
			const allSettings = await invoke<Record<string, string>>('get_all_settings')
			
			// Map backend strings to typed settings
			const mappedSettings = { ...DEFAULT_SETTINGS }
			
			if (allSettings['zen-mode']) mappedSettings['zen-mode'] = allSettings['zen-mode'] === 'true'
			if (allSettings['undo-send-delay']) mappedSettings['undo-send-delay'] = parseInt(allSettings['undo-send-delay'])
			if (allSettings['data-path']) mappedSettings['data-path'] = allSettings['data-path']
			if (allSettings['auto-lock-timeout']) mappedSettings['auto-lock-timeout'] = parseInt(allSettings['auto-lock-timeout'])
			if (allSettings['block-external-images']) mappedSettings['block-external-images'] = allSettings['block-external-images'] === 'true'

			set({ settings: mappedSettings })
		} catch (error) {
			console.error('Failed to load settings:', error)
		} finally {
			set({ isLoading: false })
		}
	},
}))
