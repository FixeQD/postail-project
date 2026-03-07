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
	const d = Math.floor(h / 24)
	return t('notifications.time.daysAgo', { d })
}

const typeIcon: Record<AppNotificationType, React.ReactNode> = {
	new_mail: <Mail className='h-4 w-4' />,
	sync_error: <AlertTriangle className='h-4 w-4' />,
	system: <Info className='h-4 w-4' />,
}

const typeColor: Record<AppNotificationType, string> = {
	new_mail: 'text-blue-400',
	sync_error: 'text-red-400',
	system: 'text-[var(--text-tertiary)]',
}

const typeBg: Record<AppNotificationType, string> = {
	new_mail: 'bg-blue-500/10',
	sync_error: 'bg-red-500/10',
	system: 'bg-[var(--surface-active)]',
}

// ── Item ──────────────────────────────────────────────────────────

function NotificationItem({ item }: { item: AppNotification; key?: string }) {
	const dismiss = useNotificationStore((s) => s.dismiss)
	const markRead = useNotificationStore((s) => s.markRead)
	const { t } = useSettingsTranslation()

	return (
		<motion.div
			layout
			initial={{ opacity: 0, x: 12 }}
			animate={{ opacity: 1, x: 0 }}
			exit={{ opacity: 0, x: 12, transition: { duration: 0.15 } }}
			transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
			className='group relative flex gap-3 rounded-xl p-3 transition-colors hover:bg-[var(--surface-hover)]'
			onMouseEnter={() => !item.read && markRead(item.id)}>
			{/* Unread dot */}
			{!item.read && (
				<span className='absolute top-3.5 left-1 h-1.5 w-1.5 rounded-full bg-[var(--accent-color)]' />
			)}

			{/* Icon */}
			<div
				className={`mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg ${typeBg[item.type]} ${typeColor[item.type]}`}>
				{typeIcon[item.type]}
			</div>

			{/* Content */}
			<div className='min-w-0 flex-1'>
				<p
					className={`text-sm leading-tight font-medium ${item.read ? 'text-[var(--text-secondary)]' : 'text-[var(--text-primary)]'}`}>
					{item.title}
				</p>
				<p className='mt-0.5 line-clamp-2 text-xs leading-relaxed text-[var(--text-secondary)]'>
					{item.body}
				</p>
				<p className='mt-1 text-[10px] text-[var(--text-tertiary)]'>
					{relativeTime(item.timestamp, t)}
				</p>
			</div>

			{/* Dismiss */}
			<button
				onClick={(e) => {
					e.stopPropagation()
					dismiss(item.id)
				}}
				className='mt-0.5 hidden h-5 w-5 shrink-0 items-center justify-center rounded text-[var(--text-tertiary)] transition-colors group-hover:flex hover:text-[var(--text-primary)]'>
				<X className='h-3.5 w-3.5' />
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

	const panelRef = useRef<HTMLDivElement>(null)

	// Close on outside click
	useEffect(() => {
		if (!centerOpen) return
		const handler = (e: MouseEvent) => {
			if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
				closeCenter()
			}
		}
		document.addEventListener('mousedown', handler)
		return () => document.removeEventListener('mousedown', handler)
	}, [centerOpen, closeCenter])

	return (
		<div ref={panelRef} className='relative'>
			{/* Bell trigger */}
			<motion.button
				className='relative flex h-8 w-8 items-center justify-center rounded-lg text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
				whileTap={{ scale: 0.88 }}
				onMouseDown={(e) => e.stopPropagation()}
				onClick={(e) => {
					e.stopPropagation()
					useNotificationStore.getState().toggleCenter()
				}}>
				<Bell className='h-[18px] w-[18px]' />
				<AnimatePresence>
					{unreadCount > 0 && (
						<motion.span
							key='badge'
							initial={{ scale: 0 }}
							animate={{ scale: 1 }}
							exit={{ scale: 0 }}
							transition={{ type: 'spring', stiffness: 500, damping: 25 }}
							className='absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[9px] font-bold text-white'
							style={{ backgroundColor: accentColor }}>
							{unreadCount > 99 ? '99+' : unreadCount}
						</motion.span>
					)}
				</AnimatePresence>
			</motion.button>

			{/* Panel */}
			<AnimatePresence>
				{centerOpen && (
					<motion.div
						initial={{ opacity: 0, scale: 0.95, y: -8 }}
						animate={{ opacity: 1, scale: 1, y: 0 }}
						exit={{ opacity: 0, scale: 0.95, y: -8 }}
						transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
						style={{ transformOrigin: 'top right' }}
						onMouseDown={(e) => e.stopPropagation()}
						className='absolute top-full right-0 z-[200] mt-2 w-80 overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--surface-glass)] shadow-2xl backdrop-blur-2xl'>
						{/* Header */}
						<div className='flex items-center justify-between border-b border-[var(--border-faint)] px-4 py-3'>
							<div className='flex items-center gap-2'>
								<Bell className='h-4 w-4 text-[var(--text-secondary)]' />
								<span className='text-sm font-semibold text-[var(--text-primary)]'>
									{t('notifications.center.title')}
								</span>
								{unreadCount > 0 && (
									<span
										className='rounded-full px-1.5 py-0.5 text-[10px] font-bold text-white'
										style={{ backgroundColor: accentColor }}>
										{unreadCount}
									</span>
								)}
							</div>
							<div className='flex items-center gap-1'>
								{unreadCount > 0 && (
									<button
										onClick={markAllRead}
										className='flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
										<CheckCheck className='h-3.5 w-3.5' />
										{t('notifications.center.allRead')}
									</button>
								)}
								{items.length > 0 && (
									<button
										onClick={clearAll}
										className='flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--text-secondary)] transition-colors hover:bg-red-500/10 hover:text-red-500'>
										<Trash2 className='h-3.5 w-3.5' />
										{t('notifications.center.clear')}
									</button>
								)}
							</div>
						</div>

						{/* List */}
						<div className='max-h-[420px] overflow-x-hidden overflow-y-auto px-2 py-2'>
							{items.length === 0 ? (
								<div className='flex flex-col items-center gap-3 py-10 text-center'>
									<div className='flex h-12 w-12 items-center justify-center rounded-2xl bg-[var(--surface-active)] ring-1 ring-[var(--border-faint)]'>
										<BellOff className='h-6 w-6 text-[var(--text-tertiary)]' />
									</div>
									<div>
										<p className='text-sm font-medium text-[var(--text-secondary)]'>
											{t('notifications.center.empty')}
										</p>
										<p className='mt-0.5 text-xs text-[var(--text-tertiary)]'>
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
		</div>
	)
}
