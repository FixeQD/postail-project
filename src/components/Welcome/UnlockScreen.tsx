import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Key, Lock, HardDrive, Cpu, AlertTriangle } from 'lucide-react'
import { motion } from 'framer-motion'
import { useSecurityTranslation } from '../../hooks/useTypedTranslation'

interface SecurityOptions {
	tpm_available: boolean
	keyring_available: boolean
	argon2_available: boolean
}

export const UnlockScreen = ({
	onChoiceSelected,
	onSuccess,
}: {
	onChoiceSelected: (method: string) => void
	onSuccess: () => void
}) => {
	const { t } = useSecurityTranslation()
	const [options, setOptions] = useState<SecurityOptions | null>(null)
	const [loading, setLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)

	useEffect(() => {
		invoke<SecurityOptions>('check_security_options').then(setOptions)
	}, [])

	const handleAutoUnlock = async (method: string) => {
		setLoading(true)
		setError(null)
		try {
			await invoke('initialize_security', { method })
			onSuccess()
		} catch (e: any) {
			console.error(e)
			setError(`Failed to unlock with ${method}: ${e}`)
		} finally {
			setLoading(false)
		}
	}

	if (!options) return null

	return (
		<div className='flex h-full flex-col items-center justify-center p-8'>
			<div className='mb-12 text-center'>
				<motion.div
					initial={{ opacity: 0, y: 20 }}
					animate={{ opacity: 1, y: 0 }}
					className='mb-6 flex justify-center'>
					<div className='rounded-2xl bg-orange-500/10 p-4 ring-1 ring-orange-500/20'>
						<Lock className='h-12 w-12 text-orange-500' />
					</div>
				</motion.div>
				<motion.h1
					initial={{ opacity: 0, y: 20 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ delay: 0.1 }}
					className='text-4xl font-bold tracking-tight text-slate-100'>
					Unlock Database
				</motion.h1>
				<motion.p
					initial={{ opacity: 0, y: 20 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ delay: 0.2 }}
					className='mt-4 max-w-md text-lg text-slate-400'>
					The database is encrypted. Please select how you want to unlock it.
				</motion.p>
			</div>

			{error && (
				<div className='mb-6 flex max-w-md items-center gap-2 rounded-lg bg-red-900/50 p-4 text-red-400 ring-1 ring-red-400/20'>
					<AlertTriangle className='h-5 w-5 shrink-0' />
					<p className='text-sm'>{error}</p>
				</div>
			)}

			<div className='grid w-full max-w-4xl gap-6 md:grid-cols-2 lg:grid-cols-3'>
				{options.tpm_available && (
					<UnlockOption
						title={t('security:options.tpm.title')}
						description={t('security:options.tpm.description')}
						icon={<Cpu className='h-6 w-6' />}
						onClick={() => handleAutoUnlock('tpm')}
						isRecommended
						loading={loading}
					/>
				)}

				{options.keyring_available && (
					<UnlockOption
						title={t('security:options.keyring.title')}
						description={t('security:options.keyring.description')}
						icon={<Key className='h-6 w-6' />}
						onClick={() => handleAutoUnlock('keyring')}
						loading={loading}
					/>
				)}

				<UnlockOption
					title={t('security:options.argon2.title')}
					description={t('security:options.argon2.description')}
					icon={<HardDrive className='h-6 w-6' />}
					onClick={() => onChoiceSelected('argon2')}
					loading={loading}
				/>
			</div>
		</div>
	)
}

const UnlockOption = ({
	title,
	description,
	icon,
	onClick,
	isRecommended,
	loading,
}: {
	title: string
	description: string
	icon: React.ReactNode
	onClick: () => void
	isRecommended?: boolean
	loading: boolean
}) => (
	<motion.button
		initial={{ opacity: 0, scale: 0.95 }}
		animate={{ opacity: 1, scale: 1 }}
		whileHover={{ scale: 1.02 }}
		whileTap={{ scale: 0.98 }}
		disabled={loading}
		onClick={onClick}
		className='group relative flex flex-col gap-4 rounded-xl border border-slate-800 bg-slate-900/50 p-6 text-left transition-colors hover:border-orange-500/50 hover:bg-slate-900 disabled:opacity-50'>
		{isRecommended && (
			<span className='absolute -top-3 left-6 rounded-full bg-orange-500 px-3 py-1 text-xs font-medium text-white shadow-lg shadow-orange-500/20'>
				Recommended
			</span>
		)}
		<div className='flex h-12 w-12 items-center justify-center rounded-lg bg-slate-800 text-slate-400 ring-1 ring-slate-700 transition-colors group-hover:bg-orange-500 group-hover:text-white group-hover:ring-orange-600'>
			{icon}
		</div>
		<div>
			<h3 className='font-semibold text-slate-100'>{title}</h3>
			<p className='mt-2 text-sm text-slate-400'>{description}</p>
		</div>
	</motion.button>
)
