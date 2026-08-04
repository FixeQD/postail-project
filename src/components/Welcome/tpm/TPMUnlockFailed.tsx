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

export function TPMUnlockFailed({
	error,
	onRetry,
	onUnlock: _onUnlock,
	onRecoveryVerified,
}: TPMUnlockFailedProps) {
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
			onRecoveryVerified()
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
							<div className='flex h-20 w-20 items-center justify-center rounded-2xl bg-[var(--surface-active)] ring-1 ring-[var(--border-subtle)]'>
								<XCircle className='h-10 w-10 text-[var(--text-secondary)]' />
							</div>
						</div>

						<h1 className='mb-3 text-2xl font-bold tracking-tight text-[var(--text-primary)]'>
							{tSec('security:tpm.unlockFailed.title')}
						</h1>
						<p className='mb-2 text-sm leading-relaxed text-[var(--text-secondary)]'>
							{description}
						</p>
						{error && !error.cancelled && (
							<p className='mb-6 rounded-lg bg-[var(--surface-active)] px-3 py-2 font-mono text-xs text-[var(--text-secondary)]'>
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
								className='w-full text-center text-xs text-[var(--text-secondary)] transition-colors hover:text-[var(--text-primary)]'>
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
							className='group mb-8 flex items-center gap-2 text-sm text-[var(--text-secondary)] transition-colors hover:text-[var(--text-primary)]'>
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
								<h2 className='text-lg font-bold text-[var(--text-primary)]'>
									{t('recovery.unlock.title') || 'Recovery Unlock'}
								</h2>
								<p className='text-sm text-[var(--text-secondary)]'>
									{t('recovery.unlock.hint')}
								</p>
							</div>
						</div>

						<form onSubmit={handleRecoverySubmit} className='space-y-5'>
							<div>
								<label className='mb-2 block text-sm font-medium text-[var(--text-primary)]'>
									{t('recovery.unlock.label')}
								</label>
								<textarea
									value={phrase}
									onChange={(e) => {
										setPhrase(e.target.value)
										setRecoveryError(null)
									}}
									className='min-h-[120px] w-full resize-none rounded-xl bg-[var(--surface-panel)] px-4 py-3 text-[var(--text-primary)] placeholder-[var(--text-tertiary)] ring-1 ring-[var(--border-subtle)] transition-all duration-200 focus:bg-[var(--surface-hover)] focus:outline-none'
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
									className='flex items-center gap-2.5 rounded-xl bg-destructive/15 p-4 ring-1 ring-destructive/30'>
									<AlertTriangle className='h-4 w-4 shrink-0 text-destructive' />
									<p className='text-sm text-destructive'>{recoveryError}</p>
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
