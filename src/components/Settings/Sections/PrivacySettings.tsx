import { motion } from 'framer-motion'
import { ShieldAlert, MailX, FileX2, Link2Off } from 'lucide-react'
import { useSettingsStore } from '@/stores/settingsStore'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { ToggleSetting } from '@/components/ui/toggle-setting'

export function PrivacySettings() {
	const { t } = useSettingsTranslation()
	const { settings, setSetting } = useSettingsStore()

	return (
		<div className='mx-auto flex h-full w-full max-w-3xl flex-col space-y-8 p-8'>
			<motion.div initial={{ opacity: 0, y: -20 }} animate={{ opacity: 1, y: 0 }}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
					{t('settings:privacy.title')}
				</h1>
				<p className='mt-1 text-slate-400'>{t('settings:privacy.subtitle')}</p>
			</motion.div>

			<div className='space-y-6'>
				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:privacy.protection.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={ShieldAlert}
							label={t('settings:privacy.protection.shieldImages.label')}
							description={t('settings:privacy.protection.shieldImages.description')}
							value={settings['block-external-images']}
							onChange={(val: boolean) => setSetting('block-external-images', val)}
						/>
						<ToggleSetting
							icon={MailX}
							label={t('settings:privacy.protection.blockReadReceipts.label')}
							description={t(
								'settings:privacy.protection.blockReadReceipts.description'
							)}
							value={false}
							onChange={() => {}}
						/>
						<ToggleSetting
							icon={Link2Off}
							label={t('settings:privacy.protection.disableLinkPreview.label')}
							description={t(
								'settings:privacy.protection.disableLinkPreview.description'
							)}
							value={false}
							onChange={() => {}}
						/>
					</div>
				</section>

				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:privacy.metadata.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={FileX2}
							label={t('settings:privacy.metadata.stripMetadata.label')}
							description={t('settings:privacy.metadata.stripMetadata.description')}
							value={false}
							onChange={() => {}}
						/>
					</div>
				</section>
			</div>
		</div>
	)
}
