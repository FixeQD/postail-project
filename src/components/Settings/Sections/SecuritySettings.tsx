import { useState, useEffect, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Timer, FileKey, ClipboardX, ChevronDown, Check } from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useAsyncState } from '@/hooks/useAsyncState'
import { useSettingsStore } from '@/stores/settingsStore'
import { useThemeStore } from '@/stores/themeStore'
import { invoke } from '@tauri-apps/api/core'
import { toast } from '../../ui/custom/Toaster'

const CLIPBOARD_DELAY_OPTIONS = [
	{ value: 0, labelKey: 'disabled' },
	{ value: 30, labelKey: '30' },
	{ value: 60, labelKey: '60' },
] as const

function SectionTitle({ children }: { children: React.ReactNode }) {
	return (
		<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
			{children}
		</h2>
	)
}

interface InlineSelectProps {
	value: number
	options: { value: number; label: string }[]
	onChange: (v: number) => void
	accentColor: string
}

function InlineSelect({ value, options, onChange, accentColor }: InlineSelectProps) {
	return (
		<div className='flex flex-wrap justify-end gap-1'>
			{options.map((opt) => (
				<button
					key={opt.value}
					type='button'
					onClick={() => onChange(opt.value)}
					className='rounded-lg px-3 py-1.5 text-xs font-semibold transition-all duration-150'
					style={
						value === opt.value
							? { backgroundColor: accentColor, color: '#fff' }
							: {
									color: 'var(--text-secondary)',
									boxShadow: 'inset 0 0 0 1px var(--border-subtle)',
								}
					}>
					{opt.label}
				</button>
			))}
		</div>
	)
}

const TIMEOUT_OPTIONS = [
	{ value: 0, label: 'disabled' },
	{ value: 1, label: '1min' },
	{ value: 5, label: '5min' },
	{ value: 10, label: '10min' },
	{ value: 15, label: '15min' },
	{ value: 30, label: '30min' },
	{ value: 60, label: '60min' },
]

const DEFAULT_LOCK_TIMEOUT = 5

const INPUT_CLASS =
	'w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-hover)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none transition-colors focus:border-[var(--accent-color)] focus:ring-1 focus:ring-[var(--accent-color)]'

type PinFormMode = 'setup' | 'change' | null

interface PinFormProps {
	mode: 'setup' | 'change'
	usesPassphrase: boolean
	isLoading: boolean
	onSubmit: (pin: string | null, useEncryption: boolean) => Promise<void>
	onCancel: () => void
	t: (key: string) => string
}

function PinForm({ mode, usesPassphrase, isLoading, onSubmit, onCancel, t }: PinFormProps) {
	const [pin, setPin] = useState('')
	const [confirmPin, setConfirmPin] = useState('')
	const [useEncryption, setUseEncryption] = useState(false)
	const isSetup = mode === 'setup'

	const handleSubmit = async () => {
		if (useEncryption) {
			await onSubmit(null, true)
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
		await onSubmit(pin, false)
	}

	const title = isSetup ? t('settings:security.session.autoLock.setupPin.title') : 'Change PIN'

	const description = isSetup
		? t('settings:security.session.autoLock.setupPin.description')
		: usesPassphrase
			? 'Set a new PIN or switch to your encryption password'
			: 'Set a new PIN for the lock screen'

	return (
		<motion.div
			initial={{ opacity: 0, height: 0 }}
			animate={{ opacity: 1, height: 'auto' }}
			exit={{ opacity: 0, height: 0 }}
			transition={{ duration: 0.2, ease: 'easeOut' }}
			className='mt-4 ml-11 overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-4'>
			<h3 className='mb-1 text-sm font-semibold text-[var(--text-primary)]'>{title}</h3>
			<p className='mb-4 text-xs leading-relaxed text-[var(--text-secondary)]'>
				{description}
			</p>

			<div className='space-y-3'>
				{!useEncryption && (
					<>
						<input
							type='password'
							value={pin}
							onChange={(e) => setPin(e.target.value)}
							placeholder={
								isSetup
									? t(
											'settings:security.session.autoLock.setupPin.pinPlaceholder'
										)
									: t(
											'settings:security.session.autoLock.setupPin.newPinPlaceholder'
										)
							}
							className={INPUT_CLASS}
						/>
						<input
							type='password'
							value={confirmPin}
							onChange={(e) => setConfirmPin(e.target.value)}
							onKeyDown={(e) => e.key === 'Enter' && !isLoading && handleSubmit()}
							placeholder={
								isSetup
									? t(
											'settings:security.session.autoLock.setupPin.confirmPlaceholder'
										)
									: t(
											'settings:security.session.autoLock.setupPin.confirmPinPlaceholder'
										)
							}
							className={INPUT_CLASS}
						/>
					</>
				)}

				{usesPassphrase && (
					<label className='flex cursor-pointer items-center gap-2.5 text-sm text-[var(--text-secondary)] transition-colors hover:text-[var(--text-primary)]'>
						<input
							type='checkbox'
							checked={useEncryption}
							onChange={(e) => setUseEncryption(e.target.checked)}
							className='rounded accent-[var(--accent-color)]'
						/>
						{t('settings:security.session.autoLock.setupPin.usePassword')}
					</label>
				)}

				<div className='flex gap-2 pt-1'>
					<button
						type='button'
						disabled={isLoading}
						onClick={handleSubmit}
						className='flex-1 rounded-lg py-2 text-sm font-medium transition-all hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50'
						style={{
							background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
							color: 'var(--accent-text)',
						}}>
						{isLoading ? '...' : isSetup ? 'Enable' : 'Save'}
					</button>
					<button
						type='button'
						disabled={isLoading}
						onClick={onCancel}
						className='flex-1 rounded-lg border border-[var(--border-subtle)] py-2 text-sm text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] disabled:cursor-not-allowed disabled:opacity-50'>
						Cancel
					</button>
				</div>
			</div>
		</motion.div>
	)
}

export function SecuritySettings() {
	const { t } = useSettingsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const { isLoading, run } = useAsyncState()
	const { settings, setSetting } = useSettingsStore()
	const accentColor = useThemeStore((s) => s.accentColor)

	const clipboardOptions = CLIPBOARD_DELAY_OPTIONS.map((o) => ({
		value: o.value,
		label: t(`settings:security.data.clearClipboard.options.${o.labelKey}`),
	}))

	const [lockEnabled, setLockEnabled] = useState(false)
	const [lockTimeout, setLockTimeout] = useState(DEFAULT_LOCK_TIMEOUT)
	const [lockConfigured, setLockConfigured] = useState(false)
	const [usesPassphrase, setUsesPassphrase] = useState(false)
	const [pinFormMode, setPinFormMode] = useState<PinFormMode>(null)
	const [showTimeoutDropdown, setShowTimeoutDropdown] = useState(false)

	useEffect(() => {
		const load = async () => {
			try {
				const [currentTimeout, securityMethod, configured] = await Promise.all([
					invoke<number>('get_auto_lock_timeout'),
					invoke<string | null>('get_security_method'),
					invoke<boolean>('is_lock_configured'),
				])
				setLockTimeout(currentTimeout)
				setLockConfigured(configured)
				setLockEnabled(currentTimeout > 0 && configured)
				setUsesPassphrase(securityMethod === 'argon2')
			} catch (err) {
				console.error('Failed to load security settings:', err)
				toast.error('Failed to load security settings')
			}
		}
		load()
	}, [])

	const handleAutoLockToggle = useCallback(
		async (enabled: boolean) => {
			if (!enabled) {
				// Cancel pending setup without touching backend
				if (pinFormMode === 'setup') {
					setPinFormMode(null)
					return
				}
				try {
					await run(async () => {
						await invoke('set_auto_lock_timeout', { minutes: 0 })
						setLockTimeout(0)
						setLockEnabled(false)
					})
					toast.success('Auto-lock disabled')
				} catch (err) {
					console.error('Failed to disable auto-lock:', err)
					toast.error('Failed to disable auto-lock')
				}
				return
			}

			// Passphrase method can enable immediately - no PIN needed
			if (usesPassphrase) {
				try {
					await run(async () => {
						await invoke('use_encryption_password_for_lock')
						await invoke('set_auto_lock_timeout', { minutes: DEFAULT_LOCK_TIMEOUT })
						setLockTimeout(DEFAULT_LOCK_TIMEOUT)
						setLockEnabled(true)
						setLockConfigured(true)
					})
					toast.success('Auto-lock enabled')
				} catch (err) {
					console.error('Failed to enable auto-lock:', err)
					toast.error('Failed to enable auto-lock')
				}
				return
			}

			// Keyring/TPM - need a PIN before enabling
			setPinFormMode('setup')
		},
		[pinFormMode, run, usesPassphrase]
	)

	const handleTimeoutChange = useCallback(
		async (minutes: number) => {
			setShowTimeoutDropdown(false)
			try {
				await run(async () => {
					await invoke('set_auto_lock_timeout', { minutes })
					setLockTimeout(minutes)
					if (minutes === 0) setLockEnabled(false)
				})
				if (minutes === 0) toast.success('Auto-lock disabled')
			} catch (err) {
				console.error('Failed to update timeout:', err)
				toast.error('Failed to update timeout')
			}
		},
		[run]
	)

	const handlePinSubmit = useCallback(
		async (pin: string | null, useEncryption: boolean) => {
			try {
				await run(async () => {
					if (useEncryption) {
						await invoke('use_encryption_password_for_lock')
					} else {
						await invoke('set_auto_lock_pin', { pin })
					}

					if (pinFormMode === 'setup') {
						await invoke('set_auto_lock_timeout', { minutes: DEFAULT_LOCK_TIMEOUT })
						setLockTimeout(DEFAULT_LOCK_TIMEOUT)
						setLockEnabled(true)
					}

					setLockConfigured(true)
					setPinFormMode(null)
				})
				toast.success(pinFormMode === 'setup' ? 'Auto-lock enabled' : 'PIN changed')
			} catch (err) {
				console.error('Failed to configure lock:', err)
				toast.error('Failed to configure lock')
			}
		},
		[pinFormMode, run]
	)

	const handlePinCancel = useCallback(() => {
		setPinFormMode(null)
	}, [])

	const currentTimeoutOption = TIMEOUT_OPTIONS.find((o) => o.value === lockTimeout)
	const timeoutDisplayText = currentTimeoutOption
		? t(`settings:security.session.autoLock.timeout.options.${currentTimeoutOption.label}`)
		: `${lockTimeout} min`

	// Toggle appears ON either when lock is active or while setup form is open
	const toggleValue = lockEnabled || pinFormMode === 'setup'

	return (
		<div className='mx-auto flex w-full max-w-3xl flex-col space-y-8 p-8 pb-16'>
			<motion.div
				{...(animationsEnabled
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-[var(--text-primary)]'>
					{t('settings:security.title')}
				</h1>
				<p className='mt-1 text-[var(--text-secondary)]'>
					{t('settings:security.subtitle')}
				</p>
			</motion.div>

			<div className='space-y-8'>
				<section>
					<SectionTitle>{t('settings:security.session.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Timer}
							label={t('settings:security.session.autoLock.label')}
							description={t('settings:security.session.autoLock.description')}
							value={toggleValue}
							onChange={handleAutoLockToggle}
							disabled={isLoading}
						/>

						<AnimatePresence mode='wait'>
							{lockEnabled && lockConfigured && pinFormMode === null && (
								<motion.div
									key='timeout-controls'
									initial={{ opacity: 0, height: 0 }}
									animate={{ opacity: 1, height: 'auto' }}
									exit={{ opacity: 0, height: 0 }}
									transition={{ duration: 0.2, ease: 'easeOut' }}
									className='ml-11 flex flex-wrap items-center gap-2'>
									<div className='relative'>
										{showTimeoutDropdown && (
											<div
												className='fixed inset-0 z-[9]'
												onClick={() => setShowTimeoutDropdown(false)}
											/>
										)}
										<button
											type='button'
											onClick={() => setShowTimeoutDropdown((v) => !v)}
											className='relative z-10 flex items-center gap-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] px-4 py-2 text-sm text-[var(--text-primary)] transition-colors hover:bg-[var(--surface-hover)]'>
											<span>
												{t(
													'settings:security.session.autoLock.timeout.label'
												)}
												: {timeoutDisplayText}
											</span>
											<ChevronDown
												className={`h-4 w-4 transition-transform duration-200 ${
													showTimeoutDropdown ? 'rotate-180' : ''
												}`}
											/>
										</button>

										<AnimatePresence>
											{showTimeoutDropdown && (
												<motion.div
													initial={{ opacity: 0, y: -6 }}
													animate={{ opacity: 1, y: 0 }}
													exit={{ opacity: 0, y: -6 }}
													transition={{ duration: 0.15 }}
													className='absolute top-full left-0 z-20 mt-1 w-52 overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-glass)] py-1 shadow-xl'>
													{TIMEOUT_OPTIONS.map((option) => (
														<button
															type='button'
															key={option.value}
															onClick={() =>
																handleTimeoutChange(option.value)
															}
															className='flex w-full items-center justify-between px-4 py-2 text-left text-sm transition-colors hover:bg-[var(--surface-hover)]'>
															<span
																className={
																	lockTimeout === option.value
																		? 'text-[var(--text-primary)]'
																		: 'text-[var(--text-secondary)]'
																}>
																{t(
																	`settings:security.session.autoLock.timeout.options.${option.label}`
																)}
															</span>
															{lockTimeout === option.value && (
																<Check className='h-3.5 w-3.5 text-[var(--accent-color)]' />
															)}
														</button>
													))}
												</motion.div>
											)}
										</AnimatePresence>
									</div>

									{!usesPassphrase && (
										<button
											type='button'
											onClick={() => setPinFormMode('change')}
											className='flex h-9 items-center rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] px-4 text-sm text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
											Change PIN
										</button>
									)}
								</motion.div>
							)}

							{pinFormMode !== null && (
								<PinForm
									key={pinFormMode}
									mode={pinFormMode}
									usesPassphrase={usesPassphrase}
									isLoading={isLoading}
									onSubmit={handlePinSubmit}
									onCancel={handlePinCancel}
									t={t}
								/>
							)}
						</AnimatePresence>
					</div>
				</section>

				<section>
					<SectionTitle>{t('settings:security.data.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={FileKey}
							label={t('settings:security.data.encryptAttachments.label')}
							description={t('settings:security.data.encryptAttachments.description')}
							value={settings['encrypt-attachments']}
							onChange={(val) => setSetting('encrypt-attachments', val)}
						/>
						<div className='flex items-center justify-between rounded-2xl border border-[var(--border-faint)] bg-[var(--surface-panel)] p-4 transition-all duration-200 hover:border-[var(--border-subtle)] hover:bg-[var(--surface-hover)]'>
							<div className='flex items-center gap-4'>
								<div
									className='flex h-10 w-10 items-center justify-center rounded-xl ring-1 transition-all duration-200'
									style={
										settings['clear-clipboard-delay'] > 0
											? {
													backgroundColor: `rgba(var(--accent-rgb), 0.08)`,
													boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
												}
											: {
													backgroundColor: 'var(--surface-active)',
													boxShadow:
														'inset 0 0 0 1px var(--border-subtle)',
												}
									}>
									<ClipboardX
										className='h-[18px] w-[18px] transition-colors duration-200'
										style={
											settings['clear-clipboard-delay'] > 0
												? { color: accentColor }
												: { color: 'var(--text-secondary)' }
										}
									/>
								</div>
								<div>
									<h3 className='text-sm font-semibold text-[var(--text-primary)]'>
										{t('settings:security.data.clearClipboard.label')}
									</h3>
									<p className='max-w-[320px] text-xs leading-relaxed text-[var(--text-secondary)]'>
										{t('settings:security.data.clearClipboard.description')}
									</p>
								</div>
							</div>
							<InlineSelect
								value={settings['clear-clipboard-delay']}
								options={clipboardOptions}
								onChange={(v) => setSetting('clear-clipboard-delay', v)}
								accentColor={accentColor}
							/>
						</div>
					</div>
				</section>
			</div>
		</div>
	)
}
