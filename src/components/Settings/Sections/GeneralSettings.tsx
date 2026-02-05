import { motion } from 'framer-motion'
import { EyeOff, Clock, HardDrive } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { useSettingsStore } from '@/stores/settingsStore'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'

const Toggle = ({ value, onChange, label, description, icon: Icon }: any) => (
	<div className='flex items-center justify-between p-4 rounded-2xl bg-white/5 border border-white/5 hover:bg-white/10 transition-colors'>
		<div className='flex items-center gap-4'>
			<div className='flex h-10 w-10 items-center justify-center rounded-xl bg-slate-900 ring-1 ring-white/10'>
				<Icon className='h-5 w-5 text-slate-400' />
			</div>
			<div>
				<h3 className='text-sm font-semibold text-slate-200'>{label}</h3>
				<p className='text-xs text-slate-500'>{description}</p>
			</div>
		</div>
		<button
			type='button'
			onClick={() => onChange(!value)}
			className={`relative h-6 w-11 rounded-full transition-colors ${
				value ? 'bg-blue-600' : 'bg-slate-800'
			}`}>
			<motion.div
				transition={{
					type: 'spring',
					stiffness: 500,
					damping: 30,
				}}
				animate={{ x: value ? 22 : 2 }}
				className='absolute top-1 left-0 h-4 w-4 rounded-full bg-white shadow-sm'
			/>
		</button>
	</div>
)

export function GeneralSettings() {
	const { t } = useSettingsTranslation()
	const { settings, setSetting } = useSettingsStore()

	const handlePickPath = async () => {
		try {
			const selected = await open({
				directory: true,
				multiple: false,
				title: t('settings:general.storage.dataNomat.label'),
			})
			if (selected) {
				await setSetting('data-path', selected)
				// TODO: Implement moving files to new directory and updating database path
			}
		} catch (error) {
			console.error('Failed to pick directory:', error)
		}
	}

	return (
		<div className='flex h-full flex-col p-8 max-w-3xl mx-auto w-full space-y-8'>
			<motion.div
				initial={{ opacity: 0, y: -20 }}
				animate={{ opacity: 1, y: 0 }}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>{t('settings:general.title')}</h1>
				<p className='mt-1 text-slate-400'>{t('settings:general.subtitle')}</p>
			</motion.div>

			<div className='space-y-6'>
				<section>
					<h2 className='text-xs font-bold uppercase tracking-widest text-slate-500 mb-4 ml-2'>
						{t('settings:general.interface.title')}
					</h2>
					<div className='space-y-3'>
						<Toggle
							icon={EyeOff}
							label={t('settings:general.interface.zenMode.label')}
							description={t('settings:general.interface.zenMode.description')}
							value={settings['zen-mode']}
							onChange={(val: boolean) => setSetting('zen-mode', val)}
						/>
					</div>
				</section>

				<section>
					<h2 className='text-xs font-bold uppercase tracking-widest text-slate-500 mb-4 ml-2'>
						{t('settings:general.behavior.title')}
					</h2>
					<div className='space-y-3'>
						<Toggle
							icon={Clock}
							label={t('settings:general.behavior.strategicDelay.label')}
							description={t('settings:general.behavior.strategicDelay.description')}
							value={settings['undo-send-delay'] > 0}
							onChange={(val: boolean) => setSetting('undo-send-delay', val ? 10 : 0)}
						/>
					</div>
				</section>

				<section>
					<h2 className='text-xs font-bold uppercase tracking-widest text-slate-500 mb-4 ml-2'>
						{t('settings:general.storage.title')}
					</h2>
					<div className='p-4 rounded-2xl bg-white/5 border border-white/5'>
						<div className='flex items-center gap-4 mb-4'>
							<div className='flex h-10 w-10 items-center justify-center rounded-xl bg-slate-900 ring-1 ring-white/10'>
								<HardDrive className='h-5 w-5 text-slate-400' />
							</div>
							<div>
								<h3 className='text-sm font-semibold text-slate-200'>
									{t('settings:general.storage.dataNomat.label')}
								</h3>
								<p className='text-xs text-slate-500'>
									{t('settings:general.storage.dataNomat.description')}
								</p>
							</div>
						</div>
						<div className='flex gap-2 bg-slate-950/50 p-2 rounded-xl border border-white/5'>
							<code className='text-[10px] text-slate-400 flex-1 py-1 px-2 overflow-hidden text-ellipsis'>
								{settings['data-path'] || t('settings:general.storage.defaultPath')}
							</code>
							<button
								onClick={handlePickPath}
								className='text-[10px] font-bold text-blue-400 hover:text-blue-300 px-2'>
								{t('settings:general.storage.change')}
							</button>
						</div>
					</div>
				</section>
			</div>
		</div>
	)
}
