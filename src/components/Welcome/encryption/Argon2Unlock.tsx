import { useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion, AnimatePresence } from 'framer-motion'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
import { Lock, Eye, EyeOff, AlertTriangle, ArrowRight } from 'lucide-react'
import icon from '@/assets/icon.png'

export const Argon2Unlock = ({
	onUnlock,
	onRecoveryVerified,
}: {
	onUnlock: () => void
	onRecoveryVerified?: () => void
}) => {
	const { t } = useTypedTranslation(['common', 'security', 'welcome'])
	const accentColor = useThemeStore((s) => s.accentColor)
	const [passphrase, setPassphrase] = useState('')
	const [recoveryPhrase, setRecoveryPhrase] = useState('')
	const [isRecoveryMode, setIsRecoveryMode] = useState(false)
	const [showPassword, setShowPassword] = useState(false)
	const [loading, setLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)
	const [shake, setShake] = useState(0)

	const inputRef = useRef<HTMLInputElement>(null)
	const recoveryRef = useRef<HTMLTextAreaElement>(null)

	// Focus input on mode change
	useEffect(() => {
		const timer = setTimeout(() => {
			if (isRecoveryMode) {
				recoveryRef.current?.focus()
			} else {
				inputRef.current?.focus()
			}
		}, 350)
		return () => clearTimeout(timer)
	}, [isRecoveryMode])

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault()
		if (loading) return

		setLoading(true)
		setError(null)

		try {
			if (isRecoveryMode) {
				await invoke('unlock_with_recovery_phrase', {
					phrase: recoveryPhrase,
				})
				if (onRecoveryVerified) {
					onRecoveryVerified()
				} else {
					onUnlock()
				}
			} else {
				await invoke('initialize_security', {
					method: 'argon2',
					passphrase,
				})
				onUnlock()
			}
		} catch (err) {
			console.error('Failed to unlock:', err)
			setError(isRecoveryMode ? t('welcome:recovery.verify.error') : 'Incorrect password')
			setShake((prev) => prev + 1)
			setPassphrase('') // Clear on fail for better UX (security practice)
		} finally {
			setLoading(false)
			// Re-focus after error
			setTimeout(() => {
				if (isRecoveryMode) recoveryRef.current?.focus()
				else inputRef.current?.focus()
			}, 100)
		}
	}

	return (
		<div className='relative flex min-h-dvh flex-col items-center justify-center overflow-hidden bg-[var(--app-bg)] text-[var(--text-primary)]'>
			{/* Ambient Background Effects */}
			<div className='absolute inset-0 overflow-hidden'>
				<div
					className='absolute -top-[10%] left-[10%] h-[50vh] w-[50vh] rounded-full opacity-20 blur-[120px] filter dark:opacity-[0.08]'
					style={{ backgroundColor: accentColor }}
				/>
				<div className='absolute right-[5%] bottom-[10%] h-[40vh] w-[40vh] rounded-full bg-status-info/15 opacity-50 blur-[100px] filter dark:opacity-40' />
			</div>

			{/* Glass Card Container */}
			<motion.div
				initial={{ opacity: 0, scale: 0.95, y: 20 }}
				animate={{ opacity: 1, scale: 1, y: 0 }}
				transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
				className='relative z-10 w-full max-w-[400px] p-6'>
				{/* Logo / Header Section */}
				<div className='mb-10 flex flex-col items-center text-center'>
					<motion.div
						initial={{ scale: 0.8, opacity: 0 }}
						animate={{ scale: 1, opacity: 1 }}
						transition={{ delay: 0.1, duration: 0.5 }}
						className='relative mb-6'>
						<div className='bg-surface-panel relative flex h-24 w-24 items-center justify-center rounded-3xl shadow-2xl ring-1 ring-[var(--border-subtle)] backdrop-blur'>
							<img
								src={icon}
								alt='Postail'
								className='h-14 w-14 object-contain drop-shadow-sm'
							/>
							{/* Pulse glow behind logo */}
							<div
								className='absolute inset-0 -z-10 rounded-3xl opacity-40 blur-xl transition-all duration-1000'
								style={{ backgroundColor: `${accentColor}40` }}
							/>
						</div>
					</motion.div>

					<motion.h1
						initial={{ opacity: 0, y: 10 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.2 }}
						className='text-3xl font-semibold tracking-[-0.02em] text-[var(--text-primary)]'>
						Welcome Back
					</motion.h1>
					<motion.p
						initial={{ opacity: 0 }}
						animate={{ opacity: 1 }}
						transition={{ delay: 0.3 }}
						className='mt-2 max-w-[40ch] text-balance text-sm font-medium text-[var(--text-secondary)]'>
						{isRecoveryMode
							? 'Use your recovery phrase to regain access.'
							: 'Enter your password to unlock your vault.'}
					</motion.p>
				</div>

				{/* Unlock Form */}
				<motion.form
					onSubmit={handleSubmit}
					animate={{ x: shake % 2 === 0 ? 0 : [-10, 10, -10, 10, 0] }}
					transition={{ type: 'spring', stiffness: 400, damping: 10 }}
					className='relative space-y-4'>
					<AnimatePresence mode='wait'>
						{isRecoveryMode ? (
							<motion.div
								key='recovery'
								initial={{ opacity: 0, x: 20 }}
								animate={{ opacity: 1, x: 0 }}
								exit={{ opacity: 0, x: -20 }}
								transition={{ duration: 0.2 }}>
								<div className='relative'>
									<textarea
										ref={recoveryRef}
										disabled={loading}
										value={recoveryPhrase}
										onChange={(e) => setRecoveryPhrase(e.target.value)}
										placeholder={t('welcome:recovery.unlock.placeholder')}
										className='min-h-[140px] w-full resize-none rounded-2xl border-0 bg-[var(--surface-panel)] px-4 py-4 text-sm leading-relaxed text-[var(--text-primary)] placeholder-[var(--text-tertiary)] shadow-sm ring-1 ring-[var(--border-subtle)] transition-all focus:bg-[var(--surface-hover)] focus:ring-2 focus:ring-[var(--accent-color)] disabled:cursor-not-allowed disabled:opacity-50'
										style={
											{
												'--tw-ring-focus': accentColor,
											} as React.CSSProperties
										}
										spellCheck={false}
									/>
								</div>
							</motion.div>
						) : (
							<motion.div
								key='password'
								initial={{ opacity: 0, x: -20 }}
								animate={{ opacity: 1, x: 0 }}
								exit={{ opacity: 0, x: 20 }}
								transition={{ duration: 0.2 }}>
								<div className='group relative'>
									<div className='pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4'>
										<Lock
											className='h-5 w-5 transition-colors'
											style={{
												color: passphrase
													? accentColor
													: 'var(--text-tertiary)',
											}}
										/>
									</div>
									<input
										ref={inputRef}
										disabled={loading}
										type={showPassword ? 'text' : 'password'}
										value={passphrase}
										onChange={(e) => setPassphrase(e.target.value)}
										className='h-14 w-full rounded-2xl border-0 bg-[var(--surface-panel)] pr-12 pl-12 text-base font-medium text-[var(--text-primary)] placeholder-[var(--text-tertiary)] shadow-sm ring-1 ring-[var(--border-subtle)] transition-all focus:bg-[var(--surface-hover)] focus:ring-2 focus:ring-[var(--accent-color)] disabled:cursor-not-allowed disabled:opacity-50'
										placeholder={t('security:argon2.passphrase.placeholder')}
										style={
											{
												'--tw-ring-focus': accentColor,
											} as React.CSSProperties
										}
									/>
									<button
										type='button'
										tabIndex={-1}
										onClick={() => setShowPassword(!showPassword)}
										className='absolute top-1/2 right-3 -translate-y-1/2 rounded-lg p-2 text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
										{showPassword ? (
											<EyeOff className='h-5 w-5' />
										) : (
											<Eye className='h-5 w-5' />
										)}
									</button>
								</div>
							</motion.div>
						)}
					</AnimatePresence>

					{/* Error Feedback */}
					<AnimatePresence>
						{error && (
							<motion.div
								initial={{ opacity: 0, height: 0, marginTop: 0 }}
								animate={{ opacity: 1, height: 'auto', marginTop: 8 }}
								exit={{ opacity: 0, height: 0, marginTop: 0 }}
								className='overflow-hidden'>
								<div className='flex items-center gap-2 rounded-xl bg-destructive/15 px-4 py-3 text-sm font-medium text-destructive'>
									<AlertTriangle className='h-4 w-4 shrink-0' />
									{error}
								</div>
							</motion.div>
						)}
					</AnimatePresence>

					{/* Action Button */}
					<motion.button
						whileHover={{ scale: 1.01 }}
						whileTap={{ scale: 0.98 }}
						disabled={loading || (isRecoveryMode ? !recoveryPhrase : !passphrase)}
						type='submit'
						className='relative mt-6 flex h-14 w-full items-center justify-center gap-2.5 rounded-2xl text-base font-semibold text-white transition-all disabled:cursor-not-allowed disabled:opacity-50 disabled:shadow-none'
						style={{
							backgroundImage: 'linear-gradient(180deg, var(--accent-light), var(--accent-color))',
							boxShadow: '0 10px 30px -10px rgba(var(--accent-rgb), 0.55)',
						}}>
						{loading ? (
							<div className='h-5 w-5 animate-spin rounded-full border-2 border-white/30 border-t-white' />
						) : (
							<>
								<span>{isRecoveryMode ? 'Recover Account' : 'Unlock Vault'}</span>
								{!isRecoveryMode && <ArrowRight className='h-5 w-5 opacity-80' />}
							</>
						)}
					</motion.button>

					{/* Toggle Mode Link */}
					<div className='mt-6 text-center'>
						<button
							type='button'
							disabled={loading}
							onClick={() => {
								setIsRecoveryMode(!isRecoveryMode)
								setError(null)
								setPassphrase('')
								setRecoveryPhrase('')
							}}
							className='rounded-lg px-2 py-1 text-sm font-medium text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] disabled:opacity-50'>
							{isRecoveryMode
								? 'Remember your password? Log in'
								: 'Forgot password? Use recovery phrase'}
						</button>
					</div>
				</motion.form>
			</motion.div>

			{/* Footer Info */}
			<div className='absolute bottom-8 font-mono text-[11px] tracking-wide text-[var(--text-tertiary)] opacity-60'>
				Secured with Argon2id Encryption
			</div>
		</div>
	)
}
