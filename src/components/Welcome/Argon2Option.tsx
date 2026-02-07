import { useSecurityTranslation } from '../../hooks/useTypedTranslation'
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
			className={`group relative flex h-full w-full flex-col justify-between overflow-hidden rounded-2xl p-6 text-left ring-1 transition-all duration-300 ${
				isDisabled
					? 'cursor-not-allowed bg-slate-800/20 opacity-50 ring-slate-800/50'
					: 'cursor-pointer bg-slate-800/40 ring-white/[0.08] hover:bg-slate-800/70'
			}`}
			style={
				!isDisabled
					? ({
							'--tw-ring-hover-color': `rgba(var(--accent-rgb), 0.3)`,
						} as React.CSSProperties)
					: undefined
			}
			onClick={isDisabled ? undefined : onSelect}
			disabled={isDisabled}>
			{/* Hover glow effect */}
			{available && !disabled && (
				<div className='pointer-events-none absolute -inset-px rounded-2xl opacity-0 transition-opacity duration-500 group-hover:opacity-100'>
					<div
						className='absolute inset-0 rounded-2xl'
						style={{
							background: `linear-gradient(to bottom right, rgba(var(--accent-rgb), 0.08), transparent, transparent)`,
						}}
					/>
					<div
						className='absolute -top-20 -right-20 h-40 w-40 rounded-full blur-3xl transition-opacity group-hover:opacity-100'
						style={{ backgroundColor: `rgba(var(--accent-rgb), 0.06)` }}
					/>
				</div>
			)}

			<div className='relative'>
				{/* Status Badge */}
				<div className='absolute top-0 right-0'>
					<div
						className='flex items-center rounded-full px-2.5 py-1 text-[11px] font-semibold tracking-wide ring-1'
						style={{
							backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
							color: accentColor,
							boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
						}}>
						<Check className='mr-1 h-3 w-3' />
						{t('common:status.available')}
					</div>
				</div>

				{/* Icon */}
				<div
					className={`mb-5 flex h-12 w-12 items-center justify-center rounded-xl ring-1 transition-all duration-300 ${
						available ? '' : 'bg-slate-900/50 ring-white/[0.06]'
					}`}
					style={
						available
							? {
									backgroundColor: `rgba(var(--accent-rgb), 0.08)`,
									boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
								}
							: undefined
					}>
					<Lock
						className={`h-6 w-6 transition-colors duration-300 ${available ? '' : 'text-slate-600'}`}
						style={available ? { color: accentColor } : undefined}
					/>
				</div>

				{/* Title */}
				<h3 className='mb-2 text-[15px] font-semibold text-slate-100'>
					{t('security:options.argon2.title')}
				</h3>

				{/* Description */}
				<p className='mb-4 text-sm leading-relaxed text-slate-500'>
					{t('security:options.argon2.description')}
				</p>
			</div>

			{/* Footer */}
			<div className='relative mt-auto'>
				{/* Divider */}
				<div className='mb-3 h-px bg-gradient-to-r from-transparent via-white/[0.06] to-transparent' />

				<p className='text-xs' style={{ color: accentColor }}>
					{t('security:options.argon2.status.available')}
				</p>

				{!loading && (
					<div className='mt-2 flex items-center text-xs text-slate-500'>
						<KeyRound
							className='mr-1.5 h-3 w-3'
							style={{ color: `rgba(var(--accent-rgb), 0.7)` }}
						/>
						Password-based encryption
					</div>
				)}

				{loading && (
					<div className='mt-2 flex items-center text-xs' style={{ color: accentColor }}>
						<div
							className='mr-1.5 h-3 w-3 animate-spin rounded-full border border-transparent'
							style={{ borderColor: accentColor, borderTopColor: 'transparent' }}
						/>
						Initializing...
					</div>
				)}
			</div>
		</button>
	)
}
