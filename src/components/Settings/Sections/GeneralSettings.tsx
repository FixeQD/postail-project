import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { Folder, Database, Coffee, Send, RotateCcw } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '@/stores/settingsStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { open } from '@tauri-apps/plugin-dialog'
import { toast } from '../../ui/custom/Toaster'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { ToggleSetting } from '@/components/ui/toggle-setting'

const SettingCard = ({ label, description, icon: Icon, children }: any) => (
	<div className='flex items-center justify-between rounded-2xl border border-white/5 bg-white/5 p-4 transition-colors hover:bg-white/10'>
		<div className='flex items-center gap-4'>
			<div className='flex h-10 w-10 items-center justify-center rounded-xl bg-slate-900 ring-1 ring-white/10'>
				<Icon className='h-5 w-5 text-slate-400' />
			</div>
			<div>
				<h3 className='text-sm font-semibold text-slate-200'>{label}</h3>
				<p className='max-w-[400px] text-xs text-slate-500'>{description}</p>
			</div>
		</div>
		<div className='flex items-center gap-2'>{children}</div>
	</div>
)

export function GeneralSettings() {
	const { t } = useSettingsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const { settings, setSetting } = useSettingsStore()
	const [isMigrationDialogOpen, setIsMigrationDialogOpen] = useState(false)
	const [pendingPath, setPendingPath] = useState<string | null>(null)
	const [defaultPath, setDefaultPath] = useState<string | null>(null)
	const [isMigrating, setIsMigrating] = useState(false)

	useEffect(() => {
		invoke<string>('get_default_data_dir').then(setDefaultPath)
	}, [])

	const isDefaultPath = settings['data-path'] === defaultPath

	const handlePathSelect = async () => {
		const selected = await open({
			directory: true,
			multiple: false,
			title: t('settings:general.storage.path.select'),
		})

		if (selected && typeof selected === 'string') {
			if (selected === settings['data-path']) {
				toast.info(t('settings:general.storage.path.alreadyCurrent'))
				return
			}
			setPendingPath(selected)
			setIsMigrationDialogOpen(true)
		}
	}

	const handleResetPath = async () => {
		try {
			const defaultPath = await invoke<string>('get_default_data_dir')
			if (settings['data-path'] === defaultPath) {
				toast.info('Already using default data path')
				return
			}
			setPendingPath(defaultPath)
			setIsMigrationDialogOpen(true)
		} catch (error) {
			console.error('Failed to get default path:', error)
		}
	}

	const handleConfirmMigration = async () => {
		if (!pendingPath) return

		setIsMigrating(true)
		setIsMigrationDialogOpen(false)

		try {
			toast.loading(t('settings:general.storage.migration.loading'), { id: 'migration' })
			await invoke('migrate_data_path', { newPath: pendingPath })
			// App will restart, so we don't need to clear toast
		} catch (error) {
			setIsMigrating(false)
			toast.error(`${t('settings:general.storage.migration.error')}: ${error}`, {
				id: 'migration',
			})
		}
	}

	return (
		<div className='mx-auto flex h-full w-full max-w-3xl flex-col space-y-8 p-8'>
			<motion.div
				{...(animationsEnabled
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
					{t('settings:general.title')}
				</h1>
				<p className='mt-1 text-slate-400'>{t('settings:general.subtitle')}</p>
			</motion.div>

			<div className='space-y-6'>
				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:general.interface.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Coffee}
							label={t('settings:general.interface.zenMode.label')}
							description={t('settings:general.interface.zenMode.description')}
							value={settings['zen-mode']}
							onChange={(val: boolean) => setSetting('zen-mode', val)}
						/>
					</div>
				</section>

				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:general.behavior.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Send}
							label={t('settings:general.behavior.strategicDelay.label')}
							description={t('settings:general.behavior.strategicDelay.description')}
							value={settings['undo-send-delay'] > 0}
							onChange={(val: boolean) => setSetting('undo-send-delay', val ? 10 : 0)}
						/>
					</div>
				</section>

				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:general.storage.title')}
					</h2>
					<div className='space-y-3'>
						<SettingCard
							icon={Database}
							label={t('settings:general.storage.path.label')}
							description={t('settings:general.storage.path.description')}>
							<div className='flex items-center gap-2'>
								<code className='rounded border border-white/5 bg-slate-900 px-2 py-1 text-[10px] text-slate-400'>
									{settings['data-path'] || 'Default'}
								</code>
								<div className='flex items-center gap-1.5'>
									<button
										type='button'
										disabled={isMigrating}
										onClick={handlePathSelect}
										title={t('settings:general.storage.path.select')}
										className='rounded-lg bg-white/5 p-2 text-slate-300 transition-colors hover:bg-white/10 disabled:opacity-50'>
										<Folder className='h-4 w-4' />
									</button>
									<button
										type='button'
										disabled={isMigrating || isDefaultPath}
										onClick={handleResetPath}
										title={t('settings:general.storage.migration.reset')}
										className='rounded-lg bg-white/5 p-2 text-slate-300 transition-colors hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-30'>
										<RotateCcw className='h-4 w-4' />
									</button>
								</div>
							</div>
						</SettingCard>
					</div>
				</section>
			</div>

			<Dialog open={isMigrationDialogOpen} onOpenChange={setIsMigrationDialogOpen}>
				<DialogContent className='border-slate-800 bg-slate-900 text-slate-100'>
					<DialogHeader>
						<DialogTitle>
							{t('settings:general.storage.migration.confirmTitle')}
						</DialogTitle>
						<DialogDescription className='text-slate-400'>
							{t('settings:general.storage.migration.confirmDescription')}
							<div className='mt-4 rounded-lg border border-blue-500/20 bg-blue-500/10 p-3 text-xs text-blue-400 italic'>
								{t('settings:general.storage.migration.newPath')}: <br />
								<span className='font-mono font-bold break-all'>{pendingPath}</span>
							</div>
						</DialogDescription>
					</DialogHeader>
					<DialogFooter>
						<Button
							variant='outline'
							onClick={() => setIsMigrationDialogOpen(false)}
							className='border-slate-700 bg-slate-800 text-slate-300 hover:bg-slate-700 hover:text-white'>
							{t('common:actions.cancel')}
						</Button>
						<Button
							onClick={handleConfirmMigration}
							className='bg-blue-600 font-bold text-white hover:bg-blue-500'>
							{t('settings:general.storage.migration.start')}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</div>
	)
}
