import { useState, useRef, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Check, Settings, ChevronDown } from 'lucide-react'
import { useAccountStore } from '@/stores/accountStore'
import { useThemeStore } from '@/stores/themeStore'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import type { AccountSwitcherProps } from '@/types/components/shared'

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
		<div ref={containerRef} className='relative'>
			<button
				onClick={(e) => {
					e.stopPropagation()
					setIsOpen(!isOpen)
				}}
				onMouseDown={(e) => e.stopPropagation()}
				className='group flex items-center gap-2 rounded-full transition-all focus-visible:ring-2 focus-visible:ring-[var(--border-subtle)] focus-visible:outline-none'>
				<div
					className='flex h-8 w-8 items-center justify-center overflow-hidden rounded-full ring-2 ring-[var(--app-bg)] ring-offset-1 ring-offset-[var(--surface-active)] transition-transform group-hover:scale-105'
					style={{
						background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
					}}>
					<span className='text-accent-contrast text-sm font-bold'>
						{activeAccount.name.charAt(0).toUpperCase()}
					</span>
				</div>
				<ChevronDown
					className={`text-muted-foreground h-4 w-4 transition-transform duration-200 ${isOpen ? 'rotate-180' : ''}`}
				/>
			</button>

			{/* Dropdown */}
			<AnimatePresence>
				{isOpen && (
					<>
						<motion.div
							{...(animationsEnabled
								? {
										initial: {
											opacity: 0,
											scale: 0.96,
											y: -8,
											filter: 'blur(4px)',
										},
										animate: {
											opacity: 1,
											scale: 1,
											y: 0,
											filter: 'blur(0px)',
										},
										exit: {
											opacity: 0,
											scale: 0.96,
											y: -4,
											filter: 'blur(4px)',
										},
										transition: { duration: 0.2, ease: [0.16, 1, 0.3, 1] },
									}
								: {})}
							onMouseDown={(e) => e.stopPropagation()}
							className='text-foreground absolute top-[calc(100%+8px)] right-0 z-50 w-56 overflow-hidden rounded-xl border bg-[var(--surface-glass)] p-1 shadow-xl backdrop-blur-xl'
							style={{
								borderColor: 'var(--border-subtle)',
								transformOrigin: 'top right',
							}}>
							<div className='px-2 py-1.5'>
								<div className='flex flex-col space-y-1'>
									<p className='text-foreground text-sm leading-none font-medium'>
										{activeAccount.name}
									</p>
									<p className='text-muted-foreground text-xs leading-none'>
										{activeAccount.email}
									</p>
								</div>
							</div>
							<div className='my-1 h-px bg-[var(--border-subtle)]' />
							<div className='hover-scrollbar max-h-[300px] overflow-y-auto'>
								{accounts.map((account) => {
									const isActive = account.id === activeAccount.id

									return (
										<button
											key={account.id}
											onClick={() => {
												setActiveAccount(account)
												setIsOpen(false)
											}}
											className={`text-foreground flex w-full cursor-pointer items-center justify-between rounded-md px-2 py-2 text-sm transition-all outline-none hover:bg-[var(--surface-hover)] focus:bg-[var(--surface-hover)]`}
											style={
												isActive
													? {
															backgroundColor: `${accentColor}1A`,
														}
													: {}
											}>
											<div className='flex items-center gap-2 truncate'>
												<div
													className='flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[10px] font-bold'
													style={
														isActive
															? {
																	background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
																	color: 'var(--accent-contrast)',
																}
															: {
																	backgroundColor:
																		'var(--surface-active)',
																	color: 'inherit',
																}
													}>
													{account.name.charAt(0).toUpperCase()}
												</div>
												<span className='truncate'>{account.name}</span>
											</div>
											{isActive && (
												<Check
													className='h-4 w-4 shrink-0'
													style={{ color: accentColor }}
												/>
											)}
										</button>
									)
								})}
							</div>
							<div className='my-1 h-px bg-[var(--border-subtle)]' />
							<button
								onClick={() => {
									onOpenSettings()
									setIsOpen(false)
								}}
								className='text-foreground/80 hover:text-foreground flex w-full cursor-pointer items-center rounded-md px-2 py-2 text-sm transition-colors outline-none hover:bg-[var(--surface-hover)] focus:bg-[var(--surface-hover)]'>
								<Settings className='mr-2 h-4 w-4' />
								<span>
									{t('settings:accounts.title', {
										defaultValue: 'Manage Accounts',
									})}
								</span>
							</button>
						</motion.div>
					</>
				)}
			</AnimatePresence>
		</div>
	)
}
