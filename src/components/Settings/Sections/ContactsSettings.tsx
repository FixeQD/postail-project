import { motion } from 'framer-motion'
import { UserPlus } from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useSettingsStore } from '@/stores/settingsStore'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'

function SectionTitle({ children }: { children: React.ReactNode }) {
	return (
		<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
			{children}
		</h2>
	)
}

export function ContactsSettings() {
	const { t } = useSettingsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const { settings, setSetting } = useSettingsStore()

	const fade = (delay = 0) =>
		animationsEnabled
			? {
					initial: { opacity: 0, y: 14 },
					animate: { opacity: 1, y: 0 },
					transition: { delay, duration: 0.35 },
				}
			: {}

	return (
		<div className='mx-auto flex w-full max-w-3xl flex-col space-y-8 p-8 pb-16'>
			<motion.div
				{...(animationsEnabled
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-[var(--text-primary)]'>
					{t('settings:contacts.title')}
				</h1>
				<p className='mt-1 text-[var(--text-secondary)]'>
					{t('settings:contacts.subtitle')}
				</p>
			</motion.div>

			<div className='space-y-8'>
				<motion.section {...fade(0.05)}>
					<SectionTitle>{t('settings:contacts.general.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={UserPlus}
							label={t('settings:contacts.general.autoCreate.label')}
							description={t('settings:contacts.general.autoCreate.description')}
							value={settings['auto-create-contacts']}
							onChange={(val) => setSetting('auto-create-contacts', val)}
						/>
					</div>
				</motion.section>
			</div>
		</div>
	)
}
