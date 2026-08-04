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
			className={`group relative flex h-full w-full flex-col justify-between overflow-hidden rounded-3xl p-6 text-left transition-all duration-300 focus:ring-2 focus:ring-offset-2 focus:ring-offset-[var(--app-bg)] focus:outline-none ${
				isDisabled
					? 'cursor-not-allowed border border-[var(--border-subtle)] bg-[var(--surface-panel)] opacity-60'
					: 'cursor-pointer border border-[var(--border-faint)] bg-[var(--surface-glass)] hover:-translate-y-1 hover:border-status-success/30 hover:bg-[var(--surface-panel)] hover:shadow-xl'
			}`}
			onClick={isDisabled ? undefined : onSelect}
			disabled={isDisabled}>
			{/* Hover glow effect */}
			{available && !disabled && (
				<div className='pointer-events-none absolute -inset-px rounded-3xl opacity-0 transition-opacity duration-500 group-hover:opacity-100'>
					<div className='absolute inset-0 rounded-3xl bg-gradient-to-br from-green-500/10 via-transparent to-transparent' />
					<div className='absolute -top-20 -right-20 h-40 w-40 rounded-full bg-status-success/15 blur-3xl transition-opacity group-hover:opacity-100' />
				</div>
			)}

			<div className='relative'>
				{/* Status Badge */}
				<div className='absolute top-0 right-0 flex flex-col items-end gap-1.5'>
					{available ? (
						<div className='flex items-center rounded-full bg-status-success/15 px-2.5 py-1 text-[11px] font-bold tracking-wider text-status-success uppercase ring-1 ring-status-success/30'>
							<Check className='mr-1 h-3.5 w-3.5' />
							{t('common:status.recommended')}
						</div>
					) : (
						<div className='flex items-center rounded-full bg-[var(--surface-active)] px-2.5 py-1 text-[11px] font-bold tracking-wider text-[var(--text-tertiary)] uppercase ring-1 ring-[var(--border-subtle)]'>
							<X className='mr-1 h-3.5 w-3.5' />
							{t('common:status.unavailable')}
						</div>
					)}

					{/* Admin Required Badge */}
					{available && requiresElevation && (
						<div className='flex items-center rounded-full bg-status-warning/15 px-2.5 py-1 text-[11px] font-bold tracking-wider text-status-warning uppercase ring-1 ring-status-warning/30'>
							<ShieldAlert className='mr-1 h-3.5 w-3.5' />
							Requires Admin
						</div>
					)}
				</div>

				{/* Icon */}
				<div
					className={`mb-5 flex h-14 w-14 items-center justify-center rounded-2xl ring-1 transition-all duration-300 ${
						available
							? 'bg-status-success/15 ring-status-success/30 group-hover:scale-110 group-hover:bg-status-success/15 group-hover:shadow-lg'
							: 'bg-[var(--surface-active)] ring-[var(--border-subtle)]'
					}`}>
					<Cpu
						className={`h-7 w-7 transition-colors duration-300 ${available ? 'text-status-success' : 'text-[var(--text-tertiary)]'}`}
					/>
				</div>

				{/* Title */}
				<h3
					className={`mb-2 text-lg font-bold tracking-tight ${available ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}`}>
					{t('security:options.tpm.title')}
				</h3>

				{/* Description */}
				<p className='text-muted-foreground mb-4 text-sm leading-relaxed'>
					{t('security:options.tpm.description')}
				</p>
			</div>

			{/* Footer */}
			<div className='relative mt-auto'>
				{/* Divider */}
				<div className='mb-4 h-px bg-[var(--border-faint)]' />

				<p className='text-xs font-medium'>
					{available ? (
						<span className='text-status-success'>
							{t('security:options.tpm.status.available')}
						</span>
					) : (
						<span className='text-[var(--text-tertiary)]'>
							{t('security:options.tpm.status.unavailable')}
						</span>
					)}
				</p>

				{available && !loading && (
					<div className='text-muted-foreground mt-2 flex items-center text-xs'>
						<Shield className='mr-1.5 h-3.5 w-3.5 text-status-success' />
						Hardware-based encryption
					</div>
				)}

				{available && requiresElevation && !loading && (
					<div className='mt-2 flex items-center text-xs text-status-warning'>
						<ShieldAlert className='mr-1.5 h-3.5 w-3.5' />
						Administrator permissions required
					</div>
				)}

				{loading && (
					<div className='mt-2 flex items-center text-xs font-medium text-status-success'>
						<div className='mr-2 h-3.5 w-3.5 animate-spin rounded-full border-2 border-status-success border-t-transparent' />
						Initializing...
					</div>
				)}
			</div>
		</button>
	)
}
