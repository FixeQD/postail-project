import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { Timer, FileKey, ClipboardX, ChevronDown } from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'sonner'

const TIMEOUT_OPTIONS = [
	{ value: 1, label: '1min' },
	{ value: 5, label: '5min' },
	{ value: 10, label: '10min' },
	{ value: 15, label: '15min' },
	{ value: 30, label: '30min' },
	{ value: 60, label: '60min' },
]

export function SecuritySettings() {
	const { t } = useSettingsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const [autoLockEnabled, setAutoLockEnabled] = useState(false)
	const [timeout, setTimeout] = useState(5)
	const [showTimeoutDropdown, setShowTimeoutDropdown] = useState(false)
	const [showPinSetup, setShowPinSetup] = useState(false)
	const [showChangePin, setShowChangePin] = useState(false)
	const [pin, setPin] = useState('')
	const [confirmPin, setConfirmPin] = useState('')
	const [useEncryptionPassword, setUseEncryptionPassword] = useState(false)
	const [usesPassphraseMethod, setUsesPassphraseMethod] = useState(false)
	const [isLockConfigured, setIsLockConfigured] = useState(false)

	useEffect(() => {
		const loadSettings = async () => {
			const currentTimeout = await invoke<number>('get_auto_lock_timeout')
			const useEncryption = await invoke<boolean>('is_lock_using_encryption_password')
			const securityMethod = await invoke<string | null>('get_security_method')
			const lockConfigured = await invoke<boolean>('is_lock_configured')
			setTimeout(currentTimeout)
			setUseEncryptionPassword(useEncryption)
			setAutoLockEnabled(currentTimeout > 0 && lockConfigured)
			setUsesPassphraseMethod(securityMethod === 'argon2')
			setIsLockConfigured(lockConfigured)
		}
		loadSettings()
	}, [])

	const handleAutoLockToggle = async (enabled: boolean) => {
		if (!enabled) {
			// Disable auto-lock
			await invoke('set_auto_lock_timeout', { minutes: 0 })
			setAutoLockEnabled(false)
			toast.success('Auto-lock disabled')
			return
		}

		if (usesPassphraseMethod) {
			// For argon2, enable immediately without PIN
			await invoke('use_encryption_password_for_lock')
			await invoke('set_auto_lock_timeout', { minutes: 5 })
			setTimeout(5)
			setAutoLockEnabled(true)
			setIsLockConfigured(true)
			toast.success('Auto-lock enabled with encryption password')
		} else {
			// For keyring/TPM, show PIN setup
			setAutoLockEnabled(true)
			setShowPinSetup(true)
		}
	}

	const handleTimeoutChange = async (minutes: number) => {
		setTimeout(minutes)
		await invoke('set_auto_lock_timeout', { minutes })
		setShowTimeoutDropdown(false)
	}

	const handlePinSetup = async () => {
		if (useEncryptionPassword) {
			await invoke('use_encryption_password_for_lock')
			await invoke('set_auto_lock_timeout', { minutes: 5 })
			setShowPinSetup(false)
			setAutoLockEnabled(true)
			setIsLockConfigured(true)
			setTimeout(5)
			toast.success('Auto-lock enabled with encryption password')
			return
		}

		if (pin.length < 4) {
			toast.error('PIN must be at least 4 characters')
			return
		}

		if (pin !== confirmPin) {
			toast.error('PINs do not match')
			return
		}

		await invoke('set_auto_lock_pin', { pin })
		await invoke('set_auto_lock_timeout', { minutes: 5 })
		setShowPinSetup(false)
		setAutoLockEnabled(true)
		setIsLockConfigured(true)
		setPin('')
		setConfirmPin('')
		setTimeout(5)
		toast.success('Auto-lock enabled')
	}

	const handleChangePin = async () => {
		if (useEncryptionPassword) {
			await invoke('use_encryption_password_for_lock')
			setShowChangePin(false)
			setUseEncryptionPassword(true)
			setIsLockConfigured(true)
			toast.success('Changed to use encryption password')
			return
		}

		if (pin.length < 4) {
			toast.error('PIN must be at least 4 characters')
			return
		}

		if (pin !== confirmPin) {
			toast.error('PINs do not match')
			return
		}

		await invoke('set_auto_lock_pin', { pin })
		setShowChangePin(false)
		setIsLockConfigured(true)
		setPin('')
		setConfirmPin('')
		toast.success('PIN changed successfully')
	}

	return (
		<div className='mx-auto flex h-full w-full max-w-3xl flex-col space-y-8 p-8'>
			<motion.div
				{...(animationsEnabled
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
					{t('settings:security.title')}
				</h1>
				<p className='mt-1 text-slate-400'>{t('settings:security.subtitle')}</p>
			</motion.div>

			<div className='space-y-6'>
				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:security.session.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Timer}
							label={t('settings:security.session.autoLock.label')}
							description={t('settings:security.session.autoLock.description')}
							value={autoLockEnabled}
							onChange={handleAutoLockToggle}
						/>

						{autoLockEnabled && !isLockConfigured && !usesPassphraseMethod && (
							<motion.div
								initial={{ opacity: 0, height: 0 }}
								animate={{ opacity: 1, height: 'auto' }}
								className='mt-2 ml-11 rounded-lg border border-amber-500/30 bg-amber-500/10 p-3'>
								<p className='text-sm text-amber-200'>
									Auto-lock requires a PIN to be set. Please configure your PIN
									below.
								</p>
							</motion.div>
						)}

						{autoLockEnabled && isLockConfigured && !showPinSetup && !showChangePin && (
							<motion.div
								initial={{ opacity: 0, height: 0 }}
								animate={{ opacity: 1, height: 'auto' }}
								className='mt-2 ml-11 flex flex-wrap items-center gap-2'>
								<div className='relative'>
									<button
										type='button'
										onClick={() => setShowTimeoutDropdown(!showTimeoutDropdown)}
										className='flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-800/50 px-4 py-2 text-sm text-slate-200 transition-colors hover:bg-slate-700/50'>
										<span>
											{t('settings:security.session.autoLock.timeout.label')}:{' '}
											{TIMEOUT_OPTIONS.find((o) => o.value === timeout)
												? t(
														`settings:security.session.autoLock.timeout.options.${TIMEOUT_OPTIONS.find((o) => o.value === timeout)?.label}`
													)
												: `${timeout} minutes`}
										</span>
										<ChevronDown className='h-4 w-4' />
									</button>

									{showTimeoutDropdown && (
										<motion.div
											initial={{ opacity: 0, y: -10 }}
											animate={{ opacity: 1, y: 0 }}
											className='absolute top-full z-10 mt-1 w-48 rounded-lg border border-slate-700 bg-slate-800 py-1 shadow-xl'>
											{TIMEOUT_OPTIONS.map((option) => (
												<button
													type='button'
													key={option.value}
													onClick={() =>
														handleTimeoutChange(option.value)
													}
													className={`w-full px-4 py-2 text-left text-sm transition-colors hover:bg-slate-700 ${
														timeout === option.value
															? 'text-slate-100'
															: 'text-slate-400'
													}`}>
													{t(
														`settings:security.session.autoLock.timeout.options.${option.label}`
													)}
												</button>
											))}
										</motion.div>
									)}
								</div>
								{!usesPassphraseMethod && (
									<button
										type='button'
										onClick={() => {
											setShowChangePin(true)
											setPin('')
											setConfirmPin('')
										}}
										className='flex h-9 items-center justify-center rounded-lg border border-slate-700 bg-slate-900/50 px-4 text-sm text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-300'>
										Change PIN
									</button>
								)}
							</motion.div>
						)}

						{showPinSetup && (
							<motion.div
								initial={{ opacity: 0, height: 0 }}
								animate={{ opacity: 1, height: 'auto' }}
								className='mt-4 ml-11 rounded-xl border border-slate-700 bg-slate-800/50 p-4'>
								<h3 className='mb-2 text-sm font-medium text-slate-200'>
									{t('settings:security.session.autoLock.setupPin.title')}
								</h3>
								<p className='mb-4 text-xs text-slate-400'>
									{t('settings:security.session.autoLock.setupPin.description')}
								</p>

								<div className='space-y-3'>
									{!useEncryptionPassword && (
										<>
											<input
												type='password'
												value={pin}
												onChange={(e) => setPin(e.target.value)}
												placeholder={t(
													'settings:security.session.autoLock.setupPin.pinPlaceholder'
												)}
												className='w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-white outline-none focus:border-slate-500'
											/>
											<input
												type='password'
												value={confirmPin}
												onChange={(e) => setConfirmPin(e.target.value)}
												placeholder={t(
													'settings:security.session.autoLock.setupPin.confirmPlaceholder'
												)}
												className='w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-white outline-none focus:border-slate-500'
											/>
										</>
									)}

									{usesPassphraseMethod && (
										<label className='flex items-center gap-2 text-sm text-slate-300'>
											<input
												type='checkbox'
												checked={useEncryptionPassword}
												onChange={(e) =>
													setUseEncryptionPassword(e.target.checked)
												}
												className='rounded border-slate-600'
											/>
											{t(
												'settings:security.session.autoLock.setupPin.usePassword'
											)}
										</label>
									)}

									<div className='flex gap-2 pt-2'>
										<button
											type='button'
											onClick={handlePinSetup}
											className='flex-1 rounded-lg bg-slate-700 py-2 text-sm font-medium text-white transition-colors hover:bg-slate-600'>
											Enable
										</button>
										<button
											type='button'
											onClick={() => {
												setShowPinSetup(false)
												setAutoLockEnabled(false)
											}}
											className='flex-1 rounded-lg border border-slate-700 py-2 text-sm text-slate-300 transition-colors hover:bg-slate-800'>
											Cancel
										</button>
									</div>
								</div>
							</motion.div>
						)}

						{showChangePin && (
							<motion.div
								initial={{ opacity: 0, height: 0 }}
								animate={{ opacity: 1, height: 'auto' }}
								className='mt-4 ml-11 rounded-xl border border-slate-700 bg-slate-800/50 p-4'>
								<h3 className='mb-2 text-sm font-medium text-slate-200'>
									Change PIN
								</h3>
								<p className='mb-4 text-xs text-slate-400'>
									{usesPassphraseMethod
										? 'Set a new PIN or switch to encryption password'
										: 'Set a new PIN'}
								</p>

								<div className='space-y-3'>
									{!useEncryptionPassword && (
										<>
											<input
												type='password'
												value={pin}
												onChange={(e) => setPin(e.target.value)}
												placeholder='New PIN'
												className='w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-white outline-none focus:border-slate-500'
											/>
											<input
												type='password'
												value={confirmPin}
												onChange={(e) => setConfirmPin(e.target.value)}
												placeholder='Confirm new PIN'
												className='w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-white outline-none focus:border-slate-500'
											/>
										</>
									)}

									{usesPassphraseMethod && (
										<label className='flex items-center gap-2 text-sm text-slate-300'>
											<input
												type='checkbox'
												checked={useEncryptionPassword}
												onChange={(e) =>
													setUseEncryptionPassword(e.target.checked)
												}
												className='rounded border-slate-600'
											/>
											Use encryption password instead
										</label>
									)}

									<div className='flex gap-2 pt-2'>
										<button
											type='button'
											onClick={handleChangePin}
											className='flex-1 rounded-lg bg-slate-700 py-2 text-sm font-medium text-white transition-colors hover:bg-slate-600'>
											Save
										</button>
										<button
											type='button'
											onClick={() => {
												setShowChangePin(false)
												setPin('')
												setConfirmPin('')
											}}
											className='flex-1 rounded-lg border border-slate-700 py-2 text-sm text-slate-300 transition-colors hover:bg-slate-800'>
											Cancel
										</button>
									</div>
								</div>
							</motion.div>
						)}
					</div>
				</section>

				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:security.data.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={FileKey}
							label={t('settings:security.data.encryptAttachments.label')}
							description={t('settings:security.data.encryptAttachments.description')}
							value={false}
							onChange={() => {}}
						/>
						<ToggleSetting
							icon={ClipboardX}
							label={t('settings:security.data.clearClipboard.label')}
							description={t('settings:security.data.clearClipboard.description')}
							value={false}
							onChange={() => {}}
						/>
					</div>
				</section>
			</div>
		</div>
	)
}
