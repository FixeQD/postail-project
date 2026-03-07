import { useSecurityTranslation } from '@/hooks/useTypedTranslation'
import { Cpu, Shield, Check, X, ShieldAlert } from 'lucide-react'

export const TPMOption = ({
	available,
	requiresElevation = false,
	onSelect,
	disabled = false,
	loading = false,
}: {
	available: boolean
	requiresElevation?: boolean
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
					? 'cursor-not-allowed bg-black/[0.02] opacity-50 ring-black/10 dark:bg-slate-800/20 dark:ring-slate-800/50'
					: 'cursor-pointer bg-black/[0.04] ring-black/[0.08] hover:bg-black/[0.07] hover:ring-green-600/30 dark:bg-slate-800/40 dark:ring-white/[0.08] dark:hover:bg-slate-800/70 dark:hover:ring-green-400/30'
			}`}
			onClick={isDisabled ? undefined : onSelect}
			disabled={isDisabled}>
			{/* Hover glow effect */}
			{available && !disabled && (
				<div className='pointer-events-none absolute -inset-px rounded-2xl opacity-0 transition-opacity duration-500 group-hover:opacity-100'>
					<div className='absolute inset-0 rounded-2xl bg-gradient-to-br from-green-500/[0.08] via-transparent to-transparent' />
					<div className='absolute -top-20 -right-20 h-40 w-40 rounded-full bg-green-500/[0.06] blur-3xl transition-opacity group-hover:opacity-100' />
				</div>
			)}

			<div className='relative'>
				{/* Status Badge */}
				<div className='absolute top-0 right-0 flex flex-col items-end gap-1.5'>
					{available ? (
						<div className='flex items-center rounded-full bg-green-500/10 px-2.5 py-1 text-[11px] font-semibold tracking-wide text-green-600 ring-1 ring-green-500/20 dark:text-green-400'>
							<Check className='mr-1 h-3 w-3' />
							{t('common:status.recommended')}
						</div>
					) : (
						<div className='flex items-center rounded-full bg-black/[0.04] px-2.5 py-1 text-[11px] font-medium text-slate-500 ring-1 ring-black/10 dark:bg-slate-800/60 dark:ring-white/[0.06]'>
							<X className='mr-1 h-3 w-3' />
							{t('common:status.unavailable')}
						</div>
					)}

					{/* Admin Required Badge */}
					{available && requiresElevation && (
						<div className='flex items-center rounded-full bg-amber-500/10 px-2.5 py-1 text-[11px] font-medium text-amber-600 ring-1 ring-amber-500/20 dark:text-amber-400 dark:ring-amber-400/20'>
							<ShieldAlert className='mr-1 h-3 w-3' />
							Requires Admin
						</div>
					)}
				</div>

				{/* Icon */}
				<div
					className={`mb-5 flex h-12 w-12 items-center justify-center rounded-xl ring-1 transition-all duration-300 ${
						available
							? 'bg-green-500/[0.08] ring-green-500/20 group-hover:bg-green-500/[0.12] group-hover:ring-green-500/30'
							: 'bg-black/[0.04] ring-black/[0.06] dark:bg-slate-900/50 dark:ring-white/[0.06]'
					}`}>
					<Cpu
						className={`h-6 w-6 transition-colors duration-300 ${available ? 'text-green-600 dark:text-green-400' : 'text-slate-400 dark:text-slate-600'}`}
					/>
				</div>

				{/* Title */}
				<h3
					className={`mb-2 text-[15px] font-semibold ${available ? 'text-slate-900 dark:text-slate-100' : 'text-slate-400 dark:text-slate-500'}`}>
					{t('security:options.tpm.title')}
				</h3>

				{/* Description */}
				<p className='mb-4 text-sm leading-relaxed text-slate-500 dark:text-slate-500'>
					{t('security:options.tpm.description')}
				</p>
			</div>

			{/* Footer */}
			<div className='relative mt-auto'>
				{/* Divider */}
				<div className='mb-3 h-px bg-gradient-to-r from-transparent via-black/[0.06] to-transparent dark:via-white/[0.06]' />

				<p className='text-xs'>
					{available ? (
						<span className='text-green-600 dark:text-green-400'>
							{t('security:options.tpm.status.available')}
						</span>
					) : (
						<span className='text-slate-400 dark:text-slate-600'>
							{t('security:options.tpm.status.unavailable')}
						</span>
					)}
				</p>

				{available && !loading && (
					<div className='mt-2 flex items-center text-xs text-slate-500'>
						<Shield className='mr-1.5 h-3 w-3 text-green-500/70 dark:text-green-400/70' />
						Hardware-based encryption
					</div>
				)}

				{available && requiresElevation && !loading && (
					<div className='mt-2 flex items-center text-xs text-amber-600/80 dark:text-amber-400/70'>
						<ShieldAlert className='mr-1.5 h-3 w-3' />
						Administrator permissions required
					</div>
				)}

				{loading && (
					<div className='mt-2 flex items-center text-xs text-green-600 dark:text-green-400'>
						<div className='mr-1.5 h-3 w-3 animate-spin rounded-full border border-green-600 border-t-transparent dark:border-green-400 dark:border-t-transparent' />
						Initializing...
					</div>
				)}
			</div>
		</button>
	)
}
