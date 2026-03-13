import { useSecurityTranslation } from '@/hooks/useTypedTranslation'
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
			className={`group relative flex h-full w-full flex-col justify-between overflow-hidden rounded-3xl p-6 text-left transition-all duration-300 focus:ring-2 focus:ring-offset-2 focus:ring-offset-[var(--app-bg)] focus:outline-none ${
				isDisabled
					? 'cursor-not-allowed border border-[var(--border-subtle)] bg-[var(--surface-panel)] opacity-60'
					: 'cursor-pointer border border-[var(--border-faint)] bg-[var(--surface-glass)] hover:-translate-y-1 hover:border-blue-500/30 hover:bg-[var(--surface-panel)] hover:shadow-xl'
			}`}
			onClick={isDisabled ? undefined : onSelect}
			disabled={isDisabled}>
			{/* Hover glow effect */}
			{available && !disabled && (
				<div className='pointer-events-none absolute -inset-px rounded-3xl opacity-0 transition-opacity duration-500 group-hover:opacity-100'>
					<div className='absolute inset-0 rounded-3xl bg-gradient-to-br from-blue-500/10 via-transparent to-transparent' />
					<div className='absolute -top-20 -right-20 h-40 w-40 rounded-full bg-blue-500/10 blur-3xl transition-opacity group-hover:opacity-100' />
				</div>
			)}

			<div className='relative'>
				{/* Status Badge */}
				<div className='absolute top-0 right-0 flex flex-col items-end gap-1.5'>
					{loading ? (
						<div className='flex items-center rounded-full bg-[var(--surface-active)] px-2.5 py-1 text-[11px] font-bold tracking-wider text-[var(--text-tertiary)] uppercase ring-1 ring-[var(--border-subtle)]'>
							<div className='mr-1.5 h-3.5 w-3.5 animate-spin rounded-full border-2 border-slate-400 border-t-transparent' />
							{t('common:status.loading')}
						</div>
					) : available ? (
						<div className='flex items-center rounded-full bg-blue-500/10 px-2.5 py-1 text-[11px] font-bold tracking-wider text-blue-500 uppercase ring-1 ring-blue-500/20'>
							<Check className='mr-1 h-3.5 w-3.5' />
							{t('common:status.available')}
						</div>
					) : (
						<div className='flex items-center rounded-full bg-[var(--surface-active)] px-2.5 py-1 text-[11px] font-bold tracking-wider text-[var(--text-tertiary)] uppercase ring-1 ring-[var(--border-subtle)]'>
							<X className='mr-1 h-3.5 w-3.5' />
							{t('common:status.unavailable')}
						</div>
					)}
				</div>

				{/* Icon */}
				<div
					className={`mb-5 flex h-14 w-14 items-center justify-center rounded-2xl ring-1 transition-all duration-300 ${
						available
							? 'bg-blue-500/10 ring-blue-500/20 group-hover:scale-110 group-hover:bg-blue-500/20 group-hover:shadow-lg'
							: 'bg-[var(--surface-active)] ring-[var(--border-subtle)]'
					}`}>
					<Key
						className={`h-7 w-7 transition-colors duration-300 ${available ? 'text-blue-500' : 'text-[var(--text-tertiary)]'}`}
					/>
				</div>

				{/* Title */}
				<h3
					className={`mb-2 text-lg font-bold tracking-tight ${available ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}`}>
					{t('security:options.keyring.title')}
				</h3>

				{/* Description */}
				<p className='text-muted-foreground mb-4 text-sm leading-relaxed'>
					{t('security:options.keyring.description')}
				</p>
			</div>

			{/* Footer */}
			<div className='relative mt-auto'>
				{/* Divider */}
				<div className='mb-4 h-px bg-[var(--border-faint)]' />

				<p className='text-xs font-medium'>
					{available ? (
						<span className='text-blue-500'>
							{t('security:options.keyring.status.available')}
						</span>
					) : (
						<span className='text-[var(--text-tertiary)]'>
							{t('security:options.keyring.status.unavailable')}
						</span>
					)}
				</p>

				{available && !loading && (
					<div className='text-muted-foreground mt-2 flex items-center text-xs'>
						<ShieldCheck className='mr-1.5 h-3.5 w-3.5 text-blue-500/80' />
						System integration
					</div>
				)}

				{loading && (
					<div className='mt-2 flex items-center text-xs font-medium text-blue-500'>
						<div className='mr-2 h-3.5 w-3.5 animate-spin rounded-full border-2 border-blue-500 border-t-transparent' />
						Initializing...
					</div>
				)}
			</div>
		</button>
	)
}
