import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Shield, Lock, AlertCircle, Loader2, CheckCircle } from 'lucide-react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'

type TpmStatus =
	| 'checking'
	| 'available'
	| 'requires_elevation'
	| 'not_available'
	| 'initializing'
	| 'success'
	| 'error'

interface TPMInitDialogProps {
	open: boolean
	onClose: () => void
	onSuccess: () => void
}

export function TPMInitDialog({ open, onClose, onSuccess }: TPMInitDialogProps) {
	const [status, setStatus] = useState<TpmStatus>('checking')
	const [error, setError] = useState<string | null>(null)

	const checkTpmAvailability = useCallback(async () => {
		setStatus('checking')
		setError(null)
		try {
			const result = await invoke<{ status: string }>('check_tpm_availability')
			switch (result.status) {
				case 'Available':
					setStatus('available')
					break
				case 'RequiresElevation':
					setStatus('requires_elevation')
					break
				case 'NotAvailable':
					setStatus('not_available')
					break
				default:
					setStatus('not_available')
			}
		} catch (e) {
			setStatus('not_available')
			setError(e instanceof Error ? e.message : 'Failed to check TPM availability')
		}
	}, [])

	useEffect(() => {
		if (open) {
			checkTpmAvailability()
		}
	}, [open, checkTpmAvailability])

	const handleInitialize = useCallback(async () => {
		setStatus('initializing')
		setError(null)
		try {
			await invoke('initialize_security', { method: 'tpm' })
			setStatus('success')
			setTimeout(() => {
				onSuccess()
			}, 1500)
		} catch (e) {
			setStatus('error')
			setError(e instanceof Error ? e.message : 'Failed to initialize TPM')
		}
	}, [onSuccess])

	const renderContent = () => {
		switch (status) {
			case 'checking':
				return (
					<div className='flex flex-col items-center py-8'>
						<Loader2 className='mb-4 h-12 w-12 animate-spin text-blue-400' />
						<p className='text-slate-300'>Checking TPM availability...</p>
					</div>
				)

			case 'available':
				return (
					<div className='py-4'>
						<div className='mb-6 flex items-center justify-center'>
							<div className='flex h-16 w-16 items-center justify-center rounded-full bg-green-900/30 ring-2 ring-green-400/30'>
								<Shield className='h-8 w-8 text-green-400' />
							</div>
						</div>
						<div className='mb-6 text-center'>
							<h3 className='mb-2 text-lg font-semibold text-slate-100'>
								TPM2 Available
							</h3>
							<p className='text-sm text-slate-400'>
								Your system has a TPM2 chip that can be used for hardware-based
								encryption.
							</p>
						</div>
						<div className='flex gap-3'>
							<Button variant='outline' onClick={onClose} className='flex-1'>
								Cancel
							</Button>
							<Button
								onClick={handleInitialize}
								className='flex-1 bg-green-600 hover:bg-green-500'>
								Initialize TPM
							</Button>
						</div>
					</div>
				)

			case 'requires_elevation':
				return (
					<div className='py-4'>
						<div className='mb-6 flex items-center justify-center'>
							<div className='flex h-16 w-16 items-center justify-center rounded-full bg-amber-900/30 ring-2 ring-amber-400/30'>
								<Lock className='h-8 w-8 text-amber-400' />
							</div>
						</div>
						<div className='mb-6 text-center'>
							<h3 className='mb-2 text-lg font-semibold text-slate-100'>
								Administrator Access Required
							</h3>
							<p className='text-sm text-slate-400'>
								TPM is available but requires administrator permissions to
								initialize. You will be prompted for authentication.
							</p>
						</div>
						<div className='flex gap-3'>
							<Button variant='outline' onClick={onClose} className='flex-1'>
								Cancel
							</Button>
							<Button
								onClick={handleInitialize}
								className='flex-1 bg-amber-600 hover:bg-amber-500'>
								<Shield className='mr-2 h-4 w-4' />
								Authenticate &amp; Initialize
							</Button>
						</div>
					</div>
				)

			case 'not_available':
				return (
					<div className='py-4'>
						<div className='mb-6 flex items-center justify-center'>
							<div className='flex h-16 w-16 items-center justify-center rounded-full bg-red-900/30 ring-2 ring-red-400/30'>
								<AlertCircle className='h-8 w-8 text-red-400' />
							</div>
						</div>
						<div className='mb-6 text-center'>
							<h3 className='mb-2 text-lg font-semibold text-slate-100'>
								TPM Not Available
							</h3>
							<p className='text-sm text-slate-400'>
								No TPM2 chip was detected on this system, or it is not accessible.
								Please use an alternative security method.
							</p>
						</div>
						<Button variant='outline' onClick={onClose} className='w-full'>
							Choose Another Method
						</Button>
					</div>
				)

			case 'initializing':
				return (
					<div className='flex flex-col items-center py-8'>
						<Loader2 className='mb-4 h-12 w-12 animate-spin text-amber-400' />
						<p className='text-slate-300'>Initializing TPM security...</p>
						<p className='mt-2 text-sm text-slate-500'>This may take a moment</p>
					</div>
				)

			case 'success':
				return (
					<div className='flex flex-col items-center py-8'>
						<div className='mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-900/30 ring-2 ring-green-400/30'>
							<CheckCircle className='h-8 w-8 text-green-400' />
						</div>
						<p className='font-medium text-slate-100'>TPM Initialized Successfully!</p>
						<p className='mt-2 text-sm text-slate-500'>Redirecting...</p>
					</div>
				)

			case 'error':
				return (
					<div className='py-4'>
						<div className='mb-6 flex items-center justify-center'>
							<div className='flex h-16 w-16 items-center justify-center rounded-full bg-red-900/30 ring-2 ring-red-400/30'>
								<AlertCircle className='h-8 w-8 text-red-400' />
							</div>
						</div>
						<div className='mb-6 text-center'>
							<h3 className='mb-2 text-lg font-semibold text-slate-100'>
								Initialization Failed
							</h3>
							<p className='text-sm text-slate-400'>{error}</p>
						</div>
						<div className='flex gap-3'>
							<Button variant='outline' onClick={onClose} className='flex-1'>
								Cancel
							</Button>
							<Button onClick={handleInitialize} className='flex-1'>
								Retry
							</Button>
						</div>
					</div>
				)
		}
	}

	return (
		<Dialog open={open} onOpenChange={(o) => !o && onClose()}>
			<DialogContent className='border-slate-800 bg-slate-900/95 text-slate-100 backdrop-blur-xl sm:max-w-md'>
				<DialogHeader>
					<DialogTitle className='flex items-center gap-2'>
						<Shield className='h-5 w-5 text-green-400' />
						TPM Security Setup
					</DialogTitle>
					<DialogDescription className='text-slate-400'>
						Configure hardware-based encryption using your TPM chip
					</DialogDescription>
				</DialogHeader>
				{renderContent()}
			</DialogContent>
		</Dialog>
	)
}
