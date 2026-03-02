import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

// ── Prefs ──────────────────────────────────────────────────────────

export interface NotificationPrefs {
	// Master switches
	enabled: boolean // OS desktop notifications
	showInCenter: boolean // Always log to in-app center

	// Sound
	sound: boolean

	// Folder filters
	inboxOnly: boolean // INBOX only — overrides importantOnly
	importantOnly: boolean // starred/important folders only
	showForSent: boolean // include Sent folder (off by default)

	// Error alerts
	syncErrors: boolean

	// Content preview (shown in notification body)
	previewSender: boolean
	previewSubject: boolean

	// Grouping
	bundleMultiple: boolean // bundle N msgs into 1 notif
	minCountToNotify: number // 1 | 2 | 5 | 10
}

export const MIN_COUNT_OPTIONS = [1, 2, 5, 10] as const
export type MinCountOption = (typeof MIN_COUNT_OPTIONS)[number]

const D: NotificationPrefs = {
	enabled: true,
	showInCenter: true,
	sound: false,
	inboxOnly: false,
	importantOnly: false,
	showForSent: false,
	syncErrors: true,
	previewSender: true,
	previewSubject: true,
	bundleMultiple: false,
	minCountToNotify: 1,
}

const KEYS: Record<keyof NotificationPrefs, string> = {
	enabled: 'notifications.enabled',
	showInCenter: 'notifications.showInCenter',
	sound: 'notifications.sound',
	inboxOnly: 'notifications.inboxOnly',
	importantOnly: 'notifications.importantOnly',
	showForSent: 'notifications.showForSent',
	syncErrors: 'notifications.syncErrors',
	previewSender: 'notifications.previewSender',
	previewSubject: 'notifications.previewSubject',
	bundleMultiple: 'notifications.bundleMultiple',
	minCountToNotify: 'notifications.minCountToNotify',
}

function parseBool(v: string | undefined, def: boolean) {
	if (v === undefined) return def
	return v !== 'false' && v !== '0'
}
function parseNum(v: string | undefined, def: number) {
	if (v === undefined) return def
	const n = parseInt(v, 10)
	return isNaN(n) ? def : n
}

// ── UID baseline ───────────────────────────────────────────────────

export type UidBaseline = Record<string, number>

function bKey(accountId: string, mailbox: string) {
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

	baseline: UidBaseline
	baselineReady: boolean

	items: AppNotification[]
	unreadCount: number
	centerOpen: boolean

	loadPrefs: () => Promise<void>
	setPref: <K extends keyof NotificationPrefs>(
		key: K,
		value: NotificationPrefs[K]
	) => Promise<void>

	loadBaseline: () => Promise<void>
	isNewMail: (accountId: string, mailbox: string, newHighestUid: number) => boolean
	updateBaseline: (accountId: string, mailbox: string, uid: number) => void

	openCenter: () => void
	closeCenter: () => void
	toggleCenter: () => void

	addNotification: (notif: Omit<AppNotification, 'id' | 'timestamp' | 'read'>) => void
	markRead: (id: string) => void
	markAllRead: () => void
	dismiss: (id: string) => void
	clearAll: () => void
}

const MAX_ITEMS = 50

export const useNotificationStore = create<NotificationState>((set, get) => ({
	prefs: D,
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
			const p: NotificationPrefs = {
				enabled: parseBool(all[KEYS.enabled], D.enabled),
				showInCenter: parseBool(all[KEYS.showInCenter], D.showInCenter),
				sound: parseBool(all[KEYS.sound], D.sound),
				inboxOnly: parseBool(all[KEYS.inboxOnly], D.inboxOnly),
				importantOnly: parseBool(all[KEYS.importantOnly], D.importantOnly),
				showForSent: parseBool(all[KEYS.showForSent], D.showForSent),
				syncErrors: parseBool(all[KEYS.syncErrors], D.syncErrors),
				previewSender: parseBool(all[KEYS.previewSender], D.previewSender),
				previewSubject: parseBool(all[KEYS.previewSubject], D.previewSubject),
				bundleMultiple: parseBool(all[KEYS.bundleMultiple], D.bundleMultiple),
				minCountToNotify: parseNum(all[KEYS.minCountToNotify], D.minCountToNotify),
			}
			set({ prefs: p })
		} catch (e) {
			console.error('[Notifications] Failed to load prefs:', e)
		} finally {
			set({ isLoadingPrefs: false })
		}
	},

	setPref: async (key, value) => {
		try {
			await invoke('set_setting', {
				key: KEYS[key as keyof NotificationPrefs],
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
			for (const row of rows) baseline[bKey(row.accountId, row.mailbox)] = row.uid
			set({ baseline, baselineReady: true })
		} catch (e) {
			console.error('[Notifications] Failed to load baseline UIDs:', e)
			set({ baselineReady: true })
		}
	},

	isNewMail: (accountId, mailbox, newHighestUid) => {
		const { baseline, baselineReady } = get()
		if (!baselineReady) return false
		return newHighestUid > (baseline[bKey(accountId, mailbox)] ?? 0)
	},

	updateBaseline: (accountId, mailbox, uid) =>
		set((s) => ({ baseline: { ...s.baseline, [bKey(accountId, mailbox)]: uid } })),

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

	markRead: (id) =>
		set((s) => {
			const items = s.items.map((n) => (n.id === id ? { ...n, read: true } : n))
			return { items, unreadCount: items.filter((n) => !n.read).length }
		}),

	markAllRead: () =>
		set((s) => ({ items: s.items.map((n) => ({ ...n, read: true })), unreadCount: 0 })),

	dismiss: (id) =>
		set((s) => {
			const items = s.items.filter((n) => n.id !== id)
			return { items, unreadCount: items.filter((n) => !n.read).length }
		}),

	clearAll: () => set({ items: [], unreadCount: 0 }),
}))
