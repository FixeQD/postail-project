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
	subject?: string // newest message subject (optional)
	sender?: string // newest message sender address (optional)
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

function isSentFolder(mailbox: string): boolean {
	const u = mailbox.toUpperCase()
	return (
		u === 'SENT' ||
		u === '[GMAIL]/SENT MAIL' ||
		u === 'SENT ITEMS' ||
		u.startsWith('[GMAIL]/SENT') ||
		u === 'SENT MESSAGES'
	)
}

let listenersInitialized = false

function initListeners() {
	if (listenersInitialized) return
	listenersInitialized = true

	// Only idle_loop and poll_loop emit this — startup catch-up is always silent
	listen<NewMessagesPayload>('sync:new_messages', (event) => {
		const { accountId, mailbox, count, newHighestUid, subject, sender } = event.payload
		const store = useNotificationStore.getState()
		const { accounts } = useAccountStore.getState()
		const { prefs } = store

		// ── Baseline guard (never notify for catch-up at startup) ──
		if (!store.isNewMail(accountId, mailbox, newHighestUid)) return
		store.updateBaseline(accountId, mailbox, newHighestUid)

		// ── Count threshold ─────────────────────────────────────────
		if (count < prefs.minCountToNotify) return

		// ── Folder filtering ────────────────────────────────────────
		const isInbox = mailbox.toUpperCase() === 'INBOX'
		const isSent = isSentFolder(mailbox)

		if (prefs.inboxOnly && !isInbox) return // strictest: INBOX only
		if (!prefs.showForSent && isSent) return // skip Sent unless opted-in
		if (prefs.importantOnly && !isInbox) return // important = inbox equivalent here

		// ── Build title ────────────────────────────────────────────
		const account = accounts.find((a) => a.id === accountId)
		const accountEmail = account?.email ?? accountId

		const title =
			count === 1
				? t('notifications.messages.newSingle', { email: accountEmail })
				: t('notifications.messages.newMultiple', { count, email: accountEmail })

		// ── Build body with preview lines ───────────────────────────
		const lines: string[] = []
		if (prefs.previewSender && sender) lines.push(sender)
		if (prefs.previewSubject && subject) lines.push(subject)
		if (lines.length === 0) {
			lines.push(
				isInbox
					? t('notifications.messages.newMailInbox')
					: t('notifications.messages.newMailIn', { mailbox })
			)
		}
		const body = lines.join(' · ')

		// ── In-app center ───────────────────────────────────────────
		if (prefs.showInCenter) {
			store.addNotification({
				type: 'new_mail',
				title,
				body,
				accountId,
				accountEmail,
				mailbox,
				count,
			})
		}

		// ── OS notification ─────────────────────────────────────────
		if (prefs.enabled) {
			fireNativeNotification(title, body)
		}
	})

	listen<SyncErrorPayload>('sync:error', (event) => {
		const { accountId, error } = event.payload
		const store = useNotificationStore.getState()
		const { accounts } = useAccountStore.getState()

		if (!store.prefs.syncErrors) return

		const account = accounts.find((a) => a.id === accountId)
		const accountEmail = account?.email ?? accountId
		const title = t('notifications.messages.syncFailed', { email: accountEmail })

		if (store.prefs.showInCenter) {
			store.addNotification({
				type: 'sync_error',
				title,
				body: error,
				accountId,
				accountEmail,
			})
		}
		if (store.prefs.enabled) {
			fireNativeNotification(title, error)
		}
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
