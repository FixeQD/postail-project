import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useSecurityTranslation } from '../../hooks/useTypedTranslation'
import { TPMOption } from './TPMOption'
import { KeyringOption } from './KeyringOption'
import { Argon2Option } from './Argon2Option'
import { TPMInitDialog } from './TPMInitDialog'
import { ArrowLeft } from 'lucide-react'

interface SecurityOptions {
	tpm_available: boolean
	keyring_available: boolean
	argon2_available: boolean
}

export const EncryptionChoice = ({
	onChoiceSelected,
	onBack,
}: {
	onChoiceSelected: (method: string) => void
	onBack: () => void
}) => {
	const { t } = useSecurityTranslation()
	const [securityOptions, setSecurityOptions] = useState<SecurityOptions | null>(null)
	const [loading, setLoading] = useState(true)
	const [tpmDialogOpen, setTpmDialogOpen] = useState(false)

	const checkOptions = useCallback(async () => {
		try {
			const options = await invoke<SecurityOptions>('check_security_options')
			setSecurityOptions(options)
		} catch (error) {
			console.error('Failed to check security options:', error)
			setSecurityOptions({
				tpm_available: false,
				keyring_available: false,
				argon2_available: true,
			})
		} finally {
			setLoading(false)
		}
	}, [])

	useEffect(() => {
		checkOptions()
	}, [checkOptions])

	const handleTpmSuccess = useCallback(() => {
		setTpmDialogOpen(false)
		onChoiceSelected('tpm')
	}, [onChoiceSelected])

	const handleTpmSelect = () => {
		setTpmDialogOpen(true)
	}

	if (loading) {
		return (
			<div className='flex h-full items-center justify-center'>
				<div className='text-center'>
					<div className='mb-4 h-10 w-10 animate-spin rounded-full border-4 border-slate-700 border-t-slate-300'></div>
					<p className='text-slate-400'>{t('common:status.loading')}</p>
				</div>
			</div>
		)
	}

	return (
		<>
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
							{t('security:title')}
						</h1>
						<p className='mt-2 text-slate-400'>{t('security:subtitle')}</p>
					</div>
				</div>

				{/* Options */}
				<div className='container mx-auto flex-1 px-4 py-8'>
					<div className='mx-auto max-w-4xl'>
						<div className='grid gap-6 md:grid-cols-1 lg:grid-cols-3'>
							<TPMOption
								available={securityOptions?.tpm_available ?? false}
								onSelect={handleTpmSelect}
							/>
							<KeyringOption
								available={securityOptions?.keyring_available ?? false}
								onSelect={() => onChoiceSelected('keyring')}
							/>
							<Argon2Option
								available={securityOptions?.argon2_available ?? true}
								onSelect={() => onChoiceSelected('argon2')}
							/>
						</div>
					</div>
				</div>
			</div>

			<TPMInitDialog
				open={tpmDialogOpen}
				onClose={() => setTpmDialogOpen(false)}
				onSuccess={handleTpmSuccess}
			/>
		</>
	)
}
