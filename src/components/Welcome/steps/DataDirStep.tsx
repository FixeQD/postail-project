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
				initial={{ opacity: 0, y: -10 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
				className='relative border-b border-black/[0.06] bg-white/40 px-4 py-6 backdrop-blur-lg dark:border-white/[0.06] dark:bg-slate-900/40'>
				<div className='pointer-events-none absolute inset-x-0 bottom-0 h-px bg-gradient-to-r from-transparent via-[rgba(var(--accent-rgb),0.1)] to-transparent' />

				<div className='container mx-auto'>
					<button
						type='button'
						onClick={onBack}
						className='group mb-6 flex items-center gap-2 text-sm text-slate-500 transition-colors hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100'>
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
							<h1 className='text-3xl font-bold tracking-tight text-slate-900 dark:text-slate-100'>
								{t('welcome:dataDir.title')}
							</h1>
							<p className='mt-1 text-sm text-slate-500 dark:text-slate-400'>
								{t('welcome:dataDir.subtitle')}
							</p>
						</div>
					</div>
				</div>
			</motion.div>

			{/* Content */}
			<div className='container mx-auto flex flex-1 items-center px-4 py-8'>
				<div className='mx-auto w-full max-w-lg space-y-6'>
					<motion.div
						initial={{ opacity: 0, y: 20 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.1, duration: 0.5, ease: [0.16, 1, 0.3, 1] }}>
						{/* Info card */}
						<div className='mb-6 flex items-start gap-3 rounded-xl border border-amber-500/20 bg-amber-500/[0.06] p-4'>
							<AlertTriangle className='mt-0.5 h-4 w-4 shrink-0 text-amber-400' />
							<p className='text-sm leading-relaxed text-amber-300/80'>
								{t('welcome:dataDir.warning')}
							</p>
						</div>

						{/* Path picker */}
						<div className='space-y-3'>
							<label className='text-xs font-bold tracking-widest text-slate-400 uppercase dark:text-slate-500'>
								{t('welcome:dataDir.pathLabel')}
							</label>

							<button
								type='button'
								onClick={handleBrowse}
								className='group flex w-full items-center gap-4 rounded-2xl border border-black/[0.08] bg-black/[0.02] p-4 text-left transition-all hover:border-black/[0.14] hover:bg-black/[0.04] dark:border-white/[0.08] dark:bg-white/[0.03] dark:hover:border-white/[0.14] dark:hover:bg-white/[0.06]'>
								<div className='flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-black/[0.06] ring-1 ring-black/10 dark:bg-slate-800 dark:ring-white/10'>
									<FolderOpen className='h-5 w-5 text-slate-500 transition-colors group-hover:text-slate-700 dark:text-slate-400 dark:group-hover:text-slate-200' />
								</div>
								<div className='min-w-0 flex-1'>
									{selectedPath ? (
										<>
											<p className='truncate font-mono text-sm text-slate-700 dark:text-slate-200'>
												{selectedPath}
											</p>
											{isDefault && (
												<p className='mt-0.5 text-xs text-slate-400 dark:text-slate-500'>
													{t('welcome:dataDir.defaultHint')}
												</p>
											)}
										</>
									) : (
										<p className='text-sm text-slate-400 dark:text-slate-500'>
											{t('welcome:dataDir.placeholder')}
										</p>
									)}
								</div>
								{selectedPath && (
									<Check className='h-4 w-4 shrink-0 text-emerald-400' />
								)}
							</button>
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
				className='relative border-t border-black/[0.06] bg-white/30 px-4 py-5 backdrop-blur-lg dark:border-white/[0.06] dark:bg-slate-900/30'>
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
