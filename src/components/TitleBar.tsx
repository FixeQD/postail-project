import { useState, useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { platform } from '@tauri-apps/plugin-os'
import { Settings, Send, ChevronLeft, ChevronRight } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import icon from '@/assets/icon.png'
import { useDraftStore } from '@/stores/draftStore'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useAccountStore } from '@/stores/accountStore'
import { useMessageViewStore } from '@/stores/messageViewStore'
import { AccountSwitcher } from './TitleBar/AccountSwitcher'
import { NotificationCenter } from './TitleBar/NotificationCenter'
import { SearchBar } from './TitleBar/SearchBar'
import type { TitleBarProps } from '@/types/components/shared'

export function TitleBar({ isDashboard, onSearch, onOpenSettings, onOpenOutbox }: TitleBarProps) {
	const { activeAccount } = useAccountStore()
	const { isSending } = useDraftStore()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const [isMobile, setIsMobile] = useState<boolean | null>(null)
	const titleMeta = useMessageViewStore((s) => s.titleMeta)
	const selectedMessage = useMessageViewStore((s) => s.selectedMessage)
	const isLoading = useMessageViewStore((s) => s.isLoading)
	const isViewingMessage = selectedMessage !== null

	useEffect(() => {
		try {
			const p = platform()
			setIsMobile(p === 'android' || p === 'ios')
		} catch {
			const ua = navigator.userAgent.toLowerCase()
			setIsMobile(/android|iphone|ipad|ipod|mobile/i.test(ua))
		}
	}, [])

	if (isMobile === null || isMobile) {
		return null
	}

	const appWindow = getCurrentWindow()

	const minimize = () => appWindow.minimize()
	const toggleMaximize = () => appWindow.toggleMaximize()
	const close = () => appWindow.close()
	const startDrag = () => appWindow.startDragging()

	return (
		<div
			className='glass relative z-50 flex h-14 shrink-0 items-center justify-between border-b transition-colors select-none'
			style={{ borderColor: 'var(--border-subtle)' }}
			onMouseDown={startDrag}>
			{/* Subtle top highlight gradient */}
			<div className='pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-white/20 to-transparent opacity-50 dark:via-white/10' />

			{/* Loading animated gradient border */}
			<AnimatePresence>
				{isLoading && animationsEnabled && (
					<motion.div
						initial={{ opacity: 0 }}
						animate={{ opacity: 1 }}
						exit={{ opacity: 0 }}
						className='pointer-events-none absolute inset-x-0 bottom-[-1px] h-[2px] overflow-hidden'>
						<motion.div
							className='h-full w-full'
							style={{
								background: `linear-gradient(90deg, transparent, ${accentColor}, transparent)`,
							}}
							animate={{ x: ['-100%', '100%'] }}
							transition={{
								duration: 1.5,
								repeat: Infinity,
								ease: 'easeInOut',
							}}
						/>
					</motion.div>
				)}
			</AnimatePresence>

			{/* Left: Branding */}
			<div className='flex w-64 shrink-0 items-center gap-3.5 px-4 pl-6'>
				<motion.div
					className='relative flex h-8 w-8 items-center justify-center rounded-xl bg-gradient-to-br from-white to-slate-100 shadow-sm ring-1 ring-black/5 dark:from-slate-800 dark:to-slate-900 dark:ring-white/10'
					{...(animationsEnabled
						? { whileHover: { scale: 1.05, rotate: -5 }, whileTap: { scale: 0.95 } }
						: {})}>
					<img src={icon} alt='Postail' className='h-5 w-5 object-contain' />
				</motion.div>
				<span className='bg-gradient-to-br from-slate-900 to-slate-600 bg-clip-text text-lg font-bold tracking-tight text-transparent dark:from-white dark:to-slate-400'>
					Postail
				</span>
			</div>

			{/* Middle: Subject (Reading) or Search (Dashboard) */}
			<div className='flex flex-1 items-center justify-center px-4'>
				<AnimatePresence mode='wait'>
					{isViewingMessage ? (
						<motion.div
							key='subject'
							className='flex w-full max-w-2xl items-center gap-2'
							{...(animationsEnabled
								? {
										initial: {
											opacity: 0,
											y: 8,
											scale: 0.98,
											filter: 'blur(4px)',
										},
										animate: {
											opacity: 1,
											y: 0,
											scale: 1,
											filter: 'blur(0px)',
										},
										exit: {
											opacity: 0,
											y: -8,
											scale: 0.98,
											filter: 'blur(4px)',
										},
										transition: { duration: 0.25, ease: [0.16, 1, 0.3, 1] },
									}
								: {})}
							onMouseDown={(e) => e.stopPropagation()}>
							<div className='flex items-center gap-1 rounded-lg bg-[var(--surface-secondary)] p-0.5 ring-1 ring-[var(--border-subtle)] transition-all hover:ring-[var(--border-strong)]'>
								<motion.button
									type='button'
									onClick={(e) => {
										e.stopPropagation()
										titleMeta?.onPrev?.()
									}}
									disabled={isLoading || !titleMeta?.onPrev}
									{...(animationsEnabled
										? {
												whileHover: {
													backgroundColor: 'var(--surface-hover)',
													scale: 1.05,
												},
												whileTap: { scale: 0.9 },
											}
										: {})}
									className='flex h-6 w-7 items-center justify-center rounded-md text-[var(--text-secondary)] transition-all hover:text-[var(--text-primary)] disabled:opacity-30'>
									<ChevronLeft className='h-3.5 w-3.5' />
								</motion.button>
								<div className='h-3 w-px bg-[var(--border-subtle)]' />
								<motion.button
									type='button'
									onClick={(e) => {
										e.stopPropagation()
										titleMeta?.onNext?.()
									}}
									disabled={isLoading || !titleMeta?.onNext}
									{...(animationsEnabled
										? {
												whileHover: {
													backgroundColor: 'var(--surface-hover)',
													scale: 1.05,
												},
												whileTap: { scale: 0.9 },
											}
										: {})}
									className='flex h-6 w-7 items-center justify-center rounded-md text-[var(--text-secondary)] transition-all hover:text-[var(--text-primary)] disabled:opacity-30'>
									<ChevronRight className='h-3.5 w-3.5' />
								</motion.button>
							</div>

							<div className='relative flex min-w-0 flex-1 items-center justify-center'>
								<AnimatePresence mode='wait'>
									{isLoading ? (
										<motion.div
											key='loading'
											{...(animationsEnabled
												? {
														initial: { opacity: 0 },
														animate: { opacity: 1 },
														exit: { opacity: 0 },
													}
												: {})}
											className='relative h-6 w-48 overflow-hidden rounded-md bg-[var(--surface-active)]'>
											<motion.div
												className='absolute inset-0'
												style={{
													background: `linear-gradient(90deg, transparent, ${accentColor}33, transparent)`,
												}}
												animate={{ x: ['-100%', '100%'] }}
												transition={{
													duration: 1.5,
													repeat: Infinity,
													ease: 'easeInOut',
												}}
											/>
										</motion.div>
									) : (
										<motion.p
											key='subject-text'
											{...(animationsEnabled
												? {
														initial: { opacity: 0, y: 4 },
														animate: { opacity: 1, y: 0 },
														exit: { opacity: 0, y: -4 },
													}
												: {})}
											className='min-w-0 flex-1 truncate text-center text-sm font-medium text-[var(--text-primary)]'>
											{titleMeta?.subject || 'No Subject'}
										</motion.p>
									)}
								</AnimatePresence>
							</div>
						</motion.div>
					) : isDashboard && onSearch ? (
						<motion.div
							key='search'
							className='relative w-full max-w-xl'
							{...(animationsEnabled
								? {
										initial: {
											opacity: 0,
											y: -8,
											scale: 0.98,
											filter: 'blur(4px)',
										},
										animate: {
											opacity: 1,
											y: 0,
											scale: 1,
											filter: 'blur(0px)',
										},
										exit: {
											opacity: 0,
											y: 8,
											scale: 0.98,
											filter: 'blur(4px)',
										},
										transition: { duration: 0.25, ease: [0.16, 1, 0.3, 1] },
									}
								: {})}
							onMouseDown={(e) => e.stopPropagation()}>
							<SearchBar onSearch={onSearch} />
						</motion.div>
					) : null}
				</AnimatePresence>
			</div>

			{/* Right: Actions & Window Controls */}
			<div className='flex w-64 shrink-0 items-center justify-end gap-3 px-3'>
				{isDashboard && activeAccount && (
					<div className='flex items-center gap-1 border-r border-[var(--border-subtle)] pr-3'>
						<NotificationCenter />

						<motion.button
							className='relative flex h-8 w-8 items-center justify-center rounded-lg text-[var(--text-secondary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
							onClick={(e) => {
								e.stopPropagation()
								onOpenOutbox?.()
							}}
							{...(animationsEnabled
								? {
										animate: isSending ? { color: accentColor } : {},
										whileHover: { scale: 1.05 },
										whileTap: { scale: 0.9 },
									}
								: {})}
							onMouseDown={(e) => e.stopPropagation()}>
							<Send
								className='h-[18px] w-[18px]'
								style={isSending ? { color: accentColor } : {}}
							/>
							{isSending && animationsEnabled && (
								<motion.span
									className='absolute inset-0 rounded-lg opacity-20'
									animate={{ scale: [1, 1.4], opacity: [0.2, 0] }}
									transition={{ duration: 1, repeat: Infinity }}
									style={{ backgroundColor: accentColor }}
								/>
							)}
						</motion.button>

						<motion.button
							className='flex h-8 w-8 items-center justify-center rounded-lg text-[var(--text-secondary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
							onClick={(e) => {
								e.stopPropagation()
								onOpenSettings?.()
							}}
							{...(animationsEnabled ? { whileTap: { scale: 0.9 } } : {})}
							onMouseDown={(e) => e.stopPropagation()}>
							<Settings className='h-[18px] w-[18px]' />
						</motion.button>

						<div className='ml-1'>
							<AccountSwitcher onOpenSettings={onOpenSettings!} />
						</div>
					</div>
				)}

				{/* Window Controls - Sleek & Minimal */}
				<div className='flex items-center gap-1.5 pl-1'>
					<button
						onClick={(e) => {
							e.stopPropagation()
							minimize()
						}}
						onMouseDown={(e) => e.stopPropagation()}
						className='group flex h-7 w-7 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-all hover:bg-[var(--surface-active)] hover:text-[var(--text-primary)]'>
						<div className='h-0.5 w-3 rounded-full bg-current opacity-70 transition-opacity group-hover:opacity-100' />
					</button>
					<button
						onClick={(e) => {
							e.stopPropagation()
							toggleMaximize()
						}}
						onMouseDown={(e) => e.stopPropagation()}
						className='group flex h-7 w-7 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-all hover:bg-[var(--surface-active)] hover:text-[var(--text-primary)]'>
						<div className='h-2.5 w-2.5 rounded-[2px] border-[1.5px] border-current opacity-70 transition-opacity group-hover:opacity-100' />
					</button>
					<button
						onClick={(e) => {
							e.stopPropagation()
							close()
						}}
						onMouseDown={(e) => e.stopPropagation()}
						className='group flex h-7 w-7 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-all hover:bg-red-500 hover:text-white'>
						<svg
							className='h-3.5 w-3.5 opacity-70 transition-opacity group-hover:opacity-100'
							fill='none'
							viewBox='0 0 24 24'
							stroke='currentColor'
							strokeWidth={2.5}>
							<path
								strokeLinecap='round'
								strokeLinejoin='round'
								d='M6 18L18 6M6 6l12 12'
							/>
						</svg>
					</button>
				</div>
			</div>
		</div>
	)
}
