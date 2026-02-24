import { useState, useCallback, useMemo, memo } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import {
	Settings,
	User,
	Shield,
	Palette,
	Bell,
	ArrowLeft,
	LogOut,
	Lock,
	PenLine,
	Info,
} from 'lucide-react'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { AccountsScreen } from './Sections/Account/AccountsScreen'
import { GeneralSettings } from './Sections/GeneralSettings'
import { PrivacySettings } from './Sections/PrivacySettings'
import { SecuritySettings } from './Sections/SecuritySettings'
import { AppearanceSettings } from './Sections/AppearanceSettings'
import { NotificationsSettings } from './Sections/NotificationsSettings'
import { ComposingSettings } from './Sections/ComposingSettings'
import { AboutSettings } from './Sections/AboutSettings'
import { invoke } from '@tauri-apps/api/core'
import { useAccountStore } from '@/stores/accountStore'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'

interface SettingsScreenProps {
	onBack: () => void
	canGoBack?: boolean
	showSidebar?: boolean
}

interface SettingsNavItemProps {
	section: { id: string; label: string; icon: React.ElementType }
	isActive: boolean
	accentColor: string
	animationsEnabled: boolean
	onClick: () => void
}

const SettingsNavItem = memo(
	({ section, isActive, accentColor, animationsEnabled, onClick }: SettingsNavItemProps) => {
		return (
			<motion.button
				type='button'
				onClick={onClick}
				{...(animationsEnabled ? { whileTap: { scale: 0.97 } } : {})}
				className={`relative flex w-full items-center gap-3 rounded-xl px-4 py-2.5 transition-colors duration-200 ${
					isActive
						? 'text-slate-100'
						: 'text-slate-400 hover:bg-white/[0.04] hover:text-slate-200'
				}`}>
				{/* Active background pill */}
				{isActive && (
					<motion.div
						{...(animationsEnabled
							? {
									layoutId: 'settings-active-bg',
									transition: {
										type: 'spring',
										stiffness: 350,
										damping: 30,
									},
								}
							: {})}
						className='absolute inset-0 rounded-xl bg-white/[0.08] ring-1 ring-white/[0.08]'
					/>
				)}

				{/* Active left accent */}
				{isActive && (
					<motion.div
						{...(animationsEnabled
							? {
									initial: { scaleY: 0, opacity: 0 },
									animate: { scaleY: 1, opacity: 1 },
									exit: { scaleY: 0, opacity: 0 },
									transition: {
										type: 'spring',
										stiffness: 400,
										damping: 25,
									},
								}
							: {})}
						className='absolute top-1/2 left-0 h-5 w-[3px] origin-center -translate-y-1/2 rounded-r-full'
						style={{ backgroundColor: accentColor }}
					/>
				)}

				<section.icon
					className='relative h-4 w-4 transition-colors duration-200'
					style={isActive ? { color: accentColor } : undefined}
				/>
				<span className='relative text-sm font-semibold'>{section.label}</span>
			</motion.button>
		)
	}
)

export function SettingsScreen({
	onBack,
	canGoBack = true,
	showSidebar = true,
}: SettingsScreenProps) {
	const { accounts, removeAccount: onRemoveAccount } = useAccountStore()

	const onSyncAccount = useCallback(async (id: string) => {
		try {
			await invoke('start_sync', { accountId: id })
		} catch (error) {
			console.error('Failed to sync account:', error)
		}
	}, [])

	const { t } = useSettingsTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const [activeSection, setActiveSection] = useState('accounts')

	const SETTINGS_SECTIONS = useMemo(
		() => [
			{ id: 'accounts', label: t('settings:sections.accounts'), icon: User },
			{ id: 'general', label: t('settings:sections.general'), icon: Settings },
			{ id: 'privacy', label: t('settings:sections.privacy'), icon: Shield },
			{ id: 'security', label: t('settings:sections.security'), icon: Lock },
			{ id: 'appearance', label: t('settings:sections.appearance'), icon: Palette },
			{ id: 'notifications', label: t('settings:sections.notifications'), icon: Bell },
			{ id: 'composing', label: t('settings:sections.composing'), icon: PenLine },
			{ id: 'about', label: 'About', icon: Info },
		],
		[t]
	)

	const activeSectionComponent = useMemo(() => {
		switch (activeSection) {
			case 'accounts':
				return (
					<AccountsScreen
						accounts={accounts}
						onRemoveAccount={onRemoveAccount}
						onSyncAccount={onSyncAccount}
					/>
				)
			case 'general':
				return <GeneralSettings />
			case 'privacy':
				return <PrivacySettings />
			case 'security':
				return <SecuritySettings />
			case 'appearance':
				return <AppearanceSettings />
			case 'notifications':
				return <NotificationsSettings />
			case 'composing':
				return <ComposingSettings />
			case 'about':
				return <AboutSettings />
			default:
				return (
					<div className='flex h-full items-center justify-center text-slate-500'>
						{t('settings:comingSoon')}
					</div>
				)
		}
	}, [activeSection, accounts, onRemoveAccount, onSyncAccount, t])

	return (
		<div className='flex h-full text-slate-100'>
			{showSidebar && (
				<motion.div
					{...(animationsEnabled
						? {
								initial: { opacity: 0, x: -12 },
								animate: { opacity: 1, x: 0 },
								transition: { duration: 0.35, ease: [0.16, 1, 0.3, 1] },
							}
						: {})}
					className='relative flex w-64 flex-col border-r border-white/[0.06] bg-slate-900/20 p-4 backdrop-blur-xl'>
					{/* Right edge gradient */}
					<div className='pointer-events-none absolute top-0 right-0 bottom-0 w-px bg-gradient-to-b from-transparent via-white/[0.06] to-transparent' />

					{canGoBack && (
						<button
							type='button'
							onClick={onBack}
							className='group mb-8 flex items-center gap-2 rounded-xl px-4 py-2 text-slate-400 transition-colors hover:bg-white/[0.04] hover:text-white'>
							<ArrowLeft className='h-4 w-4 transition-transform group-hover:-translate-x-0.5' />
							<span className='text-sm font-medium'>{t('settings:back')}</span>
						</button>
					)}

					<div className='flex-1 space-y-0.5'>
						{SETTINGS_SECTIONS.map((section) => (
							<SettingsNavItem
								key={section.id}
								section={section}
								isActive={activeSection === section.id}
								accentColor={accentColor}
								animationsEnabled={animationsEnabled}
								onClick={() => setActiveSection(section.id)}
							/>
						))}
					</div>

					<div className='border-t border-white/[0.06] pt-4'>
						<motion.button
							type='button'
							{...(animationsEnabled ? { whileTap: { scale: 0.97 } } : {})}
							className='flex w-full items-center gap-3 rounded-xl px-4 py-2.5 text-red-400 transition-all hover:bg-red-500/10'>
							<LogOut className='h-4 w-4' />
							<span className='text-sm font-semibold'>{t('settings:logout')}</span>
						</motion.button>
					</div>
				</motion.div>
			)}

			<div className='relative flex-1 overflow-hidden'>
				{animationsEnabled ? (
					<AnimatePresence mode='wait'>
						<motion.div
							key={activeSection}
							initial={{ opacity: 0, y: 8 }}
							animate={{ opacity: 1, y: 0 }}
							exit={{ opacity: 0, y: -6 }}
							transition={{ duration: 0.25, ease: [0.16, 1, 0.3, 1] }}
							className='h-full overflow-y-auto'>
							{activeSectionComponent}
						</motion.div>
					</AnimatePresence>
				) : (
					<div className='h-full overflow-y-auto'>{activeSectionComponent}</div>
				)}
			</div>
		</div>
	)
}
