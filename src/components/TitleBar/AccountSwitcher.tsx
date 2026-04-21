import { useState, useRef, useEffect } from 'react'
import { createPortal } from 'react-dom'
import { motion, AnimatePresence } from 'framer-motion'
import { Check, Settings } from 'lucide-react'
import { useAccountStore } from '@/stores/accountStore'
import { useThemeStore } from '@/stores/themeStore'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import type { AccountSwitcherProps } from '@/types/components/shared'

export function AccountSwitcher({ onOpenSettings }: AccountSwitcherProps) {
	const { accounts, activeAccount, setActiveAccount } = useAccountStore()
	const { accentColor } = useThemeStore()
	const { t } = useTypedTranslation()
	const [isOpen, setIsOpen] = useState(false)
	const [menuPos, setMenuPos] = useState<{ top: number; right: number } | null>(null)
	const triggerRef = useRef<HTMLButtonElement>(null)
	const menuRef = useRef<HTMLDivElement>(null)

	// Position menu
	useEffect(() => {
		if (!isOpen) {
			setMenuPos(null)
			return
		}
		const r = triggerRef.current?.getBoundingClientRect()
		if (!r) return
		setMenuPos({ top: r.bottom + 6, right: window.innerWidth - r.right })
	}, [isOpen])

	// Close on outside click
	useEffect(() => {
		if (!isOpen) return
		const handler = (e: MouseEvent) => {
			if (menuRef.current?.contains(e.target as Node)) return
			if (triggerRef.current?.contains(e.target as Node)) return
			setIsOpen(false)
		}
		document.addEventListener('mousedown', handler)
		return () => document.removeEventListener('mousedown', handler)
	}, [isOpen])

	if (!activeAccount) return null

	const initial = activeAccount.name.charAt(0).toUpperCase()

	return (
		<>
			<button
				ref={triggerRef}
				type='button'
				onClick={(e) => {
					e.stopPropagation()
					setIsOpen((v) => !v)
				}}
				onMouseDown={(e) => e.stopPropagation()}
				className='group relative flex h-7 w-7 items-center justify-center rounded-full transition-transform hover:scale-105'
				style={{
					background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
				}}>
				<span className='text-[11px] font-bold' style={{ color: 'var(--accent-text)' }}>
					{initial}
				</span>
				{/* Active ring */}
				<span
					className='pointer-events-none absolute inset-0 rounded-full ring-2 ring-offset-1 transition-opacity'
					style={{
						boxShadow: `0 0 0 2px var(--app-bg), 0 0 0 3.5px ${accentColor}80`,
						opacity: isOpen ? 1 : 0,
					}}
				/>
			</button>

			{/* Portal dropdown */}
			<AnimatePresence>
				{isOpen &&
					menuPos &&
					createPortal(
						<motion.div
							ref={menuRef}
							initial={{ opacity: 0, scale: 0.96, y: -6 }}
							animate={{ opacity: 1, scale: 1, y: 0 }}
							exit={{ opacity: 0, scale: 0.96, y: -4 }}
							transition={{ duration: 0.15, ease: [0.16, 1, 0.3, 1] }}
							className='fixed z-[300] w-52 overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-glass)] shadow-2xl backdrop-blur-2xl'
							style={{
								top: menuPos.top,
								right: menuPos.right,
								transformOrigin: 'top right',
							}}
							onMouseDown={(e) => e.stopPropagation()}>
							{/* Active account info */}
							<div className='px-3 py-2.5'>
								<p className='text-[13px] leading-tight font-semibold text-[var(--text-primary)]'>
									{activeAccount.name}
								</p>
								<p className='mt-0.5 truncate text-[11px] leading-tight text-[var(--text-tertiary)]'>
									{activeAccount.email}
								</p>
							</div>

							<div className='mx-2 h-px bg-[var(--border-faint)]' />

							{/* Account list */}
							<div className='px-1 py-1'>
								{accounts.map((account) => {
									const isActive = account.id === activeAccount.id
									return (
										<button
											key={account.id}
											type='button'
											onClick={() => {
												setActiveAccount(account)
												setIsOpen(false)
											}}
											className='flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left transition-colors hover:bg-[var(--surface-hover)]'>
											<div
												className='flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[10px] font-bold'
												style={
													isActive
														? {
																background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
																color: 'var(--accent-text)',
															}
														: {
																backgroundColor:
																	'var(--surface-active)',
																color: 'var(--text-secondary)',
															}
												}>
												{account.name.charAt(0).toUpperCase()}
											</div>
											<span className='flex-1 truncate text-[12px] font-medium text-[var(--text-primary)]'>
												{account.name}
											</span>
											{isActive && (
												<Check
													className='h-3.5 w-3.5 shrink-0'
													style={{ color: accentColor }}
												/>
											)}
										</button>
									)
								})}
							</div>

							<div className='mx-2 h-px bg-[var(--border-faint)]' />

							{/* Settings */}
							<div className='px-1 py-1'>
								<button
									type='button'
									onClick={() => {
										onOpenSettings()
										setIsOpen(false)
									}}
									className='flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left text-[12px] text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
									<Settings className='h-3.5 w-3.5 shrink-0 text-[var(--text-tertiary)]' />
									{t('settings:accounts.title', {
										defaultValue: 'Manage Accounts',
									})}
								</button>
							</div>
						</motion.div>,
						document.body
					)}
			</AnimatePresence>
		</>
	)
}
