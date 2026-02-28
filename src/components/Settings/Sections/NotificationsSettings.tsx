import { motion } from 'framer-motion'
import { Bell, Volume2, Star } from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useNotificationStore } from '@/stores/notificationStore'

export function NotificationsSettings() {
	const { t } = useSettingsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const prefs = useNotificationStore((s) => s.prefs)
	const setPref = useNotificationStore((s) => s.setPref)

	return (
		<div className='mx-auto flex h-full w-full max-w-3xl flex-col space-y-8 p-8'>
			<motion.div
				{...(animationsEnabled
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
					{t('settings:notifications.title')}
				</h1>
				<p className='mt-1 text-slate-400'>{t('settings:notifications.subtitle')}</p>
			</motion.div>

			<div className='space-y-6'>
				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:notifications.alerts.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Bell}
							label={t('settings:notifications.alerts.desktop.label')}
							description={t('settings:notifications.alerts.desktop.description')}
							value={prefs.enabled}
							onChange={(v) => setPref('enabled', v)}
						/>
						<ToggleSetting
							icon={Volume2}
							label={t('settings:notifications.alerts.sound.label')}
							description={t('settings:notifications.alerts.sound.description')}
							value={prefs.sound}
							onChange={(v) => setPref('sound', v)}
							disabled={!prefs.enabled}
						/>
					</div>
				</section>

				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:notifications.filters.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Star}
							label={t('settings:notifications.filters.importantOnly.label')}
							description={t(
								'settings:notifications.filters.importantOnly.description'
							)}
							value={prefs.importantOnly}
							onChange={(v) => setPref('importantOnly', v)}
							disabled={!prefs.enabled}
						/>
					</div>
				</section>
			</div>
		</div>
	)
}
