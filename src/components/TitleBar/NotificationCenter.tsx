import React, { useRef, useEffect } from 'react'

import { motion, AnimatePresence } from 'framer-motion'
import { Bell, Mail, AlertTriangle, Info, Trash2, CheckCheck, X, BellOff } from 'lucide-react'
import {
	useNotificationStore,
	type AppNotification,
	type AppNotificationType,
} from '@/stores/notificationStore'
import { useThemeStore } from '@/stores/themeStore'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'

// ── Helpers ────────────────────────────────────────────────────────

function relativeTime(
	ts: number,
	t: (key: string, opts?: Record<string, unknown>) => string
): string {
	const diff = Date.now() - ts
	const s = Math.floor(diff / 1000)
	if (s < 60) return t('notifications.time.justNow')
	const m = Math.floor(s / 60)
	if (m < 60) return t('notifications.time.minsAgo', { m })
	const h = Math.floor(m / 60)
	if (h < 24) return t('notifications.time.hoursAgo', { h })
	return t('notifications.time.daysAgo', { d: Math.floor(h / 24) })
}

const typeIcon: Record<AppNotificationType, React.ReactNode> = {
	new_mail: <Mail className='h-3.5 w-3.5' />,
	sync_error: <AlertTriangle className='h-3.5 w-3.5' />,
	system: <Info className='h-3.5 w-3.5' />,
}

const typeStyles: Record<AppNotificationType, { icon: string; bg: string }> = {
	new_mail: { icon: 'text-blue-400', bg: 'bg-blue-500/10' },
	sync_error: { icon: 'text-red-400', bg: 'bg-red-500/10' },
	system: { icon: 'text-[var(--text-tertiary)]', bg: 'bg-[var(--surface-active)]' },
}

// ── Item ──────────────────────────────────────────────────────────

function NotificationItem({ item }: { item: AppNotification }) {
	const dismiss = useNotificationStore((s) => s.dismiss)
	const markRead = useNotificationStore((s) => s.markRead)
	const { t } = useSettingsTranslation()
	const styles = typeStyles[item.type]

	return (
		<motion.div
			layout
			initial={{ opacity: 0, x: 8 }}
			animate={{ opacity: 1, x: 0 }}
			exit={{ opacity: 0, scale: 0.95, height: 0, margin: 0, transition: { duration: 0.18 } }}
			transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
			className='group relative flex gap-2.5 rounded-xl px-3 py-2.5 transition-colors hover:bg-[var(--surface-hover)]'
			onMouseEnter={() => !item.read && markRead(item.id)}>
			{/* Unread indicator */}
			{!item.read && (
				<span className='absolute top-3 left-1.5 h-1.5 w-1.5 rounded-full bg-[var(--accent-color)]' />
			)}

			{/* Icon */}
			<div
				className={`mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-lg ${styles.bg} ${styles.icon}`}>
				{typeIcon[item.type]}
			</div>

			{/* Content */}
			<div className='min-w-0 flex-1'>
				<p
					className={`text-[12px] leading-tight font-semibold ${item.read ? 'text-[var(--text-secondary)]' : 'text-[var(--text-primary)]'}`}>
					{item.title}
				</p>
				<p className='mt-0.5 line-clamp-2 text-[11px] leading-relaxed text-[var(--text-secondary)]'>
					{item.body}
				</p>
				<p className='mt-1 text-[10px] text-[var(--text-tertiary)]'>
					{relativeTime(item.timestamp, t)}
				</p>
			</div>

			{/* Dismiss */}
			<button
				type='button'
				onClick={(e) => {
					e.stopPropagation()
					dismiss(item.id)
				}}
				className='mt-0.5 hidden h-5 w-5 shrink-0 items-center justify-center rounded text-[var(--text-tertiary)] transition-colors group-hover:flex hover:text-[var(--text-primary)]'>
				<X className='h-3 w-3' />
			</button>
		</motion.div>
	)
}

// ── Panel ─────────────────────────────────────────────────────────

export function NotificationCenter() {
	const items = useNotificationStore((s) => s.items)
	const unreadCount = useNotificationStore((s) => s.unreadCount)
	const centerOpen = useNotificationStore((s) => s.centerOpen)
	const closeCenter = useNotificationStore((s) => s.closeCenter)
	const markAllRead = useNotificationStore((s) => s.markAllRead)
	const clearAll = useNotificationStore((s) => s.clearAll)
	const accentColor = useThemeStore((s) => s.accentColor)
	const { t } = useSettingsTranslation()

	const triggerRef = useRef<HTMLButtonElement>(null)
	const panelRef = useRef<HTMLDivElement>(null)
	const [panelPos, setPanelPos] = React.useState<{ top: number; right: number } | null>(null)

	useEffect(() => {
		if (!centerOpen) {
			setPanelPos(null)
			return
		}
		const r = triggerRef.current?.getBoundingClientRect()
		if (!r) return
		setPanelPos({ top: r.bottom + 6, right: window.innerWidth - r.right })
	}, [centerOpen])

	useEffect(() => {
		if (!centerOpen) return
		const handler = (e: MouseEvent) => {
			if (panelRef.current?.contains(e.target as Node)) return
			if (triggerRef.current?.contains(e.target as Node)) return
			closeCenter()
		}
		document.addEventListener('mousedown', handler)
		return () => document.removeEventListener('mousedown', handler)
	}, [centerOpen, closeCenter])

	return (
		<>
			{/* Bell trigger */}
			<button
				ref={triggerRef}
				type='button'
				className='relative flex h-7 w-7 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
				onMouseDown={(e) => e.stopPropagation()}
				onClick={(e) => {
					e.stopPropagation()
					useNotificationStore.getState().toggleCenter()
				}}>
				<Bell className='h-[15px] w-[15px]' />
				<AnimatePresence>
					{unreadCount > 0 && (
						<motion.span
							key='badge'
							initial={{ scale: 0 }}
							animate={{ scale: 1 }}
							exit={{ scale: 0 }}
							transition={{ type: 'spring', stiffness: 500, damping: 25 }}
							className='absolute -top-0.5 -right-0.5 flex h-[14px] min-w-[14px] items-center justify-center rounded-full px-0.5 text-[9px] font-bold text-white'
							style={{
								backgroundColor: accentColor,
								boxShadow: `0 0 6px ${accentColor}80`,
							}}>
							{unreadCount > 99 ? '99+' : unreadCount}
						</motion.span>
					)}
				</AnimatePresence>
			</button>

			{/* Panel */}
			<AnimatePresence>
				{centerOpen && panelPos && (
					<motion.div
						ref={panelRef}
						initial={{ opacity: 0, scale: 0.96, y: -6 }}
						animate={{ opacity: 1, scale: 1, y: 0 }}
						exit={{ opacity: 0, scale: 0.96, y: -4 }}
						transition={{ duration: 0.15, ease: [0.16, 1, 0.3, 1] }}
						className='fixed z-[300] w-80 overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--surface-glass)] shadow-2xl backdrop-blur-2xl'
						style={{
							top: panelPos.top,
							right: panelPos.right,
							transformOrigin: 'top right',
						}}
						onMouseDown={(e) => e.stopPropagation()}>
						{/* Header */}
						<div className='flex items-center justify-between border-b border-[var(--border-faint)] px-4 py-2.5'>
							<div className='flex items-center gap-2'>
								<Bell className='h-3.5 w-3.5 text-[var(--text-secondary)]' />
								<span className='text-[13px] font-semibold text-[var(--text-primary)]'>
									{t('notifications.center.title')}
								</span>
								{unreadCount > 0 && (
									<span
										className='rounded-full px-1.5 py-0.5 text-[9px] font-bold text-white'
										style={{ backgroundColor: accentColor }}>
										{unreadCount}
									</span>
								)}
							</div>
							<div className='flex items-center gap-0.5'>
								{unreadCount > 0 && (
									<button
										type='button'
										onClick={markAllRead}
										className='flex items-center gap-1 rounded-lg px-2 py-1 text-[11px] text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
										<CheckCheck className='h-3 w-3' />
										{t('notifications.center.allRead')}
									</button>
								)}
								{items.length > 0 && (
									<button
										type='button'
										onClick={clearAll}
										className='flex items-center gap-1 rounded-lg px-2 py-1 text-[11px] text-[var(--text-secondary)] transition-colors hover:bg-red-500/10 hover:text-red-400'>
										<Trash2 className='h-3 w-3' />
									</button>
								)}
							</div>
						</div>

						{/* List */}
						<div className='max-h-[400px] overflow-y-auto px-2 py-1.5'>
							{items.length === 0 ? (
								<div className='flex flex-col items-center gap-3 py-10 text-center'>
									<div className='flex h-10 w-10 items-center justify-center rounded-2xl bg-[var(--surface-active)]'>
										<BellOff className='h-5 w-5 text-[var(--text-tertiary)]' />
									</div>
									<div>
										<p className='text-[12px] font-medium text-[var(--text-secondary)]'>
											{t('notifications.center.empty')}
										</p>
										<p className='mt-0.5 text-[11px] text-[var(--text-tertiary)]'>
											{t('notifications.center.emptyDescription')}
										</p>
									</div>
								</div>
							) : (
								<AnimatePresence initial={false}>
									{items.map((item: AppNotification) => (
										<NotificationItem key={item.id} item={item} />
									))}
								</AnimatePresence>
							)}
						</div>
					</motion.div>
				)}
			</AnimatePresence>
		</>
	)
}
