import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useAccountStore } from '@/stores/accountStore'
import { useNotificationStore } from '@/stores/notificationStore'
import i18n from '@/i18n'

interface NewMessagesPayload {
	accountId: string
	mailbox: string
	count: number
	newHighestUid: number
}

interface SyncErrorPayload {
	accountId: string
	error: string
}

function t(key: string, opts?: Record<string, unknown>): string {
	return i18n.t(key, { ns: 'settings', ...opts })
}

async function fireNativeNotification(title: string, body: string) {
	try {
		await invoke('show_notification', { title, body })
	} catch (e) {
		console.warn('[Notifications] Native notification failed:', e)
	}
}

let listenersInitialized = false

function initListeners() {
	if (listenersInitialized) return
	listenersInitialized = true

	// Only idle_loop and poll_loop emit this event
	listen<NewMessagesPayload>('sync:new_messages', (event) => {
		const { accountId, mailbox, count, newHighestUid } = event.payload
		const store = useNotificationStore.getState()
		const { accounts } = useAccountStore.getState()

		if (!store.isNewMail(accountId, mailbox, newHighestUid)) return

		store.updateBaseline(accountId, mailbox, newHighestUid)

		if (store.prefs.importantOnly && mailbox.toUpperCase() !== 'INBOX') return

		const account = accounts.find((a) => a.id === accountId)
		const accountEmail = account?.email ?? accountId

		const title =
			count === 1
				? t('notifications.messages.newSingle', { email: accountEmail })
				: t('notifications.messages.newMultiple', { count, email: accountEmail })

		const body =
			mailbox.toUpperCase() === 'INBOX'
				? t('notifications.messages.newMailInbox')
				: t('notifications.messages.newMailIn', { mailbox })

		store.addNotification({
			type: 'new_mail',
			title,
			body,
			accountId,
			accountEmail,
			mailbox,
			count,
		})

		if (store.prefs.enabled) fireNativeNotification(title, body)
	})

	listen<SyncErrorPayload>('sync:error', (event) => {
		const { accountId, error } = event.payload
		const { addNotification } = useNotificationStore.getState()
		const { accounts } = useAccountStore.getState()

		const account = accounts.find((a) => a.id === accountId)
		const accountEmail = account?.email ?? accountId

		addNotification({
			type: 'sync_error',
			title: t('notifications.messages.syncFailed', { email: accountEmail }),
			body: error,
			accountId,
			accountEmail,
		})
	})
}

export function useNotifications(ready: boolean) {
	const loadPrefs = useNotificationStore((s) => s.loadPrefs)

	useEffect(() => {
		if (!ready) return
		loadPrefs()
		initListeners()
	}, [ready, loadPrefs])
}
