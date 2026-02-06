import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Settings, User, Shield, Palette, Bell, ArrowLeft, LogOut } from 'lucide-react'
import { AccountsScreen } from './Sections/Account/AccountsScreen'
import { GeneralSettings } from './Sections/GeneralSettings'
import { PrivacySettings } from './Sections/PrivacySettings'
import type { AccountMeta } from '@/types/accounts'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'

interface SettingsScreenProps {
	accounts: AccountMeta[]
	onRemoveAccount: (id: string) => void
	onSyncAccount: (id: string) => void
	onBack: () => void
	canGoBack?: boolean
	showSidebar?: boolean
}

export function SettingsScreen({
	accounts,
	onRemoveAccount,
	onSyncAccount,
	onBack,
	canGoBack = true,
	showSidebar = true,
}: SettingsScreenProps) {
	const { t } = useSettingsTranslation()
	const [activeSection, setActiveSection] = useState('accounts')

	const SETTINGS_SECTIONS = [
		{ id: 'accounts', label: t('settings:sections.accounts'), icon: User },
		{ id: 'general', label: t('settings:sections.general'), icon: Settings },
		{ id: 'privacy', label: t('settings:sections.privacy'), icon: Shield },
		{ id: 'security', label: t('settings:sections.security'), icon: Shield, disabled: true },
		{
			id: 'appearance',
			label: t('settings:sections.appearance'),
			icon: Palette,
			disabled: true,
		},
		{
			id: 'notifications',
			label: t('settings:sections.notifications'),
			icon: Bell,
			disabled: true,
		},
	]

	return (
		<div className='flex h-full bg-slate-950 text-slate-100'>
			{/* Sidebar */}
			{showSidebar && (
				<div className='flex w-64 flex-col border-r border-slate-800 bg-slate-900/30 p-4 backdrop-blur-xl'>
					{canGoBack && (
						<button
							type='button'
							onClick={onBack}
							className='group mb-8 flex items-center gap-2 px-4 py-2 text-slate-400 transition-colors hover:text-white'>
							<ArrowLeft className='h-4 w-4 transition-transform group-hover:-translate-x-1' />
							<span className='font-medium'>{t('settings:back')}</span>
						</button>
					)}

					<div className='flex-1 space-y-1'>
						{SETTINGS_SECTIONS.map((section) => (
							<button
								key={section.id}
								type='button'
								disabled={section.disabled}
								onClick={() => setActiveSection(section.id)}
								className={`flex w-full items-center gap-3 rounded-xl px-4 py-2.5 transition-all ${
									activeSection === section.id
										? 'bg-slate-100 text-slate-900 shadow-lg shadow-white/5'
										: section.disabled
											? 'cursor-not-allowed opacity-40 grayscale'
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
							<span className='text-sm font-semibold'>{t('settings:logout')}</span>
						</button>
					</div>
				</div>
			)}

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
					) : activeSection === 'general' ? (
						<motion.div
							key='general'
							initial={{ opacity: 0, y: 10 }}
							animate={{ opacity: 1, y: 0 }}
							exit={{ opacity: 0, y: -10 }}
							className='h-full overflow-y-auto'>
							<GeneralSettings />
						</motion.div>
					) : activeSection === 'privacy' ? (
						<motion.div
							key='privacy'
							initial={{ opacity: 0, y: 10 }}
							animate={{ opacity: 1, y: 0 }}
							exit={{ opacity: 0, y: -10 }}
							className='h-full overflow-y-auto'>
							<PrivacySettings />
						</motion.div>
					) : (
						<motion.div
							key='empty'
							initial={{ opacity: 0 }}
							animate={{ opacity: 1 }}
							className='flex h-full items-center justify-center text-slate-500'>
							{t('settings:comingSoon')}
						</motion.div>
					)}
				</AnimatePresence>
			</div>
		</div>
	)
}
