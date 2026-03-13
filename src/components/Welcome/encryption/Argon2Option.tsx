import { useSecurityTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
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
	const accentColor = useThemeStore((s) => s.accentColor)

	const isDisabled = !available || disabled

	return (
		<button
			type='button'
			className={`group relative flex h-full w-full flex-col justify-between overflow-hidden rounded-3xl p-6 text-left transition-all duration-300 focus:ring-2 focus:ring-offset-2 focus:ring-offset-[var(--app-bg)] focus:outline-none ${
				isDisabled
					? 'cursor-not-allowed border border-[var(--border-subtle)] bg-[var(--surface-panel)] opacity-60'
					: 'cursor-pointer border border-[var(--border-faint)] bg-[var(--surface-glass)] hover:-translate-y-1 hover:border-[var(--border-subtle)] hover:bg-[var(--surface-panel)] hover:shadow-xl'
			}`}
			onClick={isDisabled ? undefined : onSelect}
			disabled={isDisabled}>
			{/* Hover glow effect */}
			{available && !disabled && (
				<div className='pointer-events-none absolute -inset-px rounded-3xl opacity-0 transition-opacity duration-500 group-hover:opacity-100'>
					<div
						className='absolute inset-0 rounded-3xl'
						style={{
							background: `linear-gradient(to bottom right, rgba(var(--accent-rgb), 0.1), transparent, transparent)`,
						}}
					/>
					<div
						className='absolute -top-20 -right-20 h-40 w-40 rounded-full blur-3xl transition-opacity group-hover:opacity-100'
						style={{ backgroundColor: `rgba(var(--accent-rgb), 0.1)` }}
					/>
				</div>
			)}

			<div className='relative'>
				{/* Status Badge */}
				<div className='absolute top-0 right-0'>
					<div
						className='flex items-center rounded-full px-2.5 py-1 text-[11px] font-bold tracking-wider uppercase ring-1 ring-[var(--border-subtle)]'
						style={{
							backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
							color: accentColor,
							boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
						}}>
						<Check className='mr-1 h-3.5 w-3.5' />
						{t('common:status.available')}
					</div>
				</div>

				{/* Icon */}
				<div
					className={`mb-5 flex h-14 w-14 items-center justify-center rounded-2xl ring-1 transition-all duration-300 ${
						available
							? 'group-hover:scale-110 group-hover:shadow-lg'
							: 'bg-[var(--surface-active)] ring-[var(--border-subtle)]'
					}`}
					style={
						available
							? {
									backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
									boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
								}
							: undefined
					}>
					<Lock
						className={`h-7 w-7 transition-colors duration-300 ${available ? '' : 'text-[var(--text-tertiary)]'}`}
						style={available ? { color: accentColor } : undefined}
					/>
				</div>

				{/* Title */}
				<h3
					className={`mb-2 text-lg font-bold tracking-tight ${available ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}`}>
					{t('security:options.argon2.title')}
				</h3>

				{/* Description */}
				<p className='text-muted-foreground mb-4 text-sm leading-relaxed'>
					{t('security:options.argon2.description')}
				</p>
			</div>

			{/* Footer */}
			<div className='relative mt-auto'>
				{/* Divider */}
				<div className='mb-4 h-px bg-[var(--border-faint)]' />

				<p className='text-xs font-medium' style={{ color: accentColor }}>
					{t('security:options.argon2.status.available')}
				</p>

				{!loading && (
					<div className='text-muted-foreground mt-2 flex items-center text-xs'>
						<KeyRound
							className='mr-1.5 h-3.5 w-3.5'
							style={{ color: `rgba(var(--accent-rgb), 0.8)` }}
						/>
						Password-based encryption
					</div>
				)}

				{loading && (
					<div
						className='mt-2 flex items-center text-xs font-medium'
						style={{ color: accentColor }}>
						<div
							className='mr-2 h-3.5 w-3.5 animate-spin rounded-full border-2 border-transparent'
							style={{ borderColor: accentColor, borderTopColor: 'transparent' }}
						/>
						Initializing...
					</div>
				)}
			</div>
		</button>
	)
}
