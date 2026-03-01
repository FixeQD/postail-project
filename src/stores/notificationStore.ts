import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

// ── Prefs ─────────────────────────────────────────────────────────

export interface NotificationPrefs {
	enabled: boolean
	sound: boolean
	importantOnly: boolean
}

const PREF_DEFAULTS: NotificationPrefs = {
	enabled: true,
	sound: false,
	importantOnly: false,
}

const PREF_KEYS: Record<keyof NotificationPrefs, string> = {
	enabled: 'notifications.enabled',
	sound: 'notifications.sound',
	importantOnly: 'notifications.importantOnly',
}

// ── UID baseline ───────────────────────────────────────────────────
// Key: `${accountId}:${mailbox}` → highest UID seen at startup.
// Any new mail event with newHighestUid > baseline[key] is "new since start".
export type UidBaseline = Record<string, number>

function baselineKey(accountId: string, mailbox: string): string {
	return `${accountId}:${mailbox}`
}

// ── In-app notification items ──────────────────────────────────────

export type AppNotificationType = 'new_mail' | 'sync_error' | 'system'

export interface AppNotification {
	id: string
	type: AppNotificationType
	title: string
	body: string
	accountId?: string
	accountEmail?: string
	mailbox?: string
	count?: number
	timestamp: number
	read: boolean
}

// ── Store ──────────────────────────────────────────────────────────

interface NotificationState {
	prefs: NotificationPrefs
	isLoadingPrefs: boolean

	// Baseline UIDs loaded at startup
	baseline: UidBaseline
	baselineReady: boolean

	items: AppNotification[]
	unreadCount: number
	centerOpen: boolean

	// Prefs
	loadPrefs: () => Promise<void>
	setPref: <K extends keyof NotificationPrefs>(
		key: K,
		value: NotificationPrefs[K]
	) => Promise<void>

	// Baseline
	loadBaseline: () => Promise<void>
	isNewMail: (accountId: string, mailbox: string, newHighestUid: number) => boolean
	updateBaseline: (accountId: string, mailbox: string, uid: number) => void

	// Center
	openCenter: () => void
	closeCenter: () => void
	toggleCenter: () => void

	// Items
	addNotification: (notif: Omit<AppNotification, 'id' | 'timestamp' | 'read'>) => void
	markRead: (id: string) => void
	markAllRead: () => void
	dismiss: (id: string) => void
	clearAll: () => void
}

const MAX_ITEMS = 50

export const useNotificationStore = create<NotificationState>((set, get) => ({
	prefs: PREF_DEFAULTS,
	isLoadingPrefs: false,
	baseline: {},
	baselineReady: false,
	items: [],
	unreadCount: 0,
	centerOpen: false,

	// ── Prefs ──────────────────────────────────────────────────────

	loadPrefs: async () => {
		set({ isLoadingPrefs: true })
		try {
			const all = await invoke<Record<string, string>>('get_all_settings')
			const prefs = { ...PREF_DEFAULTS }
			if (all[PREF_KEYS.enabled] !== undefined)
				prefs.enabled = all[PREF_KEYS.enabled] !== 'false'
			if (all[PREF_KEYS.sound] !== undefined) prefs.sound = all[PREF_KEYS.sound] === 'true'
			if (all[PREF_KEYS.importantOnly] !== undefined)
				prefs.importantOnly = all[PREF_KEYS.importantOnly] === 'true'
			set({ prefs })
		} catch (e) {
			console.error('[Notifications] Failed to load prefs:', e)
		} finally {
			set({ isLoadingPrefs: false })
		}
	},

	setPref: async (key, value) => {
		try {
			await invoke('set_setting', {
				key: PREF_KEYS[key as keyof NotificationPrefs],
				value: String(value),
			})
			set((s) => ({ prefs: { ...s.prefs, [key]: value } }))
		} catch (e) {
			console.error(`[Notifications] Failed to set pref ${key}:`, e)
		}
	},

	// ── Baseline ───────────────────────────────────────────────────

	loadBaseline: async () => {
		try {
			const rows =
				await invoke<Array<{ accountId: string; mailbox: string; uid: number }>>(
					'get_inbox_baseline_uids'
				)
			const baseline: UidBaseline = {}
			for (const row of rows) {
				baseline[baselineKey(row.accountId, row.mailbox)] = row.uid
			}
			set({ baseline, baselineReady: true })
		} catch (e) {
			console.error('[Notifications] Failed to load baseline UIDs:', e)
			set({ baselineReady: true }) // don't block forever
		}
	},

	isNewMail: (accountId, mailbox, newHighestUid) => {
		const { baseline, baselineReady } = get()
		if (!baselineReady) return false
		const key = baselineKey(accountId, mailbox)
		const known = baseline[key] ?? 0
		return newHighestUid > known
	},

	updateBaseline: (accountId, mailbox, uid) => {
		set((s) => ({
			baseline: { ...s.baseline, [baselineKey(accountId, mailbox)]: uid },
		}))
	},

	// ── Center ─────────────────────────────────────────────────────

	openCenter: () => {
		set({ centerOpen: true })
		get().markAllRead()
	},
	closeCenter: () => set({ centerOpen: false }),
	toggleCenter: () => {
		const { centerOpen, openCenter, closeCenter } = get()
		centerOpen ? closeCenter() : openCenter()
	},

	// ── Items ──────────────────────────────────────────────────────

	addNotification: (notif) => {
		const id = `notif_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`
		const item: AppNotification = { ...notif, id, timestamp: Date.now(), read: false }
		set((s) => {
			const next = [item, ...s.items].slice(0, MAX_ITEMS)
			return { items: next, unreadCount: next.filter((n) => !n.read).length }
		})
	},

	markRead: (id) => {
		set((s) => {
			const items = s.items.map((n) => (n.id === id ? { ...n, read: true } : n))
			return { items, unreadCount: items.filter((n) => !n.read).length }
		})
	},

	markAllRead: () =>
		set((s) => ({ items: s.items.map((n) => ({ ...n, read: true })), unreadCount: 0 })),

	dismiss: (id) => {
		set((s) => {
			const items = s.items.filter((n) => n.id !== id)
			return { items, unreadCount: items.filter((n) => !n.read).length }
		})
	},

	clearAll: () => set({ items: [], unreadCount: 0 }),
}))
