import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface NotificationPrefs {
	enabled: boolean // master switch — OS notifications
	sound: boolean // play sound
	importantOnly: boolean // only notify for flagged/important mail
}

interface NotificationState {
	prefs: NotificationPrefs
	isLoading: boolean
	loadPrefs: () => Promise<void>
	setPref: <K extends keyof NotificationPrefs>(
		key: K,
		value: NotificationPrefs[K]
	) => Promise<void>
}

const DEFAULTS: NotificationPrefs = {
	enabled: true,
	sound: false,
	importantOnly: false,
}

const SETTING_KEYS: Record<keyof NotificationPrefs, string> = {
	enabled: 'notifications.enabled',
	sound: 'notifications.sound',
	importantOnly: 'notifications.importantOnly',
}

export const useNotificationStore = create<NotificationState>((set) => ({
	prefs: DEFAULTS,
	isLoading: false,

	loadPrefs: async () => {
		set({ isLoading: true })
		try {
			const all = await invoke<Record<string, string>>('get_all_settings')
			const prefs = { ...DEFAULTS }

			if (all[SETTING_KEYS.enabled] !== undefined)
				prefs.enabled = all[SETTING_KEYS.enabled] !== 'false'
			if (all[SETTING_KEYS.sound] !== undefined)
				prefs.sound = all[SETTING_KEYS.sound] === 'true'
			if (all[SETTING_KEYS.importantOnly] !== undefined)
				prefs.importantOnly = all[SETTING_KEYS.importantOnly] === 'true'

			set({ prefs })
		} catch (e) {
			console.error('Failed to load notification prefs:', e)
		} finally {
			set({ isLoading: false })
		}
	},

	setPref: async (key, value) => {
		try {
			await invoke('set_setting', {
				key: SETTING_KEYS[key],
				value: String(value),
			})
			set((state) => ({
				prefs: { ...state.prefs, [key]: value },
			}))
		} catch (e) {
			console.error(`Failed to set notification pref ${key}:`, e)
		}
	},
}))
