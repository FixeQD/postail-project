import { motion } from 'framer-motion'
import { Timer, FileKey, ClipboardX } from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'

export function SecuritySettings() {
	const { t } = useSettingsTranslation()

	return (
		<div className='mx-auto flex h-full w-full max-w-3xl flex-col space-y-8 p-8'>
			<motion.div initial={{ opacity: 0, y: -20 }} animate={{ opacity: 1, y: 0 }}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
					{t('settings:security.title')}
				</h1>
				<p className='mt-1 text-slate-400'>{t('settings:security.subtitle')}</p>
			</motion.div>

			<div className='space-y-6'>
				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:security.session.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Timer}
							label={t('settings:security.session.autoLock.label')}
							description={t('settings:security.session.autoLock.description')}
							value={false}
							onChange={() => {}}
						/>
					</div>
				</section>

				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:security.data.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={FileKey}
							label={t('settings:security.data.encryptAttachments.label')}
							description={t('settings:security.data.encryptAttachments.description')}
							value={false}
							onChange={() => {}}
						/>
						<ToggleSetting
							icon={ClipboardX}
							label={t('settings:security.data.clearClipboard.label')}
							description={t('settings:security.data.clearClipboard.description')}
							value={false}
							onChange={() => {}}
						/>
					</div>
				</section>
			</div>
		</div>
	)
}
