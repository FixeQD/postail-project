import { useSecurityTranslation } from '../../../hooks/useTypedTranslation'
import { Key, Check, X, ShieldCheck } from 'lucide-react'

export const KeyringOption = ({
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

	const isDisabled = !available || disabled

	return (
		<button
			type='button'
			className={`group relative flex h-full w-full flex-col justify-between overflow-hidden rounded-2xl p-6 text-left ring-1 transition-all duration-300 ${
				isDisabled
					? 'cursor-not-allowed bg-slate-800/20 opacity-50 ring-slate-800/50'
					: 'cursor-pointer bg-slate-800/40 ring-white/[0.08] hover:bg-slate-800/70 hover:ring-blue-400/30'
			}`}
			onClick={isDisabled ? undefined : onSelect}
			disabled={isDisabled}>
			{/* Hover glow effect */}
			{available && !disabled && (
				<div className='pointer-events-none absolute -inset-px rounded-2xl opacity-0 transition-opacity duration-500 group-hover:opacity-100'>
					<div className='absolute inset-0 rounded-2xl bg-gradient-to-br from-blue-500/[0.08] via-transparent to-transparent' />
					<div className='absolute -top-20 -right-20 h-40 w-40 rounded-full bg-blue-500/[0.06] blur-3xl transition-opacity group-hover:opacity-100' />
				</div>
			)}

			<div className='relative'>
				{/* Status Badge */}
				<div className='absolute top-0 right-0'>
					{loading ? (
						<div className='flex items-center rounded-full bg-slate-800/60 px-2.5 py-1 text-[11px] font-medium text-slate-400 ring-1 ring-white/[0.06]'>
							<div className='mr-1.5 h-3 w-3 animate-spin rounded-full border-2 border-slate-400 border-t-transparent' />
							{t('common:status.loading')}
						</div>
					) : available ? (
						<div className='flex items-center rounded-full bg-blue-500/10 px-2.5 py-1 text-[11px] font-semibold tracking-wide text-blue-400 ring-1 ring-blue-400/20'>
							<Check className='mr-1 h-3 w-3' />
							{t('common:status.available')}
						</div>
					) : (
						<div className='flex items-center rounded-full bg-slate-800/60 px-2.5 py-1 text-[11px] font-medium text-slate-500 ring-1 ring-white/[0.06]'>
							<X className='mr-1 h-3 w-3' />
							{t('common:status.unavailable')}
						</div>
					)}
				</div>

				{/* Icon */}
				<div
					className={`mb-5 flex h-12 w-12 items-center justify-center rounded-xl ring-1 transition-all duration-300 ${
						available
							? 'bg-blue-500/[0.08] ring-blue-500/20 group-hover:bg-blue-500/[0.12] group-hover:ring-blue-500/30'
							: 'bg-slate-900/50 ring-white/[0.06]'
					}`}>
					<Key
						className={`h-6 w-6 transition-colors duration-300 ${available ? 'text-blue-400' : 'text-slate-600'}`}
					/>
				</div>

				{/* Title */}
				<h3
					className={`mb-2 text-[15px] font-semibold ${available ? 'text-slate-100' : 'text-slate-500'}`}>
					{t('security:options.keyring.title')}
				</h3>

				{/* Description */}
				<p className='mb-4 text-sm leading-relaxed text-slate-500'>
					{t('security:options.keyring.description')}
				</p>
			</div>

			{/* Footer */}
			<div className='relative mt-auto'>
				{/* Divider */}
				<div className='mb-3 h-px bg-gradient-to-r from-transparent via-white/[0.06] to-transparent' />

				<p className='text-xs'>
					{available ? (
						<span className='text-blue-400'>
							{t('security:options.keyring.status.available')}
						</span>
					) : (
						<span className='text-slate-600'>
							{t('security:options.keyring.status.unavailable')}
						</span>
					)}
				</p>

				{available && !loading && (
					<div className='mt-2 flex items-center text-xs text-slate-500'>
						<ShieldCheck className='mr-1.5 h-3 w-3 text-blue-400/70' />
						System integration
					</div>
				)}

				{loading && (
					<div className='mt-2 flex items-center text-xs text-blue-400'>
						<div className='mr-1.5 h-3 w-3 animate-spin rounded-full border border-blue-400 border-t-transparent' />
						Initializing...
					</div>
				)}
			</div>
		</button>
	)
}
