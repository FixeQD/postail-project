import { XCircle, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useSecurityTranslation } from '@/hooks/useTypedTranslation'

interface TPMUnlockFailedProps {
	error: { message: string; cancelled: boolean } | null
	onRetry: () => void
}

export function TPMUnlockFailed({ error, onRetry }: TPMUnlockFailedProps) {
	const { t } = useSecurityTranslation()

	const description = error?.cancelled
		? t('security:tpm.unlockFailed.cancelledDescription')
		: t('security:tpm.unlockFailed.errorDescription')

	return (
		<div className='noise-overlay flex h-full flex-col items-center justify-center px-6'>
			<div className='w-full max-w-sm text-center'>
				{/* Icon */}
				<div className='mb-6 flex justify-center'>
					<div className='flex h-20 w-20 items-center justify-center rounded-2xl bg-slate-800/80 ring-1 ring-white/10'>
						<XCircle className='h-10 w-10 text-slate-400' />
					</div>
				</div>

				{/* Text */}
				<h1 className='mb-3 text-2xl font-bold tracking-tight text-slate-100'>
					{t('security:tpm.unlockFailed.title')}
				</h1>
				<p className='mb-2 text-sm leading-relaxed text-slate-400'>{description}</p>
				{error && !error.cancelled && (
					<p className='mb-6 rounded-lg bg-slate-800/60 px-3 py-2 font-mono text-xs text-slate-500'>
						{error.message}
					</p>
				)}
				{error?.cancelled && <div className='mb-6' />}

				{/* Actions */}
				<div className='flex flex-col gap-3'>
					<Button onClick={onRetry} className='w-full gap-2'>
						<RefreshCw className='h-4 w-4' />
						{t('security:tpm.unlockFailed.retry')}
					</Button>
				</div>
			</div>
		</div>
	)
}
