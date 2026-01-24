import { useSecurityTranslation } from '../../hooks/useTypedTranslation'
import { Lock, Check, KeyRound } from 'lucide-react'

export const Argon2Option = ({
	available,
	onSelect,
	disabled = false,
	loading = false,
}: {
	available: boolean
	onSelect: () => void
	disabled?: boolean
	loading?: boolean
}) => {
	const { t } = useSecurityTranslation()

	const cardClasses =
		available && !disabled
			? 'cursor-pointer bg-slate-800/50 ring-slate-700/50 hover:bg-slate-800 hover:ring-orange-400/50'
			: 'cursor-not-allowed bg-slate-800/20 ring-slate-800/50 opacity-60'

	const accentClasses = available ? 'text-orange-400' : 'text-slate-500'

	return (
		<button
			type='button'
			className={`relative flex h-full w-full flex-col justify-between rounded-xl p-6 text-left ring-1 transition-all duration-200 ${cardClasses}`}
			onClick={available && !disabled ? onSelect : undefined}
			disabled={!available || disabled}>
			<div>
				{/* Status Badge */}
				<div className='absolute top-4 right-4'>
					<div className='flex items-center rounded-full bg-orange-900/50 px-2 py-1 text-xs font-medium text-orange-400 ring-1 ring-orange-400/20'>
						<Check className='mr-1 h-3 w-3' />
						{t('common:status.available')}
					</div>
				</div>

				{/* Icon */}
				<div
					className={`mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-slate-900/50 ring-1 ring-slate-700/50`}>
					<Lock className={`h-6 w-6 ${accentClasses}`} />
				</div>

				{/* Title */}
				<h3 className='mb-2 font-semibold text-slate-100'>
					{t('security:options.argon2.title')}
				</h3>

				{/* Description */}
				<p className='mb-4 text-sm text-slate-400'>
					{t('security:options.argon2.description')}
				</p>
			</div>

			{/* Footer */}
			<div className='mt-auto'>
				<p className={`text-xs ${accentClasses}`}>
					{t('security:options.argon2.status.available')}
				</p>
				{!loading && (
					<div className='mt-2 flex items-center text-xs text-slate-400'>
						<KeyRound className={`mr-1.5 h-3 w-3 ${accentClasses}`} />
						Password-based encryption
					</div>
				)}

				{loading && (
					<div className='mt-2 flex items-center text-xs text-orange-400'>
						<div className='mr-1.5 h-3 w-3 animate-spin rounded-full border border-orange-400 border-t-transparent'></div>
						Initializing...
					</div>
				)}
			</div>
		</button>
	)
}
