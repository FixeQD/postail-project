import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { ArrowLeft, FolderOpen, HardDrive, AlertTriangle, Check } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useTranslation } from 'react-i18next'

interface DataDirStepProps {
	onBack: () => void
	onDataDirSet: () => void
}

export const DataDirStep = ({ onBack, onDataDirSet }: DataDirStepProps) => {
	const { t } = useTranslation()
	const [selectedPath, setSelectedPath] = useState<string | null>(null)
	const [defaultPath, setDefaultPath] = useState<string | null>(null)
	const [isApplying, setIsApplying] = useState(false)
	const [error, setError] = useState<string | null>(null)

	useEffect(() => {
		invoke<string>('get_default_data_dir').then(setDefaultPath)
	}, [])

	const handleBrowse = async () => {
		const selected = await open({
			directory: true,
			multiple: false,
			title: t('welcome:dataDir.select'),
		})

		if (selected && typeof selected === 'string') {
			setSelectedPath(selected)
			setError(null)
		}
	}

	const handleConfirm = async () => {
		if (!selectedPath) return

		setIsApplying(true)
		setError(null)

		try {
			await invoke('set_initial_data_dir', { path: selectedPath })
			onDataDirSet()
		} catch (e) {
			setError(typeof e === 'string' ? e : t('welcome:dataDir.error'))
			setIsApplying(false)
		}
	}

	const isDefault = selectedPath === defaultPath

	return (
		<div className='noise-overlay relative flex h-full flex-col'>
			{/* Header */}
			<motion.div
				initial={{ opacity: 0, y: -20 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
				className='glass relative border-b border-black/5 bg-white/10 px-4 py-6 shadow-sm dark:border-white/5 dark:bg-black/20'>
				<div className='pointer-events-none absolute inset-x-0 bottom-0 h-px bg-gradient-to-r from-transparent via-[var(--accent-color)] to-transparent opacity-20' />

				<div className='container mx-auto'>
					<button
						type='button'
						onClick={onBack}
						className='text-muted-foreground hover:text-foreground group mb-6 flex items-center gap-2 text-sm transition-colors'>
						<ArrowLeft className='h-4 w-4 transition-transform group-hover:-translate-x-0.5' />
						{t('common:actions.back')}
					</button>
					<div className='flex items-center gap-3'>
						<div
							className='flex h-10 w-10 items-center justify-center rounded-xl ring-1'
							style={{
								backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
								boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
							}}>
							<HardDrive
								className='h-5 w-5'
								style={{ color: 'var(--accent-color)' }}
							/>
						</div>
						<div>
							<h1 className='text-foreground text-3xl font-bold tracking-tight'>
								{t('welcome:dataDir.title')}
							</h1>
							<p className='text-muted-foreground mt-1 text-sm'>
								{t('welcome:dataDir.subtitle')}
							</p>
						</div>
					</div>
				</div>
			</motion.div>

			{/* Content */}
			<div className='container mx-auto flex flex-1 items-center justify-center px-4 py-8'>
				<div className='w-full max-w-lg space-y-8'>
					<motion.div
						initial={{ opacity: 0, y: 24 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.1, duration: 0.6, ease: [0.16, 1, 0.3, 1] }}>
						{/* Info card */}
						<div className='mb-8 flex items-start gap-3 rounded-2xl border border-amber-500/20 bg-amber-500/10 p-5 shadow-lg'>
							<AlertTriangle className='mt-0.5 h-5 w-5 shrink-0 text-amber-500' />
							<p className='text-foreground/90 text-sm leading-relaxed font-medium'>
								{t('welcome:dataDir.warning')}
							</p>
						</div>

						{/* Path picker - Dropzone style */}
						<div className='space-y-4'>
							<label className='text-muted-foreground text-xs font-bold tracking-widest uppercase'>
								{t('welcome:dataDir.pathLabel')}
							</label>

							<motion.button
								type='button'
								onClick={handleBrowse}
								whileHover={{ scale: 1.02 }}
								whileTap={{ scale: 0.98 }}
								className={`group relative flex w-full flex-col items-center justify-center gap-4 rounded-3xl border-2 border-dashed p-10 text-center transition-all focus:ring-2 focus:ring-offset-2 focus:ring-offset-[var(--app-bg)] focus:outline-none ${
									selectedPath
										? 'border-[var(--accent-color)] bg-[var(--surface-panel)]'
										: 'border-[var(--border-strong)] bg-[var(--surface-active)] hover:border-[var(--accent-color)] hover:bg-[var(--surface-panel)]'
								}`}
								style={
									selectedPath
										? {
												backgroundColor: `rgba(var(--accent-rgb), 0.05)`,
												boxShadow: `0 8px 32px -12px rgba(var(--accent-rgb), 0.2)`,
											}
										: {}
								}>
								<div
									className='flex h-16 w-16 items-center justify-center rounded-2xl transition-transform duration-300 group-hover:scale-110 group-hover:shadow-xl'
									style={
										selectedPath
											? {
													backgroundColor: 'var(--accent-color)',
													color: 'var(--accent-contrast)',
												}
											: {
													backgroundColor: 'var(--surface-hover)',
													color: 'var(--text-secondary)',
													boxShadow: 'inset 0 2px 4px rgba(0,0,0,0.1)',
												}
									}>
									{selectedPath ? (
										<Check className='h-8 w-8' />
									) : (
										<FolderOpen className='h-8 w-8' />
									)}
								</div>

								<div className='flex flex-col items-center gap-1.5'>
									{selectedPath ? (
										<>
											<p className='text-foreground max-w-sm truncate font-mono text-sm font-semibold'>
												{selectedPath}
											</p>
											{isDefault && (
												<p className='text-tertiary text-xs font-medium'>
													{t('welcome:dataDir.defaultHint')}
												</p>
											)}
										</>
									) : (
										<>
											<p className='text-foreground text-base font-semibold'>
												{t('welcome:dataDir.placeholder')}
											</p>
											<p className='text-muted-foreground text-sm'>
												Click to browse your computer
											</p>
										</>
									)}
								</div>
							</motion.button>
						</div>

						{/* Error */}
						{error && (
							<motion.p
								initial={{ opacity: 0, y: -4 }}
								animate={{ opacity: 1, y: 0 }}
								className='mt-3 text-sm text-red-400'>
								{error}
							</motion.p>
						)}
					</motion.div>
				</div>
			</div>

			{/* Footer */}
			<motion.div
				initial={{ opacity: 0, y: 16 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ delay: 0.3, duration: 0.4 }}
				className='glass relative border-t border-black/[0.06] bg-white/30 px-4 py-5 dark:border-white/[0.06] dark:bg-slate-900/30'>
				<div className='pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-black/[0.06] to-transparent dark:via-white/[0.06]' />
				<div className='container mx-auto flex justify-end'>
					<motion.button
						type='button'
						onClick={handleConfirm}
						disabled={!selectedPath || isApplying}
						whileHover={selectedPath && !isApplying ? { scale: 1.03, y: -1 } : {}}
						whileTap={selectedPath && !isApplying ? { scale: 0.97 } : {}}
						className='text-accent-contrast flex items-center gap-2.5 rounded-xl px-8 py-3 text-sm font-semibold shadow-lg transition-all hover:shadow-xl disabled:cursor-not-allowed disabled:opacity-40'
						style={{
							background: `linear-gradient(to right, var(--accent-dark), var(--accent-color))`,
							boxShadow: `0 8px 24px -4px rgba(var(--accent-rgb), 0.2)`,
						}}>
						{isApplying ? t('welcome:dataDir.loading') : t('welcome:dataDir.confirm')}
					</motion.button>
				</div>
			</motion.div>
		</div>
	)
}
