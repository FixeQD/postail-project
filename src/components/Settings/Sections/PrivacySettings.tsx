import { motion } from 'framer-motion'
import { ShieldAlert, MailX, FileX2, Link2Off } from 'lucide-react'
import { useSettingsStore } from '@/stores/settingsStore'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { ToggleSetting } from '@/components/ui/toggle-setting'

function SectionTitle({ children }: { children: React.ReactNode }) {
	return (
		<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
			{children}
		</h2>
	)
}

export function PrivacySettings() {
	const { t } = useSettingsTranslation()
	const { settings, setSetting } = useSettingsStore()
	const animationsEnabled = useAnimationsEnabled()

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
					{t('settings:privacy.title')}
				</h1>
				<p className='mt-1 text-[var(--text-secondary)]'>
					{t('settings:privacy.subtitle')}
				</p>
			</motion.div>

			<div className='space-y-8'>
				<motion.section {...fade(0.05)}>
					<SectionTitle>{t('settings:privacy.protection.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={ShieldAlert}
							label={t('settings:privacy.protection.shieldImages.label')}
							description={t('settings:privacy.protection.shieldImages.description')}
							value={settings['block-external-images']}
							onChange={(val) => setSetting('block-external-images', val)}
						/>
						<ToggleSetting
							icon={MailX}
							label={t('settings:privacy.protection.blockReadReceipts.label')}
							description={t(
								'settings:privacy.protection.blockReadReceipts.description'
							)}
							value={settings['block-read-receipts']}
							onChange={(val) => setSetting('block-read-receipts', val)}
						/>
						<ToggleSetting
							icon={Link2Off}
							label={t('settings:privacy.protection.disableLinkPreview.label')}
							description={t(
								'settings:privacy.protection.disableLinkPreview.description'
							)}
							value={settings['disable-link-preview']}
							onChange={(val) => setSetting('disable-link-preview', val)}
						/>
					</div>
				</motion.section>

				<motion.section {...fade(0.1)}>
					<SectionTitle>{t('settings:privacy.metadata.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={FileX2}
							label={t('settings:privacy.metadata.stripMetadata.label')}
							description={t('settings:privacy.metadata.stripMetadata.description')}
							value={settings['strip-attachment-metadata']}
							onChange={(val) => setSetting('strip-attachment-metadata', val)}
						/>
					</div>
				</motion.section>
			</div>
		</div>
	)
}
