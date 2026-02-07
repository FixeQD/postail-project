import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion } from 'framer-motion'
import { useSecurityTranslation } from '../../hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
import { TPMOption } from './TPMOption'
import { KeyringOption } from './KeyringOption'
import { Argon2Option } from './Argon2Option'
import { TPMInitDialog } from './TPMInitDialog'
import { ArrowLeft, Shield } from 'lucide-react'

interface SecurityOptions {
	tpm_available: boolean
	keyring_available: boolean
	argon2_available: boolean
}

export const EncryptionChoice = ({
	onChoiceSelected,
	onBack,
}: {
	onChoiceSelected: (method: string) => Promise<void>
	onBack: () => void
}) => {
	const { t } = useSecurityTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const [securityOptions, setSecurityOptions] = useState<SecurityOptions | null>(null)
	const [loading, setLoading] = useState(true)
	const [tpmDialogOpen, setTpmDialogOpen] = useState(false)
	const [loadingMethod, setLoadingMethod] = useState<string | null>(null)

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

	const handleTpmSuccess = useCallback(async () => {
		try {
			await onChoiceSelected('tpm')
		} finally {
			setLoadingMethod(null)
			setTpmDialogOpen(false)
		}
	}, [onChoiceSelected])

	const handleTpmSelect = () => {
		setLoadingMethod('tpm')
		setTpmDialogOpen(true)
	}

	if (loading) {
		return (
			<div className='flex h-full items-center justify-center'>
				<motion.div
					initial={{ opacity: 0 }}
					animate={{ opacity: 1 }}
					className='flex flex-col items-center gap-4'>
					<div className='relative h-12 w-12'>
						<div
							className='absolute inset-0 animate-spin rounded-full border-2 border-transparent'
							style={{ borderTopColor: accentColor }}
						/>
						<div
							className='absolute inset-1.5 animate-spin rounded-full border-2 border-transparent'
							style={{
								borderBottomColor: `rgba(var(--accent-rgb), 0.3)`,
								animationDirection: 'reverse',
								animationDuration: '1.5s',
							}}
						/>
					</div>
					<p className='text-sm text-slate-400'>{t('common:status.loading')}</p>
				</motion.div>
			</div>
		)
	}

	const cardVariants = {
		hidden: { opacity: 0, y: 24, scale: 0.96 },
		visible: (i: number) => ({
			opacity: 1,
			y: 0,
			scale: 1,
			transition: {
				delay: 0.15 + i * 0.1,
				duration: 0.5,
				ease: [0.16, 1, 0.3, 1] as [number, number, number, number],
			},
		}),
	}

	return (
		<>
			<div className='noise-overlay relative flex h-full flex-col'>
				{/* Header */}
				<motion.div
					initial={{ opacity: 0, y: -10 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
					className='relative border-b border-white/[0.06] bg-slate-900/40 px-4 py-6 backdrop-blur-lg'>
					{/* Top highlight line */}
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
							className='group mb-6 flex items-center gap-2 text-sm text-slate-400 transition-colors hover:text-slate-100'>
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
								<Shield className='h-5 w-5' style={{ color: accentColor }} />
							</div>
							<div>
								<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
									{t('security:title')}
								</h1>
								<p className='mt-1 text-sm text-slate-400'>
									{t('security:subtitle')}
								</p>
							</div>
						</div>
					</div>
				</motion.div>

				{/* Options */}
				<div className='container mx-auto flex-1 px-4 py-8'>
					<div className='mx-auto max-w-4xl'>
						<div className='grid gap-5 md:grid-cols-1 lg:grid-cols-3'>
							<motion.div
								custom={0}
								initial='hidden'
								animate='visible'
								variants={cardVariants}
								className='hover-lift'>
								<TPMOption
									available={securityOptions?.tpm_available ?? false}
									onSelect={handleTpmSelect}
									disabled={loadingMethod !== null}
									loading={loadingMethod === 'tpm'}
								/>
							</motion.div>
							<motion.div
								custom={1}
								initial='hidden'
								animate='visible'
								variants={cardVariants}
								className='hover-lift'>
								<KeyringOption
									available={securityOptions?.keyring_available ?? false}
									onSelect={async () => {
										setLoadingMethod('keyring')
										try {
											await onChoiceSelected('keyring')
										} finally {
											setLoadingMethod(null)
										}
									}}
									disabled={loadingMethod !== null}
									loading={loadingMethod === 'keyring'}
								/>
							</motion.div>
							<motion.div
								custom={2}
								initial='hidden'
								animate='visible'
								variants={cardVariants}
								className='hover-lift'>
								<Argon2Option
									available={securityOptions?.argon2_available ?? true}
									onSelect={async () => {
										setLoadingMethod('argon2')
										try {
											await onChoiceSelected('argon2')
										} finally {
											setLoadingMethod(null)
										}
									}}
									disabled={loadingMethod !== null}
									loading={loadingMethod === 'argon2'}
								/>
							</motion.div>
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
