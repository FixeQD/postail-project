import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useSecurityTranslation } from '../../hooks/useTypedTranslation'
import { ArrowLeft, Lock, Eye, EyeOff, Check, AlertTriangle } from 'lucide-react'

export const Argon2Setup = ({
	onBack,
	onComplete,
}: {
	onBack: () => void
	onComplete: () => void
}) => {
	const { t } = useSecurityTranslation()
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
			await invoke('initialize_security', {
				method: 'argon2',
				passphrase,
			})
			onComplete()
		} catch (err) {
			console.error('Failed to initialize Argon2 security:', err)
			setError(t('security:argon2.errors.initFailed'))
		} finally {
			setLoading(false)
		}
	}

	const isValid = passphrase.length >= 8 && passphrase === confirmPassphrase

	return (
		<div className='flex h-full flex-col'>
			{/* Header */}
			<div className='border-b border-slate-800 bg-slate-900/50 px-4 py-6 backdrop-blur-lg'>
				<div className='container mx-auto'>
					<button
						type='button'
						onClick={onBack}
						className='mb-6 flex items-center gap-2 text-sm text-slate-300 transition-colors hover:text-slate-100'>
						<ArrowLeft className='h-4 w-4' />
						{t('common:actions.back')}
					</button>
					<h1 className='text-4xl font-bold tracking-tight text-slate-100'>
						{t('security:argon2.title')}
					</h1>
					<p className='mt-2 text-slate-400'>{t('security:argon2.subtitle')}</p>
				</div>
			</div>

			{/* Form */}
			<div className='container mx-auto flex-1 px-4 py-8'>
				<div className='mx-auto max-w-md'>
					<form onSubmit={handleSubmit} className='space-y-6'>
						{/* Passphrase Input */}
						<div>
							<label
								htmlFor='passphrase'
								className='mb-2 block text-sm font-medium text-slate-100'>
								{t('security:argon2.passphrase.label')}
							</label>
							<div className='relative'>
								<input
									id='passphrase'
									type={showPassword ? 'text' : 'password'}
									value={passphrase}
									onChange={(e) => setPassphrase(e.target.value)}
									className='w-full rounded-lg bg-slate-800/50 px-4 py-3 pr-12 text-slate-100 placeholder-slate-500 ring-1 ring-slate-700 focus:ring-orange-400 focus:outline-none'
									placeholder={t('security:argon2.passphrase.placeholder')}
									required
								/>
								<button
									type='button'
									onClick={() => setShowPassword(!showPassword)}
									className='absolute top-1/2 right-3 -translate-y-1/2 text-slate-400 hover:text-slate-300'>
									{showPassword ? (
										<EyeOff className='h-5 w-5' />
									) : (
										<Eye className='h-5 w-5' />
									)}
								</button>
							</div>
							<p className='mt-1 text-xs text-slate-400'>
								{t('security:argon2.passphrase.hint')}
							</p>
						</div>

						{/* Confirm Passphrase Input */}
						<div>
							<label
								htmlFor='confirmPassphrase'
								className='mb-2 block text-sm font-medium text-slate-100'>
								{t('security:argon2.confirm.label')}
							</label>
							<div className='relative'>
								<input
									id='confirmPassphrase'
									type={showConfirmPassword ? 'text' : 'password'}
									value={confirmPassphrase}
									onChange={(e) => setConfirmPassphrase(e.target.value)}
									className='w-full rounded-lg bg-slate-800/50 px-4 py-3 pr-12 text-slate-100 placeholder-slate-500 ring-1 ring-slate-700 focus:ring-orange-400 focus:outline-none'
									placeholder={t('security:argon2.confirm.placeholder')}
									required
								/>
								<button
									type='button'
									onClick={() => setShowConfirmPassword(!showConfirmPassword)}
									className='absolute top-1/2 right-3 -translate-y-1/2 text-slate-400 hover:text-slate-300'>
									{showConfirmPassword ? (
										<EyeOff className='h-5 w-5' />
									) : (
										<Eye className='h-5 w-5' />
									)}
								</button>
							</div>
						</div>

						{/* Password Strength Indicator */}
						{passphrase && (
							<div className='space-y-2'>
								<div className='flex items-center justify-between text-sm'>
									<span className='text-slate-400'>
										{t('security:argon2.strength.label')}
									</span>
									<span
										className={`font-medium ${
											passphrase.length >= 12
												? 'text-green-400'
												: passphrase.length >= 8
													? 'text-yellow-400'
													: 'text-red-400'
										}`}>
										{passphrase.length >= 12
											? t('security:argon2.strength.strong')
											: passphrase.length >= 8
												? t('security:argon2.strength.medium')
												: t('security:argon2.strength.weak')}
									</span>
								</div>
								<div className='h-2 rounded-full bg-slate-700'>
									<div
										className={`h-full rounded-full transition-all ${
											passphrase.length >= 12
												? 'w-full bg-green-400'
												: passphrase.length >= 8
													? 'w-2/3 bg-yellow-400'
													: 'w-1/3 bg-red-400'
										}`}
									/>
								</div>
							</div>
						)}

						{/* Error Message */}
						{error && (
							<div className='flex items-center gap-2 rounded-lg bg-red-900/50 p-4 text-red-400 ring-1 ring-red-400/20'>
								<AlertTriangle className='h-5 w-5 shrink-0' />
								<p className='text-sm'>{error}</p>
							</div>
						)}

						{/* Submit Button */}
						<button
							type='submit'
							disabled={!isValid || loading}
							className='flex w-full items-center justify-center gap-2 rounded-lg bg-orange-600 px-6 py-3 font-medium text-white transition-colors hover:bg-orange-500 disabled:cursor-not-allowed disabled:opacity-50'>
							{loading ? (
								<>
									<div className='h-5 w-5 animate-spin rounded-full border-2 border-white border-t-transparent' />
									{t('security:argon2.creating')}
								</>
							) : (
								<>
									<Lock className='h-5 w-5' />
									{t('security:argon2.create')}
								</>
							)}
						</button>
					</form>

					{/* Security Info */}
					<div className='mt-8 rounded-lg bg-slate-800/30 p-4 ring-1 ring-slate-700/50'>
						<div className='flex items-start gap-3'>
							<Check className='mt-0.5 h-5 w-5 text-green-400' />
							<div>
								<h4 className='mb-1 font-medium text-slate-100'>
									{t('security:argon2.info.title')}
								</h4>
								<p className='text-sm text-slate-400'>
									{t('security:argon2.info.description')}
								</p>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	)
}
