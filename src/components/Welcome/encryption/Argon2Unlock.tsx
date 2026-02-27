import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion } from 'framer-motion'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
import { ArrowLeft, Lock, Eye, EyeOff, AlertTriangle } from 'lucide-react'
import icon from '@/assets/icon.png'

export const Argon2Unlock = ({
	onBack,
	onUnlock,
}: {
	onBack: () => void
	onUnlock: () => void
}) => {
	const { t } = useTypedTranslation(['common', 'security', 'welcome'])
	const accentColor = useThemeStore((s) => s.accentColor)
	const [passphrase, setPassphrase] = useState('')
	const [recoveryPhrase, setRecoveryPhrase] = useState('')
	const [isRecoveryMode, setIsRecoveryMode] = useState(false)
	const [showPassword, setShowPassword] = useState(false)
	const [loading, setLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault()

		setLoading(true)
		setError(null)

		try {
			if (isRecoveryMode) {
				await invoke('unlock_with_recovery_phrase', {
					phrase: recoveryPhrase,
				})
			} else {
				await invoke('initialize_security', {
					method: 'argon2',
					passphrase,
				})
			}
			onUnlock()
		} catch (err) {
			console.error('Failed to unlock:', err)
			setError(
				isRecoveryMode
					? t('welcome:recovery.verify.error')
					: 'Failed to unlock database. Incorrect password?'
			)
		} finally {
			setLoading(false)
		}
	}

	return (
		<div className='ambient-glow noise-overlay relative flex h-full flex-col items-center justify-center overflow-hidden'>
			{/* Background accent orbs */}
			<div
				className='pointer-events-none absolute top-1/4 left-1/3 h-64 w-64 rounded-full blur-[100px]'
				style={{ backgroundColor: `rgba(var(--accent-rgb), 0.04)` }}
			/>
			<div className='pointer-events-none absolute right-1/4 bottom-1/3 h-48 w-48 rounded-full bg-indigo-500/[0.03] blur-[80px]' />

			{/* Back button - top left */}
			<motion.div
				initial={{ opacity: 0, x: -8 }}
				animate={{ opacity: 1, x: 0 }}
				transition={{ duration: 0.3 }}
				className='absolute top-6 left-6 z-10'>
				<button
					type='button'
					onClick={onBack}
					className='group flex items-center gap-2 text-sm text-slate-500 transition-colors hover:text-slate-200'>
					<ArrowLeft className='h-4 w-4 transition-transform group-hover:-translate-x-0.5' />
					{t('common:actions.back')}
				</button>
			</motion.div>

			{/* Central unlock card */}
			<motion.div
				initial={{ opacity: 0, y: 30, scale: 0.96 }}
				animate={{ opacity: 1, y: 0, scale: 1 }}
				transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
				className='relative z-10 w-full max-w-sm px-6'>
				{/* Logo + Title */}
				<div className='mb-10 flex flex-col items-center text-center'>
					<motion.div
						initial={{ opacity: 0, scale: 0.8 }}
						animate={{ opacity: 1, scale: 1 }}
						transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
						className='animate-subtle-float mb-6'>
						<div className='relative flex h-20 w-20 items-center justify-center rounded-2xl bg-slate-800/80 shadow-xl ring-1 ring-white/[0.08]'>
							<img src={icon} alt='Postail' className='h-16 w-16' />
							<div
								className='animate-glow-breathe absolute -inset-3 -z-10 rounded-3xl blur-xl'
								style={{ backgroundColor: `rgba(var(--accent-rgb), 0.1)` }}
							/>
						</div>
					</motion.div>

					<motion.h1
						initial={{ opacity: 0, y: 10 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.15, duration: 0.4 }}
						className='gradient-text mb-2 text-2xl font-bold tracking-tight'>
						Unlock Database
					</motion.h1>
					<motion.p
						initial={{ opacity: 0, y: 8 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.25, duration: 0.4 }}
						className='text-sm text-slate-500'>
						Enter your password to decrypt your mail database.
					</motion.p>
				</div>

				{/* Form */}
				<motion.form
					onSubmit={handleSubmit}
					initial={{ opacity: 0, y: 16 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ delay: 0.3, duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
					className='space-y-5'>
					{/* Passphrase Input */}
					{/* Input Fields */}
					<motion.div
						key={isRecoveryMode ? 'recovery' : 'password'}
						initial={{ opacity: 0, x: isRecoveryMode ? 20 : -20 }}
						animate={{ opacity: 1, x: 0 }}
						exit={{ opacity: 0, x: isRecoveryMode ? -20 : 20 }}
						transition={{ duration: 0.3 }}>
						{isRecoveryMode ? (
							<div>
								<label
									htmlFor='recoveryPhrase'
									className='mb-2 block text-sm font-medium text-slate-300'>
									{t('welcome:recovery.unlock.label')}
								</label>
								<textarea
									id='recoveryPhrase'
									value={recoveryPhrase}
									onChange={(e) => setRecoveryPhrase(e.target.value)}
									className='min-h-[120px] w-full resize-none rounded-xl bg-slate-800/40 px-4 py-3 text-slate-100 placeholder-slate-600 ring-1 ring-white/[0.08] transition-all duration-200 focus:bg-slate-800/60 focus:outline-none'
									placeholder={t('welcome:recovery.unlock.placeholder')}
									required
									autoFocus
									style={
										{
											'--tw-ring-color': `rgba(var(--accent-rgb), 0.4)`,
										} as React.CSSProperties
									}
								/>
								<p className='mt-2 text-xs text-slate-500'>
									{t('welcome:recovery.unlock.hint')}
								</p>
							</div>
						) : (
							<div>
								<label
									htmlFor='passphrase'
									className='mb-2 block text-sm font-medium text-slate-300'>
									{t('security:argon2.passphrase.label')}
								</label>
								<div className='group relative'>
									<div className='pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3.5'>
										<Lock
											className='h-4 w-4 transition-colors duration-200'
											style={{ color: passphrase ? accentColor : undefined }}
										/>
									</div>
									<input
										id='passphrase'
										type={showPassword ? 'text' : 'password'}
										value={passphrase}
										onChange={(e) => setPassphrase(e.target.value)}
										className='w-full rounded-xl bg-slate-800/40 py-3.5 pr-12 pl-10 text-slate-100 placeholder-slate-600 ring-1 ring-white/[0.08] transition-all duration-200 focus:bg-slate-800/60 focus:outline-none'
										placeholder={t('security:argon2.passphrase.placeholder')}
										required
										autoFocus
									/>
									<button
										type='button'
										onClick={() => setShowPassword(!showPassword)}
										className='absolute top-1/2 right-3 -translate-y-1/2 rounded-lg p-1 text-slate-500 transition-colors hover:text-slate-300'>
										{showPassword ? (
											<EyeOff className='h-[18px] w-[18px]' />
										) : (
											<Eye className='h-[18px] w-[18px]' />
										)}
									</button>
								</div>
								<div className='mt-2 flex justify-end'>
									<button
										type='button'
										onClick={() => {
											setIsRecoveryMode(true)
											setError(null)
										}}
										className='text-xs text-slate-500 transition-colors hover:text-slate-300'>
										{t('welcome:recovery.unlock.forgotPassword')}
									</button>
								</div>
							</div>
						)}
					</motion.div>

					{/* Error Message */}
					{error && (
						<motion.div
							initial={{ opacity: 0, y: -8, scale: 0.98 }}
							animate={{ opacity: 1, y: 0, scale: 1 }}
							transition={{ duration: 0.2 }}
							className='flex items-center gap-2.5 rounded-xl bg-red-500/10 p-4 ring-1 ring-red-500/20'>
							<AlertTriangle className='h-4 w-4 shrink-0 text-red-400' />
							<p className='text-sm text-red-400'>{error}</p>
						</motion.div>
					)}

					{/* Submit Button */}
					<motion.button
						type='submit'
						disabled={loading || (isRecoveryMode ? !recoveryPhrase : !passphrase)}
						whileHover={
							!loading && (isRecoveryMode ? recoveryPhrase : passphrase)
								? { scale: 1.02 }
								: {}
						}
						whileTap={
							!loading && (isRecoveryMode ? recoveryPhrase : passphrase)
								? { scale: 0.97 }
								: {}
						}
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
							<>
								<Lock className='h-4 w-4' />
								{isRecoveryMode ? t('welcome:recovery.unlock.button') : 'Unlock'}
							</>
						)}
					</motion.button>

					{isRecoveryMode && (
						<button
							type='button'
							onClick={() => {
								setIsRecoveryMode(false)
								setError(null)
							}}
							className='w-full text-center text-xs text-slate-500 transition-colors hover:text-slate-300'>
							{t('welcome:recovery.unlock.usePassword')}
						</button>
					)}
				</motion.form>
			</motion.div>

			{/* Decorative bottom gradient line */}
			<motion.div
				initial={{ scaleX: 0, opacity: 0 }}
				animate={{ scaleX: 1, opacity: 1 }}
				transition={{ duration: 0.8, delay: 0.6, ease: [0.16, 1, 0.3, 1] }}
				className='absolute bottom-0 left-0 h-px w-full origin-center'
				style={{
					background: `linear-gradient(to right, transparent, rgba(var(--accent-rgb), 0.2), transparent)`,
				}}
			/>
		</div>
	)
}
