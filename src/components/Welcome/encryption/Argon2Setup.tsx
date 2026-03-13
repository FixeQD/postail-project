import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion } from 'framer-motion'
import { useSecurityTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
import { ArrowLeft, Lock, Eye, EyeOff, AlertTriangle, ShieldCheck } from 'lucide-react'

export const Argon2Setup = ({
	onBack,
	onComplete,
	onInitialize,
}: {
	onBack: () => void
	onComplete: () => void
	onInitialize?: (passphrase: string) => Promise<void>
}) => {
	const { t } = useSecurityTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const [passphrase, setPassphrase] = useState('')
	const [confirmPassphrase, setConfirmPassphrase] = useState('')
	const [showPassword, setShowPassword] = useState(false)
	const [showConfirmPassword, setShowConfirmPassword] = useState(false)
	const [loading, setLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault()

		if (passphrase.length < 8) {
			setError(t('security:argon2.errors.tooShort'))
			return
		}

		if (passphrase !== confirmPassphrase) {
			setError(t('security:argon2.errors.noMatch'))
			return
		}

		setLoading(true)
		setError(null)

		try {
			if (onInitialize) {
				await onInitialize(passphrase)
			} else {
				await invoke('initialize_security', {
					method: 'argon2',
					passphrase,
				})
			}
			onComplete()
		} catch (err) {
			console.error('Failed to initialize Argon2 security:', err)
			setError(t('security:argon2.errors.initFailed'))
		} finally {
			setLoading(false)
		}
	}

	const isValid = passphrase.length >= 8 && passphrase === confirmPassphrase

	const strengthLevel =
		passphrase.length >= 12 ? 'strong' : passphrase.length >= 8 ? 'medium' : 'weak'

	const strengthConfig = {
		strong: { width: 'w-full', color: 'bg-green-400', text: 'text-green-400' },
		medium: { width: 'w-2/3', color: 'bg-amber-400', text: 'text-amber-400' },
		weak: { width: 'w-1/3', color: 'bg-red-400', text: 'text-red-400' },
	}

	return (
		<div className='noise-overlay relative flex h-full flex-col'>
			{/* Header */}
			<motion.div
				initial={{ opacity: 0, y: -20, filter: 'blur(8px)' }}
				animate={{ opacity: 1, y: 0, filter: 'blur(0px)' }}
				transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
				className='relative border-b border-black/5 bg-white/10 px-4 py-6 shadow-sm backdrop-blur-[32px] dark:border-white/5 dark:bg-black/20'>
				<div
					className='pointer-events-none absolute inset-x-0 bottom-0 h-px'
					style={{
						background: `linear-gradient(to right, transparent, rgba(var(--accent-rgb), 0.1), transparent)`,
					}}
				/>

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
							<Lock className='h-5 w-5' style={{ color: accentColor }} />
						</div>
						<div>
							<h1 className='text-3xl font-bold tracking-tight text-slate-900 dark:text-slate-100'>
								{t('security:argon2.title')}
							</h1>
							<p className='mt-1 text-sm text-slate-500 dark:text-slate-400'>
								{t('security:argon2.subtitle')}
							</p>
						</div>
					</div>
				</div>
			</motion.div>

			{/* Form */}
			<div className='container mx-auto flex-1 px-4 py-8'>
				<motion.div
					initial={{ opacity: 0, y: 20 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.5, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
					className='mx-auto max-w-md'>
					<form onSubmit={handleSubmit} className='space-y-6'>
						{/* Passphrase Input */}
						<motion.div
							initial={{ opacity: 0, y: 12 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ delay: 0.15, duration: 0.4 }}>
							<label
								htmlFor='passphrase'
								className='mb-2 block text-sm font-medium text-slate-700 dark:text-slate-200'>
								{t('security:argon2.passphrase.label')}
							</label>
							<div className='group relative'>
								<input
									id='passphrase'
									type={showPassword ? 'text' : 'password'}
									value={passphrase}
									onChange={(e) => setPassphrase(e.target.value)}
									className='w-full rounded-xl bg-black/[0.04] px-4 py-3 pr-12 text-slate-900 placeholder-slate-400 ring-1 ring-black/[0.08] transition-all duration-200 focus:bg-black/[0.06] focus:outline-none dark:bg-slate-800/40 dark:text-slate-100 dark:placeholder-slate-600 dark:ring-white/[0.08] dark:focus:bg-slate-800/60'
									style={
										{
											'--tw-ring-color': `rgba(var(--accent-rgb), 0.4)`,
										} as React.CSSProperties
									}
									placeholder={t('security:argon2.passphrase.placeholder')}
									required
								/>
								<button
									type='button'
									onClick={() => setShowPassword(!showPassword)}
									className='absolute top-1/2 right-3 -translate-y-1/2 rounded-lg p-1 text-slate-400 transition-colors hover:text-slate-700 dark:text-slate-500 dark:hover:text-slate-300'>
									{showPassword ? (
										<EyeOff className='h-[18px] w-[18px]' />
									) : (
										<Eye className='h-[18px] w-[18px]' />
									)}
								</button>
							</div>
							<p className='mt-1.5 text-xs text-slate-400 dark:text-slate-500'>
								{t('security:argon2.passphrase.hint')}
							</p>
						</motion.div>

						{/* Confirm Passphrase Input */}
						<motion.div
							initial={{ opacity: 0, y: 12 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ delay: 0.25, duration: 0.4 }}>
							<label
								htmlFor='confirmPassphrase'
								className='mb-2 block text-sm font-medium text-slate-700 dark:text-slate-200'>
								{t('security:argon2.confirm.label')}
							</label>
							<div className='group relative'>
								<input
									id='confirmPassphrase'
									type={showConfirmPassword ? 'text' : 'password'}
									value={confirmPassphrase}
									onChange={(e) => setConfirmPassphrase(e.target.value)}
									className='w-full rounded-xl bg-black/[0.04] px-4 py-3 pr-12 text-slate-900 placeholder-slate-400 ring-1 ring-black/[0.08] transition-all duration-200 focus:bg-black/[0.06] focus:outline-none dark:bg-slate-800/40 dark:text-slate-100 dark:placeholder-slate-600 dark:ring-white/[0.08] dark:focus:bg-slate-800/60'
									style={
										{
											'--tw-ring-color': `rgba(var(--accent-rgb), 0.4)`,
										} as React.CSSProperties
									}
									placeholder={t('security:argon2.confirm.placeholder')}
									required
								/>
								<button
									type='button'
									onClick={() => setShowConfirmPassword(!showConfirmPassword)}
									className='absolute top-1/2 right-3 -translate-y-1/2 rounded-lg p-1 text-slate-400 transition-colors hover:text-slate-700 dark:text-slate-500 dark:hover:text-slate-300'>
									{showConfirmPassword ? (
										<EyeOff className='h-[18px] w-[18px]' />
									) : (
										<Eye className='h-[18px] w-[18px]' />
									)}
								</button>
							</div>
						</motion.div>

						{/* Password Strength Indicator */}
						{passphrase && (
							<motion.div
								initial={{ opacity: 0, height: 0 }}
								animate={{ opacity: 1, height: 'auto' }}
								transition={{ duration: 0.25 }}
								className='space-y-2.5 overflow-hidden'>
								<div className='flex items-center justify-between text-sm'>
									<span className='text-slate-400 dark:text-slate-500'>
										{t('security:argon2.strength.label')}
									</span>
									<span
										className={`text-xs font-semibold ${strengthConfig[strengthLevel].text}`}>
										{strengthLevel === 'strong'
											? t('security:argon2.strength.strong')
											: strengthLevel === 'medium'
												? t('security:argon2.strength.medium')
												: t('security:argon2.strength.weak')}
									</span>
								</div>
								<div className='h-1.5 overflow-hidden rounded-full bg-black/[0.08] dark:bg-slate-800'>
									<motion.div
										className={`h-full rounded-full ${strengthConfig[strengthLevel].color}`}
										initial={{ width: 0 }}
										animate={{
											width:
												strengthLevel === 'strong'
													? '100%'
													: strengthLevel === 'medium'
														? '66%'
														: '33%',
										}}
										transition={{
											duration: 0.4,
											ease: [0.16, 1, 0.3, 1],
										}}
									/>
								</div>
							</motion.div>
						)}

						{/* Error Message */}
						{error && (
							<motion.div
								initial={{ opacity: 0, y: -8 }}
								animate={{ opacity: 1, y: 0 }}
								transition={{ duration: 0.2 }}
								className='flex items-center gap-2.5 rounded-xl bg-red-500/10 p-4 ring-1 ring-red-500/20'>
								<AlertTriangle className='h-4 w-4 shrink-0 text-red-400' />
								<p className='text-sm text-red-400'>{error}</p>
							</motion.div>
						)}

						{/* Submit Button */}
						<motion.button
							type='submit'
							disabled={!isValid || loading}
							whileHover={isValid && !loading ? { scale: 1.02 } : {}}
							whileTap={isValid && !loading ? { scale: 0.97 } : {}}
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
									{t('security:argon2.creating')}
								</>
							) : (
								<>
									<Lock className='h-4 w-4' />
									{t('security:argon2.create')}
								</>
							)}
						</motion.button>
					</form>

					{/* Security Info */}
					<motion.div
						initial={{ opacity: 0, y: 12 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.4, duration: 0.5 }}
						className='mt-8 rounded-2xl bg-black/[0.03] p-5 ring-1 ring-black/[0.06] dark:bg-slate-800/30 dark:ring-white/[0.06]'>
						<div className='flex items-start gap-3'>
							<div className='mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-green-500/10 ring-1 ring-green-500/20'>
								<ShieldCheck className='h-4 w-4 text-green-400' />
							</div>
							<div>
								<h4 className='mb-1 text-sm font-semibold text-slate-700 dark:text-slate-200'>
									{t('security:argon2.info.title')}
								</h4>
								<p className='text-xs leading-relaxed text-slate-500 dark:text-slate-500'>
									{t('security:argon2.info.description')}
								</p>
							</div>
						</div>
					</motion.div>
				</motion.div>
			</div>
		</div>
	)
}
