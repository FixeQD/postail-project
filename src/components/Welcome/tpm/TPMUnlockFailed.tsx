import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion, AnimatePresence } from 'framer-motion'
import { XCircle, RefreshCw, ShieldAlert, ArrowLeft, AlertTriangle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useSecurityTranslation } from '@/hooks/useTypedTranslation'
import { useTranslation } from 'react-i18next'
import { useThemeStore } from '@/stores/themeStore'
import type { TPMUnlockFailedProps } from '@/types/components/welcome'

type View = 'failed' | 'recovery'

export function TPMUnlockFailed({ error, onRetry, onUnlock }: TPMUnlockFailedProps) {
	const { t: tSec } = useSecurityTranslation()
	const { t } = useTranslation('welcome')
	const accentColor = useThemeStore((s) => s.accentColor)

	const [view, setView] = useState<View>('failed')
	const [phrase, setPhrase] = useState('')
	const [loading, setLoading] = useState(false)
	const [recoveryError, setRecoveryError] = useState<string | null>(null)

	const description = error?.cancelled
		? tSec('security:tpm.unlockFailed.cancelledDescription')
		: tSec('security:tpm.unlockFailed.errorDescription')

	const handleRecoverySubmit = async (e: React.FormEvent) => {
		e.preventDefault()
		setLoading(true)
		setRecoveryError(null)
		try {
			await invoke('unlock_with_recovery_phrase', { phrase })
			onUnlock()
		} catch (err) {
			console.error('Recovery unlock failed:', err)
			setRecoveryError(t('recovery.verify.error'))
		} finally {
			setLoading(false)
		}
	}

	return (
		<div className='noise-overlay flex h-full flex-col items-center justify-center px-6'>
			<AnimatePresence mode='wait' initial={false}>
				{view === 'failed' ? (
					<motion.div
						key='failed'
						initial={{ opacity: 0, y: 8 }}
						animate={{ opacity: 1, y: 0 }}
						exit={{ opacity: 0, y: -8 }}
						transition={{ duration: 0.2 }}
						className='w-full max-w-sm text-center'>
						<div className='mb-6 flex justify-center'>
							<div className='flex h-20 w-20 items-center justify-center rounded-2xl bg-slate-800/80 ring-1 ring-white/10'>
								<XCircle className='h-10 w-10 text-slate-400' />
							</div>
						</div>

						<h1 className='mb-3 text-2xl font-bold tracking-tight text-slate-100'>
							{tSec('security:tpm.unlockFailed.title')}
						</h1>
						<p className='mb-2 text-sm leading-relaxed text-slate-400'>{description}</p>
						{error && !error.cancelled && (
							<p className='mb-6 rounded-lg bg-slate-800/60 px-3 py-2 font-mono text-xs text-slate-500'>
								{error.message}
							</p>
						)}
						{error?.cancelled && <div className='mb-6' />}

						<div className='flex flex-col gap-3'>
							<Button onClick={onRetry} className='w-full gap-2'>
								<RefreshCw className='h-4 w-4' />
								{tSec('security:tpm.unlockFailed.retry')}
							</Button>

							<button
								type='button'
								onClick={() => {
									setView('recovery')
									setRecoveryError(null)
								}}
								className='w-full text-center text-xs text-slate-500 transition-colors hover:text-slate-300'>
								{tSec('security:tpm.unlockFailed.useRecovery')}
							</button>
						</div>
					</motion.div>
				) : (
					<motion.div
						key='recovery'
						initial={{ opacity: 0, y: 8 }}
						animate={{ opacity: 1, y: 0 }}
						exit={{ opacity: 0, y: -8 }}
						transition={{ duration: 0.2 }}
						className='w-full max-w-sm'>
						{/* Back */}
						<button
							type='button'
							onClick={() => {
								setView('failed')
								setRecoveryError(null)
								setPhrase('')
							}}
							className='group mb-8 flex items-center gap-2 text-sm text-slate-500 transition-colors hover:text-slate-200'>
							<ArrowLeft className='h-4 w-4 transition-transform group-hover:-translate-x-0.5' />
							Back
						</button>

						{/* Header */}
						<div className='mb-8 flex items-center gap-3'>
							<div
								className='flex h-10 w-10 items-center justify-center rounded-xl'
								style={{
									backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
									boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
								}}>
								<ShieldAlert className='h-5 w-5' style={{ color: accentColor }} />
							</div>
							<div>
								<h2 className='text-lg font-bold text-slate-100'>
									{t('recovery.unlock.title') || 'Recovery Unlock'}
								</h2>
								<p className='text-sm text-slate-500'>
									{t('recovery.unlock.hint')}
								</p>
							</div>
						</div>

						<form onSubmit={handleRecoverySubmit} className='space-y-5'>
							<div>
								<label className='mb-2 block text-sm font-medium text-slate-300'>
									{t('recovery.unlock.label')}
								</label>
								<textarea
									value={phrase}
									onChange={(e) => {
										setPhrase(e.target.value)
										setRecoveryError(null)
									}}
									className='min-h-[120px] w-full resize-none rounded-xl bg-slate-800/40 px-4 py-3 text-slate-100 placeholder-slate-600 ring-1 ring-white/[0.08] transition-all duration-200 focus:bg-slate-800/60 focus:outline-none'
									placeholder={t('recovery.unlock.placeholder')}
									required
									autoFocus
									style={
										{
											'--tw-ring-color': `rgba(var(--accent-rgb), 0.4)`,
										} as React.CSSProperties
									}
								/>
							</div>

							{recoveryError && (
								<motion.div
									initial={{ opacity: 0, y: -8, scale: 0.98 }}
									animate={{ opacity: 1, y: 0, scale: 1 }}
									transition={{ duration: 0.2 }}
									className='flex items-center gap-2.5 rounded-xl bg-red-500/10 p-4 ring-1 ring-red-500/20'>
									<AlertTriangle className='h-4 w-4 shrink-0 text-red-400' />
									<p className='text-sm text-red-400'>{recoveryError}</p>
								</motion.div>
							)}

							<motion.button
								type='submit'
								disabled={loading || !phrase.trim()}
								whileHover={!loading && phrase.trim() ? { scale: 1.02 } : {}}
								whileTap={!loading && phrase.trim() ? { scale: 0.97 } : {}}
								className='text-accent-contrast flex w-full items-center justify-center gap-2.5 rounded-xl px-6 py-3.5 text-sm font-semibold shadow-lg transition-all hover:shadow-xl disabled:cursor-not-allowed disabled:opacity-40 disabled:shadow-none'
								style={{
									background: `linear-gradient(to right, var(--accent-dark), var(--accent-color))`,
									boxShadow: `0 8px 24px -4px rgba(var(--accent-rgb), 0.2)`,
								}}>
								{loading ? (
									<>
										<div className='relative h-5 w-5'>
											<div className='border-accent-contrast border-t-accent-contrast absolute inset-0 animate-spin rounded-full border-2' />
										</div>
										Unlocking...
									</>
								) : (
									t('recovery.unlock.button')
								)}
							</motion.button>
						</form>
					</motion.div>
				)}
			</AnimatePresence>
		</div>
	)
}
