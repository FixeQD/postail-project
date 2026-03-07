import { motion } from 'framer-motion'
import { useThemeStore } from '@/stores/themeStore'
import type { ToggleSettingProps } from '@/types/components/ui'

export function ToggleSetting({
	value,
	onChange,
	label,
	description,
	icon: Icon,
	disabled = false,
}: ToggleSettingProps) {
	const accentColor = useThemeStore((s) => s.accentColor)

	return (
		<div
			className={`group flex items-center justify-between rounded-2xl border border-[var(--border-faint)] bg-[var(--surface-panel)] p-4 transition-all duration-200 ${
				disabled
					? 'cursor-not-allowed opacity-50'
					: 'hover:border-[var(--border-subtle)] hover:bg-[var(--surface-hover)]'
			}`}>
			<div className='flex items-center gap-4'>
				<div
					className={`flex h-10 w-10 items-center justify-center rounded-xl ring-1 transition-all duration-200 ${
						value ? '' : 'bg-[var(--surface-active)] ring-[var(--border-subtle)]'
					} ${!disabled ? 'group-hover:ring-[var(--border-subtle)]' : ''}`}
					style={
						value
							? {
									backgroundColor: `rgba(var(--accent-rgb), 0.08)`,
									boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
								}
							: undefined
					}>
					<Icon
						className={`h-[18px] w-[18px] transition-colors duration-200 ${
							value ? '' : 'text-[var(--text-secondary)]'
						}`}
						style={value ? { color: accentColor } : undefined}
					/>
				</div>
				<div>
					<h3 className='text-sm font-semibold text-[var(--text-primary)]'>{label}</h3>
					<p className='max-w-[400px] text-xs leading-relaxed text-[var(--text-secondary)]'>
						{description}
					</p>
				</div>
			</div>
			<button
				type='button'
				disabled={disabled}
				onClick={() => onChange(!value)}
				className={`relative h-6 w-11 shrink-0 rounded-full transition-all duration-200 ${
					value
						? 'shadow-sm'
						: 'bg-[var(--surface-active)] hover:bg-[var(--border-subtle)]'
				} ${disabled ? 'cursor-not-allowed' : 'cursor-pointer'}`}
				style={
					value
						? {
								backgroundColor: accentColor,
								boxShadow: `0 1px 3px rgba(var(--accent-rgb), 0.3)`,
							}
						: undefined
				}>
				<motion.div
					transition={{
						type: 'spring',
						stiffness: 500,
						damping: 30,
					}}
					animate={{ x: value ? 22 : 2 }}
					className={`absolute top-1 left-0 h-4 w-4 rounded-full shadow-sm ${
						value ? 'bg-white' : 'bg-[var(--text-tertiary)]'
					}`}
					style={
						value ? { boxShadow: `0 1px 3px rgba(var(--accent-rgb), 0.2)` } : undefined
					}
				/>
			</button>
		</div>
	)
}
