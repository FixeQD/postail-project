import { useSecurityTranslation } from '../../hooks/useTypedTranslation'
import { Cpu, Shield, Check, X } from 'lucide-react'

export const TPMOption = ({
	available,
	onSelect,
}: {
	available: boolean
	onSelect: () => void
}) => {
	const { t } = useSecurityTranslation()

	const cardClasses = available
		? 'cursor-pointer bg-slate-800/50 ring-slate-700/50 hover:bg-slate-800 hover:ring-green-400/50'
		: 'cursor-not-allowed bg-slate-800/20 ring-slate-800/50 opacity-60'

	const accentClasses = available ? 'text-green-400' : 'text-slate-500'

	return (
		<button
			type='button'
			className={`relative flex h-full w-full flex-col justify-between rounded-xl p-6 text-left ring-1 transition-all duration-200 ${cardClasses}`}
			onClick={available ? onSelect : undefined}
			disabled={!available}>
			<div>
				{/* Status Badge */}
				<div className='absolute top-4 right-4'>
					{available ? (
						<div className='flex items-center rounded-full bg-green-900/50 px-2 py-1 text-xs font-medium text-green-400 ring-1 ring-green-400/20'>
							<Check className='mr-1 h-3 w-3' />
							{t('common:status.recommended')}
						</div>
					) : (
						<div className='flex items-center rounded-full bg-slate-800/50 px-2 py-1 text-xs font-medium text-slate-400 ring-1 ring-slate-700/50'>
							<X className='mr-1 h-3 w-3' />
							{t('common:status.unavailable')}
						</div>
					)}
				</div>

				{/* Icon */}
				<div
					className={`mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-slate-900/50 ring-1 ring-slate-700/50`}>
					<Cpu className={`h-6 w-6 ${accentClasses}`} />
				</div>

				{/* Title */}
				<h3
					className={`mb-2 font-semibold ${available ? 'text-slate-100' : 'text-slate-400'}`}>
					{t('security:options.tpm.title')}
				</h3>

				{/* Description */}
				<p className='mb-4 text-sm text-slate-400'>
					{t('security:options.tpm.description')}
				</p>
			</div>

			{/* Footer */}
			<div className='mt-auto'>
				<p className='text-xs'>
					{available ? (
						<span className={accentClasses}>
							{t('security:options.tpm.status.available')}
						</span>
					) : (
						<span className='text-slate-500'>
							{t('security:options.tpm.status.unavailable')}
						</span>
					)}
				</p>

				{available && (
					<div className='mt-2 flex items-center text-xs text-slate-400'>
						<Shield className={`mr-1.5 h-3 w-3 ${accentClasses}`} />
						Hardware-based encryption
					</div>
				)}
			</div>
		</button>
	)
}
