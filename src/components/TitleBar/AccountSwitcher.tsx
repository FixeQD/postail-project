import { useState, useRef, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Check, Settings, ChevronDown } from 'lucide-react'
import { useAccountStore } from '@/stores/accountStore'
import { useThemeStore } from '@/stores/themeStore'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'

interface AccountSwitcherProps {
	onOpenSettings: () => void
}

export function AccountSwitcher({ onOpenSettings }: AccountSwitcherProps) {
	const { accounts, activeAccount, setActiveAccount } = useAccountStore()
	const { accentColor } = useThemeStore()
	const animationsEnabled = useAnimationsEnabled()
	const { t } = useTypedTranslation()
	const [isOpen, setIsOpen] = useState(false)

	const containerRef = useRef<HTMLDivElement>(null)

	useEffect(() => {
		const handleClickOutside = (event: MouseEvent) => {
			if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
				setIsOpen(false)
			}
		}

		if (isOpen) {
			document.addEventListener('mousedown', handleClickOutside)
		}
		return () => {
			document.removeEventListener('mousedown', handleClickOutside)
		}
	}, [isOpen])

	if (!activeAccount) return null

	return (
		<div ref={containerRef} className="relative">
			<button
				onClick={(e) => {
					e.stopPropagation()
					setIsOpen(!isOpen)
				}}
				className="group flex items-center gap-2 rounded-full ring-offset-slate-900 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/20"
			>
				<div
					className="flex h-8 w-8 items-center justify-center overflow-hidden rounded-full ring-2 ring-slate-950 ring-offset-1 ring-offset-slate-800/50 transition-transform group-hover:scale-105"
					style={{
						background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
					}}
				>
					<span className="text-accent-contrast text-sm font-bold">
						{activeAccount.name.charAt(0).toUpperCase()}
					</span>
				</div>
				<ChevronDown
					className={`h-4 w-4 text-slate-400 text-muted-foreground transition-transform duration-200 ${isOpen ? 'rotate-180' : ''}`}
				/>
			</button>

			{/* Dropdown */}
			<AnimatePresence>
				{isOpen && (
					<>

						<motion.div
							{...(animationsEnabled
								? {
										initial: { opacity: 0, scale: 0.95, y: -8 },
										animate: { opacity: 1, scale: 1, y: 0 },
										exit: { opacity: 0, scale: 0.95, y: -4 },
										transition: { duration: 0.15, ease: 'easeOut' },
									}
								: {})}
							className="absolute right-0 top-[calc(100%+8px)] z-50 w-56 overflow-hidden rounded-xl border border-white/[0.06] bg-slate-900/95 p-1 text-slate-200 shadow-xl backdrop-blur-xl"
							style={{ transformOrigin: 'top right' }}
						>
							<div className="px-2 py-1.5">
								<div className="flex flex-col space-y-1">
									<p className="text-sm font-medium leading-none">{activeAccount.name}</p>
									<p className="text-xs leading-none text-slate-400">{activeAccount.email}</p>
								</div>
							</div>
							<div className="my-1 h-px bg-white/[0.06]" />
							<div className="max-h-[300px] overflow-y-auto hover-scrollbar">
								{accounts.map((account) => {
									const isActive = account.id === activeAccount.id

									return (
										<button
											key={account.id}
											onClick={() => {
												setActiveAccount(account)
												setIsOpen(false)
											}}
											className={`flex w-full cursor-pointer items-center justify-between rounded-md px-2 py-2 text-sm outline-none transition-colors hover:bg-white/[0.06] hover:text-slate-100 focus:bg-white/[0.06] focus:text-slate-100 ${
												isActive ? 'bg-white/[0.04]' : ''
											}`}
										>
											<div className="flex items-center gap-2 truncate">
												<div
													className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[10px] font-bold"
													style={
														isActive
															? {
																	background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
																	color: 'var(--accent-contrast)',
																}
															: {
																	backgroundColor: 'rgba(255,255,255,0.1)',
																	color: 'inherit',
																}
													}
												>
													{account.name.charAt(0).toUpperCase()}
												</div>
												<span className="truncate">{account.name}</span>
											</div>
											{isActive && (
												<Check
													className="h-4 w-4 shrink-0"
													style={{ color: accentColor }}
												/>
											)}
										</button>
									)
								})}
							</div>
							<div className="my-1 h-px bg-white/[0.06]" />
							<button
								onClick={() => {
									onOpenSettings()
									setIsOpen(false)
								}}
								className="flex w-full cursor-pointer items-center rounded-md px-2 py-2 text-sm text-slate-300 outline-none transition-colors hover:bg-white/[0.06] hover:text-slate-100 focus:bg-white/[0.06] focus:text-slate-100"
							>
								<Settings className="mr-2 h-4 w-4" />
								<span>{t('settings:accounts.title', { defaultValue: 'Manage Accounts' })}</span>
							</button>
						</motion.div>
					</>
				)}
			</AnimatePresence>
		</div>
	)
}
