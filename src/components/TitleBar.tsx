import { useState, useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { platform } from '@tauri-apps/plugin-os'
import { Search, Settings, Send } from 'lucide-react'
import { motion } from 'framer-motion'
import icon from '@/assets/icon.png'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useDraftStore } from '@/stores/draftStore'
import { useThemeStore } from '@/stores/themeStore'
import { useAccountStore } from '@/stores/accountStore'
import { useMessageViewStore } from '@/stores/messageViewStore'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { AccountSwitcher } from './TitleBar/AccountSwitcher'
import { NotificationCenter } from './TitleBar/NotificationCenter'
import type { TitleBarProps } from '@/types/components/shared'

export function TitleBar({ isDashboard, onSearch, onOpenSettings, onOpenOutbox }: TitleBarProps) {
	const { activeAccount } = useAccountStore()
	const { t } = useTypedTranslation()
	const { isSending } = useDraftStore()
	const accentColor = useThemeStore((s) => s.accentColor)
	const [isMobile, setIsMobile] = useState<boolean | null>(null)
	const [searchQuery, setSearchQuery] = useState('')
	const [searchFocused, setSearchFocused] = useState(false)
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
			className='glass relative z-50 flex h-14 shrink-0 items-center justify-between border-b select-none'
			style={{ borderColor: 'var(--border-subtle)' }}
			onMouseDown={startDrag}>
			{/* Subtle top highlight */}
			<div className='pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-black/[0.05] to-transparent dark:via-white/[0.06]' />

			{/* Left: Branding */}
			<div className='flex w-64 shrink-0 items-center gap-3 px-4 pl-6'>
				<motion.img
					src={icon}
					alt='Postail'
					className='h-8 w-8'
					initial={false}
					whileHover={{ rotate: [0, -8, 8, 0] }}
					transition={{ duration: 0.4 }}
				/>
				<span className='gradient-text-brand text-lg font-bold tracking-tight'>
					Postail
				</span>
			</div>

			{/* Middle: Subject when reading a message, Search otherwise */}
			<div className='flex flex-1 items-center justify-center px-4'>
				{isViewingMessage ? (
					<motion.div
						className='flex w-full max-w-2xl items-center gap-1'
						initial={{ opacity: 0, y: -4 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ duration: 0.18, ease: 'easeOut' }}
						onMouseDown={(e) => e.stopPropagation()}>
						<motion.button
							type='button'
							onClick={(e) => {
								e.stopPropagation()
								titleMeta?.onPrev?.()
							}}
							disabled={isLoading || !titleMeta?.onPrev}
							whileTap={{ scale: 0.88 }}
							className='flex h-7 w-7 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-white/[0.07] hover:text-slate-200 disabled:cursor-default disabled:opacity-30 disabled:hover:bg-transparent'>
							<ChevronLeft className='h-4 w-4' />
						</motion.button>
						{isLoading ? (
							<div
								className='h-4 w-48 rounded'
								style={{
									backgroundImage:
										'linear-gradient(90deg, rgba(var(--accent-rgb), 0.05) 25%, rgba(var(--accent-rgb), 0.12) 50%, rgba(var(--accent-rgb), 0.05) 75%)',
									backgroundSize: '200% 100%',
									animation: 'shimmer 1.5s ease-in-out infinite',
								}}
							/>
						) : (
							<p className='min-w-0 flex-1 truncate text-center text-sm font-medium text-slate-200'>
								{titleMeta?.subject}
							</p>
						)}
						<motion.button
							type='button'
							onClick={(e) => {
								e.stopPropagation()
								titleMeta?.onNext?.()
							}}
							disabled={isLoading || !titleMeta?.onNext}
							whileTap={{ scale: 0.88 }}
							className='flex h-7 w-7 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-white/[0.07] hover:text-slate-200 disabled:cursor-default disabled:opacity-30 disabled:hover:bg-transparent'>
							<ChevronRight className='h-4 w-4' />
						</motion.button>
					</motion.div>
				) : isDashboard ? (
					<motion.div
						className='relative w-full max-w-2xl'
						animate={{ scale: searchFocused ? 1.01 : 1 }}
						transition={{ duration: 0.2, ease: 'easeOut' }}>
						<div className='pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3.5'>
							<Search
								className='h-4 w-4 transition-colors duration-200'
								style={{ color: searchFocused ? accentColor : undefined }}
							/>
						</div>
						<input
							type='text'
							data-search-input
							value={searchQuery}
							onChange={(e) => {
								setSearchQuery(e.target.value)
								onSearch?.(e.target.value)
							}}
							onFocus={() => setSearchFocused(true)}
							onBlur={() => setSearchFocused(false)}
							onMouseDown={(e) => e.stopPropagation()}
							placeholder={t('inbox:search.placeholder')}
							className={`block w-full rounded-xl border py-2.5 pr-4 pl-10 text-sm transition-all duration-200 focus:outline-none ${
								searchFocused
									? 'bg-background/90 text-foreground shadow-lg ring-1'
									: 'bg-background/60 text-foreground placeholder:text-muted-foreground hover:bg-background/80 border-[var(--border-subtle)]'
							}`}
							style={
								searchFocused
									? {
											borderColor: `rgba(var(--accent-rgb), 0.3)`,
											boxShadow: `0 4px 12px -2px rgba(var(--accent-rgb), 0.05), 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
										}
									: undefined
							}
						/>
						<motion.div
							className='pointer-events-none absolute inset-x-4 -bottom-1 h-4 rounded-full blur-md'
							style={{ backgroundColor: `rgba(var(--accent-rgb), 0.1)` }}
							initial={false}
							animate={{ opacity: searchFocused ? 1 : 0 }}
							transition={{ duration: 0.2 }}
						/>
					</motion.div>
				) : null}
			</div>

			<div className='flex w-64 shrink-0 items-center justify-end gap-2 px-2'>
				{/* Dashboard Actions */}
				{isDashboard && activeAccount && (
					<div
						className='mr-3 flex items-center gap-1 border-r pr-3'
						style={{ borderColor: 'var(--border-subtle)' }}>
						<NotificationCenter />

						<motion.button
							id='outbox-button'
							className='text-muted-foreground hover:text-foreground flex h-8 w-8 items-center justify-center rounded-lg transition-colors hover:bg-[var(--surface-hover)]'
							onClick={(e) => {
								e.stopPropagation()
								onOpenOutbox?.()
							}}
							animate={
								isSending
									? {
											scale: [1, 1.15, 1],
											rotate: [0, 12, -12, 0],
											color: accentColor,
										}
									: {}
							}
							transition={
								isSending
									? {
											duration: 0.5,
											repeat: Infinity,
											repeatType: 'loop',
										}
									: {}
							}
							whileTap={{ scale: 0.9 }}
							onMouseDown={(e) => e.stopPropagation()}>
							<Send className='h-[18px] w-[18px]' />
						</motion.button>

						<motion.button
							className='text-muted-foreground hover:text-foreground flex h-8 w-8 items-center justify-center rounded-lg transition-colors hover:bg-[var(--surface-hover)]'
							onClick={(e) => {
								e.stopPropagation()
								onOpenSettings?.()
							}}
							whileTap={{ scale: 0.9 }}
							onMouseDown={(e) => e.stopPropagation()}>
							<Settings className='h-[18px] w-[18px]' />
						</motion.button>

						{/* Avatar / Account Switcher */}
						<AccountSwitcher onOpenSettings={onOpenSettings!} />
					</div>
				)}

				{/* Window Controls */}
				<div className='flex h-full items-center gap-0.5 pl-1'>
					{/* Minimize */}
					<motion.button
						onClick={(e) => {
							e.stopPropagation()
							minimize()
						}}
						onMouseDown={(e) => e.stopPropagation()}
						whileHover={{ scale: 1.1 }}
						whileTap={{ scale: 0.85 }}
						className='group flex h-7 w-7 items-center justify-center rounded-full transition-colors hover:bg-[var(--surface-active)]'>
						<div className='bg-muted-foreground group-hover:bg-foreground h-0.5 w-2.5 rounded-full transition-colors' />
					</motion.button>

					{/* Maximize */}
					<motion.button
						onClick={(e) => {
							e.stopPropagation()
							toggleMaximize()
						}}
						onMouseDown={(e) => e.stopPropagation()}
						whileHover={{ scale: 1.1 }}
						whileTap={{ scale: 0.85 }}
						className='group flex h-7 w-7 items-center justify-center rounded-full transition-colors hover:bg-[var(--surface-active)]'>
						<div className='border-muted-foreground group-hover:border-foreground h-2.5 w-2.5 rounded-[2px] border-[1.5px] transition-colors' />
					</motion.button>

					{/* Close */}
					<motion.button
						onClick={(e) => {
							e.stopPropagation()
							close()
						}}
						onMouseDown={(e) => e.stopPropagation()}
						whileHover={{ scale: 1.1 }}
						whileTap={{ scale: 0.85 }}
						className='group flex h-7 w-7 items-center justify-center rounded-full transition-colors hover:bg-red-500/80'>
						<svg
							className='text-muted-foreground h-3.5 w-3.5 transition-colors group-hover:text-white'
							fill='none'
							stroke='currentColor'
							viewBox='0 0 24 24'>
							<path
								strokeLinecap='round'
								strokeLinejoin='round'
								strokeWidth={2.5}
								d='M6 18L18 6M6 6l12 12'
							/>
						</svg>
					</motion.button>
				</div>
			</div>
		</div>
	)
}
