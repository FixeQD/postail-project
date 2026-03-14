import { motion } from 'framer-motion'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import type { SettingCardProps } from '@/types/components/shared'

export function SettingCard({ label, description, icon: Icon, children, disabled = false }: SettingCardProps) {
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()

	return (
		<motion.div
			whileHover={animationsEnabled && !disabled ? { y: -2, transition: { duration: 0.2, ease: 'easeOut' } } : {}}
			className={`group relative flex items-center justify-between overflow-hidden rounded-2xl border bg-[var(--surface-panel)] p-4 transition-colors duration-300 ${
				disabled
					? 'cursor-not-allowed opacity-50 border-[var(--border-faint)]'
					: 'hover:bg-[var(--surface-hover)] border-[var(--border-subtle)]'
			}`}
		>
			{/* Ambient hover glow line */}
			{!disabled && animationsEnabled && (
				<div 
					className="pointer-events-none absolute inset-x-0 bottom-0 h-px w-full bg-gradient-to-r from-transparent via-white/10 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100 dark:via-white/5" 
				/>
			)}

			<div className='relative z-10 flex items-center gap-4'>
				<div
					className={`relative flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ring-1 transition-all duration-300 ${
						disabled
							? 'bg-[var(--surface-active)] ring-[var(--border-subtle)]'
							: 'bg-[var(--surface-panel)] ring-[var(--border-faint)] group-hover:ring-[var(--border-subtle)] group-hover:shadow-lg'
					}`}>
					<Icon className={`h-[18px] w-[18px] transition-colors duration-300 ${disabled ? 'text-muted-foreground' : 'text-[var(--text-secondary)] group-hover:text-foreground'}`} />
					
					{/* Subtle glow behind icon on hover */}
					{!disabled && animationsEnabled && (
						<div 
							className="absolute inset-0 -z-10 rounded-xl blur-md opacity-0 transition-opacity duration-500 group-hover:opacity-30"
							style={{ backgroundColor: accentColor }}
						/>
					)}
				</div>
				<div className="flex flex-col gap-0.5">
					<h3 className='text-sm font-semibold text-[var(--text-primary)] transition-colors duration-200 group-hover:text-foreground'>{label}</h3>
					<p className='max-w-[400px] text-xs leading-relaxed text-[var(--text-secondary)] transition-colors duration-200 group-hover:text-[var(--text-primary)]'>
						{description}
					</p>
				</div>
			</div>
			<div className='relative z-10 flex items-center gap-2'>{children}</div>
		</motion.div>
	)
}
