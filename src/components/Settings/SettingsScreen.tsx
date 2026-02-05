import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Settings, User, Shield, Palette, Bell, ArrowLeft, LogOut } from 'lucide-react'
import { AccountsScreen } from '../Account/AccountsScreen'
import type { AccountMeta } from '@/types/accounts'

interface SettingsScreenProps {
	accounts: AccountMeta[]
	onRemoveAccount: (id: string) => void
	onSyncAccount: (id: string) => void
	onBack: () => void
}

const SETTINGS_SECTIONS = [
	{ id: 'accounts', label: 'Accounts', icon: User },
	{ id: 'general', label: 'General', icon: Settings, disabled: true },
	{ id: 'security', label: 'Security', icon: Shield, disabled: true },
	{ id: 'appearance', label: 'Appearance', icon: Palette, disabled: true },
	{ id: 'notifications', label: 'Notifications', icon: Bell, disabled: true },
]

export function SettingsScreen({ accounts, onRemoveAccount, onSyncAccount, onBack }: SettingsScreenProps) {
	const [activeSection, setActiveSection] = useState('accounts')

	return (
		<div className='flex h-full bg-slate-950 text-slate-100'>
			{/* Sidebar */}
			<div className='w-64 border-r border-slate-800 bg-slate-900/30 backdrop-blur-xl flex flex-col p-4'>
				<button
					type='button'
					onClick={onBack}
					className='flex items-center gap-2 px-4 py-2 text-slate-400 hover:text-white transition-colors mb-8 group'>
					<ArrowLeft className='h-4 w-4 transition-transform group-hover:-translate-x-1' />
					<span className='font-medium'>Back</span>
				</button>

				<div className='flex-1 space-y-1'>
					{SETTINGS_SECTIONS.map((section) => (
						<button
							key={section.id}
							type='button'
							disabled={section.disabled}
							onClick={() => setActiveSection(section.id)}
							className={`w-full flex items-center gap-3 px-4 py-2.5 rounded-xl transition-all ${
								activeSection === section.id
									? 'bg-slate-100 text-slate-900 shadow-lg shadow-white/5'
									: section.disabled
									? 'opacity-40 cursor-not-allowed grayscale'
									: 'text-slate-400 hover:bg-slate-800/50 hover:text-slate-100'
							}`}>
							<section.icon className='h-4 w-4' />
							<span className='text-sm font-semibold'>{section.label}</span>
						</button>
					))}
				</div>

				<div className='border-t border-slate-800 pt-4'>
					<button
						type='button'
						className='flex w-full items-center gap-3 rounded-xl px-4 py-2.5 text-red-400 transition-all hover:bg-red-400/10'>
						<LogOut className='h-4 w-4' />
						<span className='text-sm font-semibold'>Log out</span>
					</button>
				</div>
			</div>

			{/* Content area */}
			<div className='relative flex-1 overflow-hidden'>
				<AnimatePresence mode='wait'>
					{activeSection === 'accounts' ? (
						<motion.div
							key='accounts'
							initial={{ opacity: 0, y: 10 }}
							animate={{ opacity: 1, y: 0 }}
							exit={{ opacity: 0, y: -10 }}
							className='h-full'>
							<AccountsScreen
								accounts={accounts}
								onRemoveAccount={onRemoveAccount}
								onSyncAccount={onSyncAccount}
							/>
						</motion.div>
					) : (
						<motion.div
							key='empty'
							initial={{ opacity: 0 }}
							animate={{ opacity: 1 }}
							className='flex h-full items-center justify-center text-slate-500'>
							Coming soon...
						</motion.div>
					)}
				</AnimatePresence>
			</div>
		</div>
	)
}
