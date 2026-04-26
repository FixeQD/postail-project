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

// Window control button
function WinBtn({
	onClick,
	color,
	title,
	children,
}: {
	onClick: () => void
	color?: string
	title?: string
	children?: React.ReactNode
}) {
	return (
		<button
			type='button'
			onClick={(e) => {
				e.stopPropagation()
				onClick()
			}}
			onMouseDown={(e) => e.stopPropagation()}
			title={title}
			className='group relative flex h-3.5 w-3.5 items-center justify-center rounded-full transition-all duration-150'
			style={{ backgroundColor: color || 'var(--surface-active)' }}>
			<span className='opacity-0 transition-opacity duration-100 group-hover:opacity-100'>
				{children}
			</span>
		</button>
	)
}

function NavBtn({
	onClick,
	disabled,
	children,
}: {
	onClick: () => void
	disabled?: boolean
	children: React.ReactNode
}) {
	return (
		<button
			type='button'
			onClick={(e) => {
				e.stopPropagation()
				onClick()
			}}
			disabled={disabled}
			className='flex h-6 w-6 items-center justify-center rounded-md text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] disabled:pointer-events-none disabled:opacity-25'>
			{children}
		</button>
	)
}

function IconBtn({
	onClick,

	title,
	children,
}: {
	onClick: () => void
	active?: boolean
	title?: string
	children: React.ReactNode
}) {
	return (
		<button
			type='button'
			onClick={(e) => {
				e.stopPropagation()
				onClick()
			}}
			onMouseDown={(e) => e.stopPropagation()}
			title={title}
			className='relative flex h-7 w-7 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
			{children}
		</button>
	)
}

export function TitleBar({ isDashboard, onSearch, onOpenSettings, onOpenOutbox }: TitleBarProps) {
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const isSending = useDraftStore((s) => s.isSending)
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
			setIsMobile(/android|iphone|ipad|ipod|mobile/i.test(navigator.userAgent.toLowerCase()))
		}
	}, [])

	if (isMobile === null || isMobile) return null

	const appWindow = getCurrentWindow()

	return (
		<div
			className='relative z-50 flex h-11 shrink-0 items-center border-b select-none'
			style={{ borderColor: 'var(--border-subtle)' }}
			onMouseDown={() => appWindow.startDragging()}>
			{/* Loading bar */}
			<AnimatePresence>
				{isLoading && animationsEnabled && (
					<motion.div
						initial={{ opacity: 0 }}
						animate={{ opacity: 1 }}
						exit={{ opacity: 0 }}
						className='pointer-events-none absolute inset-x-0 bottom-[-1px] h-px overflow-hidden'>
						<motion.div
							className='h-full w-1/3'
							style={{
								background: `linear-gradient(90deg, transparent, ${accentColor}, transparent)`,
							}}
							animate={{ x: ['-100%', '400%'] }}
							transition={{ duration: 1.4, repeat: Infinity, ease: 'easeInOut' }}
						/>
					</motion.div>
				)}
			</AnimatePresence>

			{/* LEFT - logo + window controls */}
			<div
				className='flex w-56 shrink-0 items-center gap-3 px-4'
				onMouseDown={(e) => e.stopPropagation()}>
				{/* window controls */}
				<div className='flex items-center gap-1.5'>
					<WinBtn onClick={() => appWindow.close()} color='#ff5f57' title='Close'>
						<svg
							className='h-1.5 w-1.5'
							viewBox='0 0 6 6'
							fill='currentColor'
							style={{ color: '#4e0002' }}>
							<path
								d='M1 1l4 4M5 1L1 5'
								stroke='currentColor'
								strokeWidth='1.2'
								fill='none'
							/>
						</svg>
					</WinBtn>
					<WinBtn onClick={() => appWindow.minimize()} color='#febc2e' title='Minimize'>
						<svg
							className='h-1.5 w-1.5'
							viewBox='0 0 6 6'
							fill='none'
							style={{ color: '#4e2900' }}>
							<path d='M1 3h4' stroke='currentColor' strokeWidth='1.2' />
						</svg>
					</WinBtn>
					<WinBtn
						onClick={() => appWindow.toggleMaximize()}
						color='#28c840'
						title='Maximize'>
						<svg
							className='h-1.5 w-1.5'
							viewBox='0 0 6 6'
							fill='none'
							style={{ color: '#003500' }}>
							<path d='M1 5V1h4M5 5H1' stroke='currentColor' strokeWidth='1.2' />
						</svg>
					</WinBtn>
				</div>

				{/* Branding */}
				<div className='flex items-center gap-2'>
					<img src={icon} alt='Postail' className='h-5 w-5 object-contain opacity-90' />
					<span className='text-[13px] font-semibold tracking-tight text-[var(--text-primary)] opacity-80'>
						Postail
					</span>
				</div>
			</div>

			{/* CENTER - search or message subject */}
			<div className='flex flex-1 items-center justify-center px-3'>
				<AnimatePresence mode='wait'>
					{isViewingMessage ? (
						<motion.div
							key='subject'
							className='flex w-full max-w-2xl items-center gap-2'
							initial={animationsEnabled ? { opacity: 0, y: 5 } : {}}
							animate={animationsEnabled ? { opacity: 1, y: 0 } : {}}
							exit={animationsEnabled ? { opacity: 0, y: -5 } : {}}
							transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
							onMouseDown={(e) => e.stopPropagation()}>
							{/* Prev/next */}
							<div className='flex items-center gap-0.5'>
								<NavBtn
									onClick={() => titleMeta?.onPrev?.()}
									disabled={isLoading || !titleMeta?.onPrev}>
									<ChevronLeft className='h-3.5 w-3.5' />
								</NavBtn>
								<NavBtn
									onClick={() => titleMeta?.onNext?.()}
									disabled={isLoading || !titleMeta?.onNext}>
									<ChevronRight className='h-3.5 w-3.5' />
								</NavBtn>
							</div>

							{/* Subject */}
							<div className='min-w-0 flex-1'>
								<AnimatePresence mode='wait'>
									{isLoading ? (
										<motion.div
											key='shimmer'
											initial={animationsEnabled ? { opacity: 0 } : {}}
											animate={animationsEnabled ? { opacity: 1 } : {}}
											exit={animationsEnabled ? { opacity: 0 } : {}}
											className='relative h-4 w-48 overflow-hidden rounded bg-[var(--surface-active)]'>
											<motion.div
												className='absolute inset-0'
												style={{
													background: `linear-gradient(90deg, transparent, ${accentColor}30, transparent)`,
												}}
												animate={{ x: ['-100%', '100%'] }}
												transition={{ duration: 1.4, repeat: Infinity }}
											/>
										</motion.div>
									) : (
										<motion.p
											key={titleMeta?.subject}
											initial={animationsEnabled ? { opacity: 0, y: 3 } : {}}
											animate={animationsEnabled ? { opacity: 1, y: 0 } : {}}
											exit={animationsEnabled ? { opacity: 0, y: -3 } : {}}
											className='truncate text-center text-[13px] font-medium text-[var(--text-primary)]'>
											{titleMeta?.subject || 'No Subject'}
										</motion.p>
									)}
								</AnimatePresence>
							</div>
						</motion.div>
					) : isDashboard && onSearch ? (
						<motion.div
							key='search'
							className='w-full max-w-lg'
							initial={animationsEnabled ? { opacity: 0, y: -5 } : {}}
							animate={animationsEnabled ? { opacity: 1, y: 0 } : {}}
							exit={animationsEnabled ? { opacity: 0, y: 5 } : {}}
							transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
							onMouseDown={(e) => e.stopPropagation()}>
							<SearchBar onSearch={onSearch} />
						</motion.div>
					) : null}
				</AnimatePresence>
			</div>

			{/* RIGHT - actions + account */}
			<div
				className='flex w-56 shrink-0 items-center justify-end gap-1 px-3'
				onMouseDown={(e) => e.stopPropagation()}>
				{isDashboard && activeAccount && (
					<>
						<NotificationCenter />

						<IconBtn onClick={() => onOpenOutbox?.()} title='Outbox'>
							<Send
								className='h-[15px] w-[15px]'
								style={isSending ? { color: accentColor } : undefined}
							/>
							{isSending && animationsEnabled && (
								<motion.span
									className='absolute inset-0 rounded-lg'
									animate={{ scale: [1, 1.5], opacity: [0.25, 0] }}
									transition={{ duration: 0.9, repeat: Infinity }}
									style={{ backgroundColor: accentColor }}
								/>
							)}
						</IconBtn>

						<IconBtn onClick={() => onOpenSettings?.()} title='Settings'>
							<Settings className='h-[15px] w-[15px]' />
						</IconBtn>

						<div className='ml-1'>
							<AccountSwitcher onOpenSettings={onOpenSettings!} />
						</div>
					</>
				)}
			</div>
		</div>
	)
}
