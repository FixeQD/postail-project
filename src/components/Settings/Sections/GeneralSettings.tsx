import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
	Folder,
	Database,
	Coffee,
	Send,
	RotateCcw,
	Trash2,
	Power,
	MonitorOff,
	BookOpen,
	AlignLeft,
	Clock,
	Layers,
} from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '@/stores/settingsStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
import { formatFileSize } from '@/lib/formatFileSize'
import { open } from '@tauri-apps/plugin-dialog'
import { toast } from '../../ui/custom/Toaster'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import type { SettingCardProps } from '@/types/components/shared'

const SettingCard = ({ label, description, icon: Icon, children }: SettingCardProps) => (
	<div className='flex items-center justify-between rounded-2xl border border-[var(--border-faint)] bg-[var(--surface-panel)] p-4 transition-colors hover:bg-[var(--surface-hover)]'>
		<div className='flex items-center gap-4'>
			<div className='flex h-10 w-10 items-center justify-center rounded-xl bg-[var(--surface-active)] ring-1 ring-[var(--border-subtle)]'>
				<Icon className='h-5 w-5 text-[var(--text-secondary)]' />
			</div>
			<div>
				<h3 className='text-sm font-semibold text-[var(--text-primary)]'>{label}</h3>
				<p className='max-w-[400px] text-xs text-[var(--text-secondary)]'>{description}</p>
			</div>
		</div>
		<div className='flex items-center gap-2'>{children}</div>
	</div>
)

function SectionTitle({ children }: { children: React.ReactNode }) {
	return (
		<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
			{children}
		</h2>
	)
}

interface InlineSelectProps {
	value: string | number
	options: { value: string | number; label: string }[]
	onChange: (v: string | number) => void
	accentColor: string
}

function InlineSelect({ value, options, onChange, accentColor }: InlineSelectProps) {
	return (
		<div className='flex flex-wrap justify-end gap-1'>
			{options.map((opt) => (
				<button
					key={String(opt.value)}
					type='button'
					onClick={() => onChange(opt.value)}
					className='rounded-lg px-3 py-1.5 text-xs font-semibold transition-all duration-150'
					style={
						value === opt.value
							? { backgroundColor: accentColor, color: '#fff' }
							: {
									color: 'var(--text-secondary)',
									boxShadow: 'inset 0 0 0 1px var(--border-subtle)',
								}
					}>
					{opt.label}
				</button>
			))}
		</div>
	)
}

export function GeneralSettings() {
	const { t } = useSettingsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const { settings, setSetting } = useSettingsStore()
	const accentColor = useThemeStore((s) => s.accentColor)

	const [isMigrationDialogOpen, setIsMigrationDialogOpen] = useState(false)
	const [pendingPath, setPendingPath] = useState<string | null>(null)
	const [defaultPath, setDefaultPath] = useState<string | null>(null)
	const [isMigrating, setIsMigrating] = useState(false)
	const [isClearCacheDialogOpen, setIsClearCacheDialogOpen] = useState(false)

	useEffect(() => {
		invoke<string>('get_default_data_dir').then(setDefaultPath)
	}, [])

	const currentPath = settings['data-path'] || null
	const isDefaultPath = !currentPath || currentPath === defaultPath

	const handlePathSelect = async () => {
		const selected = await open({
			directory: true,
			multiple: false,
			title: t('settings:general.storage.path.select'),
		})

		if (selected && typeof selected === 'string') {
			if (selected === currentPath) {
				toast.info(t('settings:general.storage.path.alreadyCurrent'))
				return
			}
			setPendingPath(selected)
			setIsMigrationDialogOpen(true)
		}
	}

	const handleResetPath = async () => {
		try {
			const fetchedDefault = await invoke<string>('get_default_data_dir')
			if (currentPath === fetchedDefault) {
				toast.info('Already using default data path')
				return
			}
			setPendingPath(fetchedDefault)
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
			toast.success(t('settings:general.storage.migration.success'), { id: 'migration' })
		} catch (error) {
			setIsMigrating(false)
			toast.error(`${t('settings:general.storage.migration.error')}: ${error}`, {
				id: 'migration',
			})
		}
	}

	const handleClearCache = async () => {
		setIsClearCacheDialogOpen(false)
		try {
			const freed = await invoke<number>('clear_cache')
			toast.success(
				`${t('settings:privacy.danger.clearCache.success')} (${formatFileSize(freed)} freed)`
			)
		} catch (error) {
			toast.error(`${t('settings:privacy.danger.clearCache.error')}: ${error}`)
		}
	}

	const markAsReadOptions = [
		{ value: 0, label: t('settings:general.reading.markAsRead.options.immediate') },
		{ value: 2, label: t('settings:general.reading.markAsRead.options.2s') },
		{ value: 5, label: t('settings:general.reading.markAsRead.options.5s') },
		{ value: -1, label: t('settings:general.reading.markAsRead.options.manual') },
	]

	const previewLineOptions = [
		{ value: 1, label: t('settings:general.reading.previewLines.options.1') },
		{ value: 2, label: t('settings:general.reading.previewLines.options.2') },
		{ value: 3, label: t('settings:general.reading.previewLines.options.3') },
	]

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
					{t('settings:general.title')}
				</h1>
				<p className='mt-1 text-[var(--text-secondary)]'>
					{t('settings:general.subtitle')}
				</p>
			</motion.div>

			<div className='space-y-8'>
				{/* Interface */}
				<motion.section {...fade(0.05)}>
					<SectionTitle>{t('settings:general.interface.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Coffee}
							label={t('settings:general.interface.zenMode.label')}
							description={t('settings:general.interface.zenMode.description')}
							value={settings['zen-mode']}
							onChange={(val) => setSetting('zen-mode', val)}
						/>
					</div>
				</motion.section>

				{/* Reading */}
				<motion.section {...fade(0.08)}>
					<SectionTitle>{t('settings:general.reading.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={BookOpen}
							label={t('settings:general.reading.threadView.label')}
							description={t('settings:general.reading.threadView.description')}
							value={settings['thread-view']}
							onChange={(val) => setSetting('thread-view', val)}
						/>
						<SettingCard
							icon={Clock}
							label={t('settings:general.reading.markAsRead.label')}
							description={t('settings:general.reading.markAsRead.description')}>
							<InlineSelect
								value={settings['mark-as-read-delay']}
								options={markAsReadOptions}
								onChange={(v) => setSetting('mark-as-read-delay', v as number)}
								accentColor={accentColor}
							/>
						</SettingCard>
						<SettingCard
							icon={AlignLeft}
							label={t('settings:general.reading.previewLines.label')}
							description={t('settings:general.reading.previewLines.description')}>
							<InlineSelect
								value={settings['preview-lines']}
								options={previewLineOptions}
								onChange={(v) => setSetting('preview-lines', v as number)}
								accentColor={accentColor}
							/>
						</SettingCard>
					</div>
				</motion.section>

				{/* Behavior */}
				<motion.section {...fade(0.11)}>
					<SectionTitle>{t('settings:general.behavior.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Send}
							label={t('settings:general.behavior.strategicDelay.label')}
							description={t('settings:general.behavior.strategicDelay.description')}
							value={settings['undo-send-delay'] > 0}
							onChange={(val) => setSetting('undo-send-delay', val ? 10 : 0)}
						/>
						<ToggleSetting
							icon={Trash2}
							label={t('settings:general.behavior.confirmBeforeDelete.label')}
							description={t(
								'settings:general.behavior.confirmBeforeDelete.description'
							)}
							value={settings['confirm-before-delete']}
							onChange={(val) => setSetting('confirm-before-delete', val)}
						/>
					</div>
				</motion.section>

				{/* Startup */}
				<motion.section {...fade(0.14)}>
					<SectionTitle>{t('settings:general.startup.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Power}
							label={t('settings:general.startup.openOnStartup.label')}
							description={t('settings:general.startup.openOnStartup.description')}
							value={settings['open-on-startup']}
							onChange={(val) => setSetting('open-on-startup', val)}
						/>
						<ToggleSetting
							icon={MonitorOff}
							label={t('settings:general.startup.minimizeToTray.label')}
							description={t('settings:general.startup.minimizeToTray.description')}
							value={settings['minimize-to-tray']}
							onChange={(val) => setSetting('minimize-to-tray', val)}
						/>
					</div>
				</motion.section>

				{/* Storage */}
				<motion.section {...fade(0.17)}>
					<SectionTitle>{t('settings:general.storage.title')}</SectionTitle>
					<div className='space-y-3'>
						<SettingCard
							icon={Database}
							label={t('settings:general.storage.path.label')}
							description={t('settings:general.storage.path.description')}>
							<div className='flex items-center gap-2'>
								<code className='rounded border border-[var(--border-faint)] bg-[var(--surface-active)] px-2 py-1 text-[10px] text-[var(--text-secondary)]'>
									{currentPath || 'Default'}
								</code>
								<div className='flex items-center gap-1.5'>
									<button
										type='button'
										disabled={isMigrating}
										onClick={handlePathSelect}
										title={t('settings:general.storage.path.select')}
										className='rounded-lg bg-[var(--surface-panel)] p-2 text-[var(--text-primary)] transition-colors hover:bg-[var(--surface-hover)] disabled:opacity-50'>
										<Folder className='h-4 w-4' />
									</button>
									<button
										type='button'
										disabled={isMigrating || isDefaultPath}
										onClick={handleResetPath}
										title={t('settings:general.storage.migration.reset')}
										className='rounded-lg bg-[var(--surface-panel)] p-2 text-[var(--text-primary)] transition-colors hover:bg-[var(--surface-hover)] disabled:cursor-not-allowed disabled:opacity-30'>
										<RotateCcw className='h-4 w-4' />
									</button>
								</div>
							</div>
						</SettingCard>

						<SettingCard
							icon={Layers}
							label={t('settings:privacy.danger.clearCache.label')}
							description={t('settings:privacy.danger.clearCache.description')}>
							<button
								type='button'
								onClick={() => setIsClearCacheDialogOpen(true)}
								className='rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-2 text-xs font-semibold text-red-400 transition-all hover:border-red-500/50 hover:bg-red-500/20'>
								{t('settings:privacy.danger.clearCache.button')}
							</button>
						</SettingCard>
					</div>
				</motion.section>
			</div>

			<ConfirmationDialog
				open={isMigrationDialogOpen}
				onOpenChange={setIsMigrationDialogOpen}
				title={t('settings:general.storage.migration.confirmTitle')}
				description={t('settings:general.storage.migration.confirmDescription')}
				cancelLabel={t('common:actions.cancel')}
				confirmLabel={t('settings:general.storage.migration.start')}
				onConfirm={handleConfirmMigration}
				confirmClassName='w-full border-0 font-bold shadow-lg bg-blue-600 text-white hover:bg-blue-500'>
				<div className='mt-4 rounded-lg border border-blue-500/20 bg-blue-500/10 p-3 text-xs text-blue-400 italic'>
					{t('settings:general.storage.migration.newPath')}: <br />
					<span className='font-mono font-bold break-all'>{pendingPath}</span>
				</div>
			</ConfirmationDialog>

			<ConfirmationDialog
				open={isClearCacheDialogOpen}
				onOpenChange={setIsClearCacheDialogOpen}
				title={t('settings:privacy.danger.clearCache.label')}
				description={t('settings:privacy.danger.clearCache.confirm')}
				cancelLabel={t('common:actions.cancel')}
				confirmLabel={t('settings:privacy.danger.clearCache.button')}
				onConfirm={handleClearCache}
				confirmClassName='w-full border-0 font-bold shadow-lg bg-red-600 text-white hover:bg-red-500'
			/>
		</div>
	)
}
