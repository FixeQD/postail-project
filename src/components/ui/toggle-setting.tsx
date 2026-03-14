import { motion } from 'framer-motion'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
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
	const animationsEnabled = useAnimationsEnabled()

	return (
		<motion.div
			whileHover={animationsEnabled && !disabled ? { y: -2, transition: { duration: 0.2, ease: 'easeOut' } } : {}}
			className={`group relative flex items-center justify-between overflow-hidden rounded-2xl border transition-colors duration-300 ${
				disabled
					? 'cursor-not-allowed opacity-50 border-[var(--border-faint)] bg-[var(--surface-panel)]'
					: value 
						? 'border-[var(--border-subtle)] hover:border-transparent'
						: 'border-[var(--border-faint)] bg-[var(--surface-panel)] hover:border-[var(--border-subtle)] hover:bg-[var(--surface-hover)]'
			}`}
			style={value && !disabled ? {
				backgroundColor: `rgba(var(--accent-rgb), 0.03)`,
				boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.15), 0 4px 20px -8px rgba(var(--accent-rgb), 0.1)`,
			} : undefined}
		>
			{/* Ambient hover glow line */}
			{!disabled && animationsEnabled && !value && (
				<div className="pointer-events-none absolute inset-x-0 bottom-0 h-px w-full bg-gradient-to-r from-transparent via-white/10 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100 dark:via-white/5" />
			)}
			
			{/* Active glow line */}
			{value && !disabled && (
				<div 
					className="pointer-events-none absolute inset-x-0 bottom-0 h-px w-full opacity-50" 
					style={{ background: `linear-gradient(to right, transparent, ${accentColor}, transparent)` }}
				/>
			)}

			<div className='relative z-10 flex items-center gap-4 p-4'>
				<div
					className={`relative flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ring-1 transition-all duration-300 ${
						value && !disabled
							? 'bg-transparent ring-transparent'
							: disabled
								? 'bg-[var(--surface-active)] ring-[var(--border-subtle)]'
								: 'bg-[var(--surface-panel)] ring-[var(--border-faint)] group-hover:ring-[var(--border-subtle)] group-hover:shadow-[0_0_12px_-4px_rgba(var(--accent-rgb),0.3)]'
					}`}
					style={
						value && !disabled
							? {
									backgroundColor: `rgba(var(--accent-rgb), 0.08)`,
									boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
								}
							: undefined
					}
				>
					<Icon
						className={`h-[18px] w-[18px] transition-colors duration-300 ${
							value && !disabled ? '' : disabled ? 'text-muted-foreground' : 'text-[var(--text-secondary)] group-hover:text-foreground'
						}`}
						style={value && !disabled ? { color: accentColor } : undefined}
					/>
					{!disabled && animationsEnabled && !value && (
						<div 
							className="absolute inset-0 -z-10 rounded-xl blur-md opacity-0 transition-opacity duration-500 group-hover:opacity-30"
							style={{ backgroundColor: accentColor }}
						/>
					)}
				</div>
				<div className="flex flex-col gap-0.5">
					<h3 className={`text-sm font-semibold transition-colors duration-200 ${value && !disabled ? 'text-foreground' : 'text-[var(--text-primary)] group-hover:text-foreground'}`}>{label}</h3>
					<p className={`max-w-[400px] text-xs leading-relaxed transition-colors duration-200 ${value && !disabled ? 'text-[var(--text-secondary)]' : 'text-[var(--text-secondary)] group-hover:text-[var(--text-primary)]'}`}>
						{description}
					</p>
				</div>
			</div>
			
			<div className="relative z-10 pr-4">
				<button
					type='button'
					disabled={disabled}
					onClick={() => onChange(!value)}
					className={`relative h-6 w-11 shrink-0 rounded-full transition-all duration-300 ${
						value
							? 'shadow-sm'
							: 'bg-[var(--surface-active)] hover:bg-[var(--border-strong)]'
					} ${disabled ? 'cursor-not-allowed' : 'cursor-pointer'}`}
					style={
						value
							? {
									backgroundColor: accentColor,
									boxShadow: `0 4px 12px -2px rgba(var(--accent-rgb), 0.4)`,
								}
							: undefined
					}>
					<motion.div
						transition={{
							type: 'spring',
							stiffness: 700,
							damping: 30,
						}}
						animate={{ x: value ? 22 : 2 }}
						className={`absolute top-1 left-0 h-4 w-4 rounded-full shadow-sm ${
							value ? 'bg-white' : 'bg-[var(--text-tertiary)]'
						}`}
						style={
							value ? { boxShadow: `0 2px 4px rgba(var(--accent-rgb), 0.3)` } : undefined
						}
					/>
				</button>
			</div>
		</motion.div>
	)
}
