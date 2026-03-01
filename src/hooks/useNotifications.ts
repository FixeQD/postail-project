import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useAccountStore } from '@/stores/accountStore'
import { useNotificationStore } from '@/stores/notificationStore'

interface NewMessagesPayload {
	accountId: string
	mailbox: string
	count: number
}

interface SyncErrorPayload {
	accountId: string
	error: string
}

async function ensureOsPermission(): Promise<boolean> {
	if (!('Notification' in window)) return false
	if (Notification.permission === 'granted') return true
	if (Notification.permission === 'denied') return false
	const result = await Notification.requestPermission()
	return result === 'granted'
}

function fireOsNotification(title: string, body: string) {
	if (Notification.permission !== 'granted') return
	new Notification(title, { body })
}

export function useNotifications() {
	const prefs = useNotificationStore((s) => s.prefs)
	const loadPrefs = useNotificationStore((s) => s.loadPrefs)
	const addNotification = useNotificationStore((s) => s.addNotification)
	const accounts = useAccountStore((s) => s.accounts)

	useEffect(() => {
		loadPrefs()
	}, [loadPrefs])

	useEffect(() => {
		if (prefs.enabled) ensureOsPermission()
	}, [prefs.enabled])

	// ── New mail ────────────────────────────────────────────────────
	useEffect(() => {
		const unlisten = listen<NewMessagesPayload>('sync:new_messages', (event) => {
			const { accountId, mailbox, count } = event.payload
			if (prefs.importantOnly && mailbox.toUpperCase() !== 'INBOX') return

			const account = accounts.find((a) => a.id === accountId)
			const accountEmail = account?.email ?? accountId

			const title =
				count === 1
					? `New message — ${accountEmail}`
					: `${count} new messages — ${accountEmail}`
			const body =
				mailbox.toUpperCase() === 'INBOX'
					? 'New mail in your inbox.'
					: `New mail in ${mailbox}.`

			// Always push to in-app center
			addNotification({
				type: 'new_mail',
				title,
				body,
				accountId,
				accountEmail,
				mailbox,
				count,
			})

			// OS notification only if enabled
			if (prefs.enabled) fireOsNotification(title, body)
		})

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [prefs.enabled, prefs.importantOnly, accounts, addNotification])

	// ── Sync errors ─────────────────────────────────────────────────
	useEffect(() => {
		const unlisten = listen<SyncErrorPayload>('sync:error', (event) => {
			const { accountId, error } = event.payload
			const account = accounts.find((a) => a.id === accountId)
			const accountEmail = account?.email ?? accountId

			addNotification({
				type: 'sync_error',
				title: `Sync failed — ${accountEmail}`,
				body: error,
				accountId,
				accountEmail,
			})
		})

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [accounts, addNotification])
}
