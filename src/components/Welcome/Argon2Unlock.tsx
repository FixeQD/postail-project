import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useSecurityTranslation } from '../../hooks/useTypedTranslation'
import { ArrowLeft, Lock, Eye, EyeOff, AlertTriangle } from 'lucide-react'

export const Argon2Unlock = ({
	onBack,
	onUnlock,
}: {
	onBack: () => void
	onUnlock: () => void
}) => {
	const { t } = useSecurityTranslation()
	const [passphrase, setPassphrase] = useState('')
	const [showPassword, setShowPassword] = useState(false)
	const [loading, setLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault()

		setLoading(true)
		setError(null)

		try {
			await invoke('initialize_security', {
				method: 'argon2',
				passphrase,
			})
			onUnlock()
		} catch (err) {
			console.error('Failed to unlock with Argon2:', err)
			setError('Failed to unlock database. Incorrect password?')
		} finally {
			setLoading(false)
		}
	}

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
						Unlock Database
					</h1>
					<p className='mt-2 text-slate-400'>
						Enter your password to decrypt your mail database.
					</p>
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
									autoFocus
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
						</div>

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
							disabled={loading || !passphrase}
							className='flex w-full items-center justify-center gap-2 rounded-lg bg-orange-600 px-6 py-3 font-medium text-white transition-colors hover:bg-orange-500 disabled:cursor-not-allowed disabled:opacity-50'>
							{loading ? (
								<>
									<div className='h-5 w-5 animate-spin rounded-full border-2 border-white border-t-transparent' />
									Unlocking...
								</>
							) : (
								<>
									<Lock className='h-5 w-5' />
									Unlock
								</>
							)}
						</button>
					</form>
				</div>
			</div>
		</div>
	)
}
