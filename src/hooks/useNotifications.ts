import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useAccountStore } from '@/stores/accountStore'
import { useNotificationStore } from '@/stores/notificationStore'

interface NewMessagesPayload {
	accountId: string
	mailbox: string
	count: number
}

async function requestPermission(): Promise<boolean> {
	if (!('Notification' in window)) return false
	if (Notification.permission === 'granted') return true
	if (Notification.permission === 'denied') return false
	const result = await Notification.requestPermission()
	return result === 'granted'
}

function sendNotification(title: string, body: string) {
	if (Notification.permission !== 'granted') return
	new Notification(title, {
		body,
		// icon
	})
}

export function useNotifications() {
	const prefs = useNotificationStore((s) => s.prefs)
	const loadPrefs = useNotificationStore((s) => s.loadPrefs)
	const accounts = useAccountStore((s) => s.accounts)

	// Load prefs once on mount
	useEffect(() => {
		loadPrefs()
	}, [loadPrefs])

	// Request OS permission once notifications are enabled
	useEffect(() => {
		if (prefs.enabled) {
			requestPermission()
		}
	}, [prefs.enabled])

	// Listen for new mail events from backend
	useEffect(() => {
		if (!prefs.enabled) return

		const unlisten = listen<NewMessagesPayload>('sync:new_messages', (event) => {
			const { accountId, mailbox, count } = event.payload

			// importantOnly filter: for now skip non-INBOX mailboxes when enabled
			if (prefs.importantOnly && mailbox.toUpperCase() !== 'INBOX') return

			const account = accounts.find((a) => a.id === accountId)
			const accountLabel = account?.email ?? accountId

			const title =
				count === 1
					? `New message — ${accountLabel}`
					: `${count} new messages — ${accountLabel}`

			const body =
				mailbox.toUpperCase() === 'INBOX'
					? 'You have new mail in your inbox.'
					: `You have new mail in ${mailbox}.`

			sendNotification(title, body)
		})

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [prefs.enabled, prefs.importantOnly, accounts])
}
