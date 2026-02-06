import { motion } from 'framer-motion'
import { Moon, Minimize2, UserCircle, Sparkles } from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'

export function AppearanceSettings() {
	const { t } = useSettingsTranslation()

	return (
		<div className='flex h-full flex-col p-8 max-w-3xl mx-auto w-full space-y-8'>
			<motion.div
				initial={{ opacity: 0, y: -20 }}
				animate={{ opacity: 1, y: 0 }}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
					{t('settings:appearance.title')}
				</h1>
				<p className='mt-1 text-slate-400'>{t('settings:appearance.subtitle')}</p>
			</motion.div>

			<div className='space-y-6'>
				<section>
					<h2 className='text-xs font-bold uppercase tracking-widest text-slate-500 mb-4 ml-2'>
						{t('settings:appearance.theme.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Moon}
							label={t('settings:appearance.theme.darkMode.label')}
							description={t('settings:appearance.theme.darkMode.description')}
							value={true}
							onChange={() => {}}
						/>
					</div>
				</section>

				<section>
					<h2 className='text-xs font-bold uppercase tracking-widest text-slate-500 mb-4 ml-2'>
						{t('settings:appearance.layout.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Minimize2}
							label={t('settings:appearance.layout.compactMode.label')}
							description={t('settings:appearance.layout.compactMode.description')}
							value={false}
							onChange={() => {}}
						/>
						<ToggleSetting
							icon={UserCircle}
							label={t('settings:appearance.layout.showAvatars.label')}
							description={t('settings:appearance.layout.showAvatars.description')}
							value={true}
							onChange={() => {}}
						/>
						<ToggleSetting
							icon={Sparkles}
							label={t('settings:appearance.layout.animations.label')}
							description={t('settings:appearance.layout.animations.description')}
							value={true}
							onChange={() => {}}
						/>
					</div>
				</section>
			</div>
		</div>
	)
}
