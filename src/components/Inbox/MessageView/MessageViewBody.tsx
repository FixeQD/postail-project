import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useThemeStore } from '@/stores/themeStore'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { EmailFreezeNotice, FreezeStats } from './EmailFreezeNotice'

import type { MessageViewBodyProps } from '@/types/components/shared'

interface PrepareEmailViewResult {
	hasExternalResources: boolean
	isPlainOnly: boolean
}

export const MessageViewBody = ({
	accountId,
	mailbox,
	uid,
	viewMode,
	allowExternalResources = false,
	onExternalDetected,
	onLoadingChange,
}: MessageViewBodyProps) => {
	const { t } = useTypedTranslation(['security', 'common'])
	const accentColor = useThemeStore((s) => s.accentColor)

	const containerRef = useRef<HTMLDivElement>(null)
	const rafPendingRef = useRef(false)

	const [pendingUrl, setPendingUrl] = useState<string | null>(null)
	const [warningOpen, setWarningOpen] = useState(false)
	const [frozenStats, setFrozenStats] = useState<FreezeStats | null>(null)

	// 1. Prepare content in Rust - builds the full HTML, processes inline images, rewrites external resources, and stores it for the protocol handler.
	useEffect(() => {
		onLoadingChange?.(true)

		invoke<PrepareEmailViewResult>('prepare_email_view', {
			accountId,
			mailbox,
			uid,
			accentColor,
			allowExternal: allowExternalResources,
			viewMode,
		})
			.then((res) => {
				if (res.hasExternalResources) onExternalDetected?.()
				// Tell the native webview to reload so it fetches the freshly prepared HTML
				invoke('reload_email_webview').catch(console.error)
			})
			.catch(console.error)
			.finally(() => onLoadingChange?.(false))
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [accountId, mailbox, uid, viewMode, accentColor, allowExternalResources])

	// 2. Spawn / destroy the child webview per message
	useEffect(() => {
		const win = getCurrentWindow()
		invoke('create_email_webview', { window: win }).catch(console.error)

		return () => {
			invoke('destroy_email_webview').catch(console.error)
		}
	}, [accountId, mailbox, uid])

	// 3. ResizeObserver & Scroll — sync native webview bounds to the placeholder div
	useEffect(() => {
		const el = containerRef.current
		if (!el) return

		const syncBounds = () => {
			rafPendingRef.current = false
			const rect = el.getBoundingClientRect()
			const dpr = window.devicePixelRatio ?? 1

			invoke('update_email_webview_bounds', {
				x: rect.x / dpr,
				y: rect.y / dpr,
				width: rect.width / dpr,
				height: rect.height / dpr,
			}).catch(console.error)
		}

		const ro = new ResizeObserver(() => {
			if (rafPendingRef.current) return
			rafPendingRef.current = true
			requestAnimationFrame(syncBounds)
		})

		ro.observe(el)

		const scrollParent = el.closest('.overflow-y-auto') || window
		const onScroll = () => {
			if (rafPendingRef.current) return
			rafPendingRef.current = true
			requestAnimationFrame(syncBounds)
		}
		scrollParent.addEventListener('scroll', onScroll, { passive: true })

		requestAnimationFrame(syncBounds)

		return () => {
			ro.disconnect()
			scrollParent.removeEventListener('scroll', onScroll)
		}
	}, [frozenStats])

	// 4. Watchdog freeze/resume listeners
	useEffect(() => {
		const unlistenFrozen = listen<FreezeStats>('email_webview_frozen', (event) => {
			setFrozenStats(event.payload)
		})

		const unlistenResumed = listen('email_webview_resumed', () => {
			setFrozenStats(null)
		})

		return () => {
			unlistenFrozen.then((fn) => fn())
			unlistenResumed.then((fn) => fn())
		}
	}, [])

	return (
		<div className='flex h-full w-full flex-1 flex-col'>
			{frozenStats && (
				<div className='z-10 w-full shrink-0'>
					<EmailFreezeNotice stats={frozenStats} onDismiss={() => setFrozenStats(null)} />
				</div>
			)}

			<div
				ref={containerRef}
				id='email-webview-container'
				className='relative min-h-[400px] w-full flex-1'
			/>

			<ConfirmationDialog
				open={warningOpen}
				onOpenChange={setWarningOpen}
				title={t('security:externalLink.title')}
				description={t('security:externalLink.description')}
				cancelLabel={t('security:externalLink.cancel')}
				confirmLabel={t('security:externalLink.open')}
				onConfirm={() => {
					if (pendingUrl) openUrl(pendingUrl)
					setWarningOpen(false)
					setPendingUrl(null)
				}}
				confirmClassName='w-full border-0 font-semibold shadow-lg bg-sky-500 text-white hover:bg-sky-600'>
				<div className='flex flex-col gap-1.5 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-3'>
					<p className='text-[10px] font-bold tracking-wider text-[var(--text-tertiary)] uppercase'>
						Target URL
					</p>
					<p className='font-mono text-xs break-all text-[var(--text-primary)]'>
						{pendingUrl}
					</p>
				</div>
			</ConfirmationDialog>
		</div>
	)
}
