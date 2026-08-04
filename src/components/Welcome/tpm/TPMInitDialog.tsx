import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Shield, Lock, AlertCircle, Loader2, CheckCircle, XCircle } from 'lucide-react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { useSecurityTranslation } from '@/hooks/useTypedTranslation'
import { useShellTransition } from '@/hooks/useShellTransition'
import { cn } from '@/lib/utils'
import { RecoveryStep } from '@/components/Welcome/recovery/RecoveryStep'
import type { TPMInitDialogProps } from '@/types/components/welcome'

type TpmStatus =
	| 'checking'
	| 'available'
	| 'requires_elevation'
	| 'not_available'
	| 'initializing'
	| 'success'
	| 'error'
	| 'elevation_cancelled'

/** Matches error strings produced by initialize_tpm_elevated in security.rs */
function isCancellationError(msg: string): boolean {
	const lower = msg.toLowerCase()
	return (
		lower.includes('cancelled') ||
		lower.includes('canceled') ||
		lower.includes('authorization failed') ||
		lower.includes('pkexec') ||
		lower.includes('helper failed or was cancelled')
	)
}

const SUCCESS_TO_RECOVERY_DELAY_MS = 1400

export function TPMInitDialog({ open, onClose, onSuccess, requiresElevation }: TPMInitDialogProps) {
	const { t } = useSecurityTranslation()
	const [status, setStatus] = useState<TpmStatus>('checking')
	const [error, setError] = useState<string | null>(null)
	const [showRecoveryStep, setShowRecoveryStep] = useState(false)
	const { shellScope, contentScope, transition, reset } = useShellTransition()
	const recoveryCompleted = useRef(false)

	const checkTpmAvailability = useCallback(async () => {
		if (requiresElevation !== undefined) {
			setStatus(requiresElevation ? 'requires_elevation' : 'available')
			return
		}
		setStatus('checking')
		setError(null)
		try {
			const result = await invoke<string>('check_tpm_availability')
			switch (result) {
				case 'Available':
					setStatus('available')
					break
				case 'RequiresElevation':
					setStatus('requires_elevation')
					break
				case 'NotAvailable':
				default:
					setStatus('not_available')
			}
		} catch (e) {
			setStatus('not_available')
			setError(e instanceof Error ? e.message : String(e))
		}
	}, [requiresElevation])

	useEffect(() => {
		if (open) {
			checkTpmAvailability()
		}
	}, [open, checkTpmAvailability])

	const switchToRecovery = useCallback(() => {
		transition(() => setShowRecoveryStep(true))
	}, [transition])

	const handleInitialize = useCallback(async () => {
		setStatus('initializing')
		setError(null)
		try {
			await invoke('initialize_security', { method: 'tpm' })
			setStatus('success')
			setTimeout(() => switchToRecovery(), SUCCESS_TO_RECOVERY_DELAY_MS)
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e)
			if (isCancellationError(msg)) {
				setStatus('elevation_cancelled')
			} else {
				setStatus('error')
				setError(msg)
			}
		}
	}, [switchToRecovery])

	const handleRecoveryVerified = useCallback(async () => {
		recoveryCompleted.current = true
		await onSuccess()
		onClose()
	}, [onSuccess, onClose])

	const handleOpenChange = useCallback(
		(o: boolean) => {
			if (!o) {
				reset()
				if (showRecoveryStep && !recoveryCompleted.current) {
					invoke('reset_security_setup').catch((e) =>
						console.error('[TPMInitDialog] Failed to roll back security setup:', e)
					)
				}
				onClose()
			}
		},
		[onClose, reset, showRecoveryStep]
	)

	const handleAnimationEnd = useCallback(() => {
		if (!open) {
			recoveryCompleted.current = false
			setShowRecoveryStep(false)
		}
	}, [open])

	// Go back to the elevation prompt so user can try authenticating again
	const handleRetryFromCancelled = useCallback(() => {
		setStatus(requiresElevation ? 'requires_elevation' : 'available')
	}, [requiresElevation])

	const renderContent = () => {
		switch (status) {
			case 'checking':
				return (
					<div className='flex flex-col items-center py-8'>
						<Loader2 className='mb-4 h-12 w-12 animate-spin text-status-info' />
						<p className='text-[var(--text-primary)]'>
							{t('security:tpm.status.checking')}
						</p>
					</div>
				)

			case 'available':
				return (
					<div className='py-4'>
						<div className='mb-6 flex items-center justify-center'>
							<div className='flex h-16 w-16 items-center justify-center rounded-full bg-green-900/30 ring-2 ring-green-400/30'>
								<Shield className='h-8 w-8 text-status-success' />
							</div>
						</div>
						<div className='mb-6 text-center'>
							<h3 className='mb-2 text-lg font-semibold text-[var(--text-primary)]'>
								{t('security:tpm.available.title')}
							</h3>
							<p className='text-sm text-[var(--text-secondary)]'>
								{t('security:tpm.available.description')}
							</p>
						</div>
						<div className='flex gap-3'>
							<Button variant='outline' onClick={onClose} className='flex-1'>
								{t('actions.cancel')}
							</Button>
							<Button
								onClick={handleInitialize}
								className='flex-1 bg-status-success hover:bg-status-success'>
								{t('security:tpm.available.cta')}
							</Button>
						</div>
					</div>
				)

			case 'requires_elevation':
				return (
					<div className='py-4'>
						<div className='mb-6 flex items-center justify-center'>
							<div className='flex h-16 w-16 items-center justify-center rounded-full bg-status-warning/15 ring-2 ring-status-warning/30'>
								<Lock className='h-8 w-8 text-status-warning' />
							</div>
						</div>
						<div className='mb-6 text-center'>
							<h3 className='mb-2 text-lg font-semibold text-[var(--text-primary)]'>
								{t('security:tpm.elevation.title')}
							</h3>
							<p className='text-sm text-[var(--text-secondary)]'>
								{t('security:tpm.elevation.description')}
							</p>
						</div>
						<div className='flex gap-3'>
							<Button variant='outline' onClick={onClose} className='flex-1'>
								{t('actions.cancel')}
							</Button>
							<Button
								onClick={handleInitialize}
								className='flex-1 bg-status-warning hover:brightness-110'>
								<Shield className='mr-2 h-4 w-4' />
								{t('security:tpm.elevation.cta')}
							</Button>
						</div>
					</div>
				)

			case 'not_available':
				return (
					<div className='py-4'>
						<div className='mb-6 flex items-center justify-center'>
							<div className='flex h-16 w-16 items-center justify-center rounded-full bg-red-900/30 ring-2 ring-red-400/30'>
								<AlertCircle className='h-8 w-8 text-destructive' />
							</div>
						</div>
						<div className='mb-6 text-center'>
							<h3 className='mb-2 text-lg font-semibold text-[var(--text-primary)]'>
								{t('security:tpm.notAvailable.title')}
							</h3>
							<p className='text-sm text-[var(--text-secondary)]'>
								{t('security:tpm.notAvailable.description')}
							</p>
						</div>
						<Button variant='outline' onClick={onClose} className='w-full'>
							{t('security:tpm.notAvailable.cta')}
						</Button>
					</div>
				)

			case 'initializing':
				return (
					<div className='flex flex-col items-center py-8'>
						<Loader2 className='mb-4 h-12 w-12 animate-spin text-status-warning' />
						<p className='text-[var(--text-primary)]'>
							{t('security:tpm.status.initializing')}
						</p>
						<p className='mt-2 text-sm text-[var(--text-secondary)]'>
							{t('security:tpm.status.initializingHint')}
						</p>
					</div>
				)

			case 'success':
				return (
					<div className='flex flex-col items-center py-8'>
						<div className='mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-900/30 ring-2 ring-green-400/30'>
							<CheckCircle className='h-8 w-8 text-status-success' />
						</div>
						<p className='font-medium text-[var(--text-primary)]'>
							{t('security:tpm.status.success')}
						</p>
						<p className='mt-2 text-sm text-[var(--text-secondary)]'>
							{t('security:tpm.status.redirecting')}
						</p>
					</div>
				)

			case 'elevation_cancelled':
				return (
					<div className='py-4'>
						<div className='mb-6 flex items-center justify-center'>
							<div className='flex h-16 w-16 items-center justify-center rounded-full bg-[var(--surface-active)] ring-2 ring-[var(--border-subtle)]'>
								<XCircle className='h-8 w-8 text-[var(--text-secondary)]' />
							</div>
						</div>
						<div className='mb-6 text-center'>
							<h3 className='mb-2 text-lg font-semibold text-[var(--text-primary)]'>
								{t('security:tpm.cancelled.title')}
							</h3>
							<p className='text-sm text-[var(--text-secondary)]'>
								{t('security:tpm.cancelled.description')}
							</p>
						</div>
						<div className='flex gap-3'>
							<Button variant='outline' onClick={onClose} className='flex-1'>
								{t('security:tpm.cancelled.chooseAnother')}
							</Button>
							<Button
								onClick={handleRetryFromCancelled}
								className='flex-1 bg-status-warning hover:brightness-110'>
								<Shield className='mr-2 h-4 w-4' />
								{t('security:tpm.cancelled.retry')}
							</Button>
						</div>
					</div>
				)

			case 'error':
				return (
					<div className='py-4'>
						<div className='mb-6 flex items-center justify-center'>
							<div className='flex h-16 w-16 items-center justify-center rounded-full bg-red-900/30 ring-2 ring-red-400/30'>
								<AlertCircle className='h-8 w-8 text-destructive' />
							</div>
						</div>
						<div className='mb-6 text-center'>
							<h3 className='mb-2 text-lg font-semibold text-[var(--text-primary)]'>
								{t('security:tpm.error.title')}
							</h3>
							{error && (
								<p className='mt-1 text-sm text-[var(--text-secondary)]'>
									{error}
								</p>
							)}
						</div>
						<div className='flex gap-3'>
							<Button variant='outline' onClick={onClose} className='flex-1'>
								{t('security:tpm.error.cancel')}
							</Button>
							<Button onClick={handleInitialize} className='flex-1'>
								{t('security:tpm.error.retry')}
							</Button>
						</div>
					</div>
				)
		}
	}

	return (
		<>
			<Dialog open={open} onOpenChange={handleOpenChange}>
				<DialogContent
					onAnimationEnd={handleAnimationEnd}
					className={cn(
						'overflow-hidden border-[var(--border-subtle)] bg-[var(--surface-glass-solid)] p-0 text-[var(--text-primary)] sm:max-w-md',
						showRecoveryStep && 'sm:max-w-2xl'
					)}>
					<div ref={shellScope} className='w-full'>
						<div ref={contentScope} className='p-6'>
							<DialogHeader className='mb-2'>
								<DialogTitle className='flex items-center gap-2'>
									<Shield className='h-5 w-5 text-status-success' />
									{t('security:tpm.dialog.title')}
								</DialogTitle>
								<DialogDescription className='text-[var(--text-secondary)]'>
									{t('security:tpm.dialog.description')}
								</DialogDescription>
							</DialogHeader>

							{showRecoveryStep ? (
								<RecoveryStep
									variant='embedded'
									onNext={handleRecoveryVerified}
									encryptionMethod='tpm'
								/>
							) : (
								renderContent()
							)}
						</div>
					</div>
				</DialogContent>
			</Dialog>
		</>
	)
}
