import { motion } from 'framer-motion'
import type { LucideIcon } from 'lucide-react'
import { useThemeStore } from '@/stores/themeStore'

interface ToggleSettingProps {
	value: boolean
	onChange: (value: boolean) => void
	label: string
	description: string
	icon: LucideIcon
	disabled?: boolean
}

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
			className={`group flex items-center justify-between rounded-2xl border border-white/[0.05] bg-white/[0.03] p-4 transition-all duration-200 ${
				disabled
					? 'cursor-not-allowed opacity-50'
					: 'hover:border-white/[0.08] hover:bg-white/[0.06]'
			}`}>
			<div className='flex items-center gap-4'>
				<div
					className={`flex h-10 w-10 items-center justify-center rounded-xl ring-1 transition-all duration-200 ${
						value ? '' : 'bg-slate-900 ring-white/[0.08]'
					} ${!disabled ? 'group-hover:ring-white/[0.12]' : ''}`}
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
							value ? '' : 'text-slate-400'
						}`}
						style={value ? { color: accentColor } : undefined}
					/>
				</div>
				<div>
					<h3 className='text-sm font-semibold text-slate-200'>{label}</h3>
					<p className='max-w-[400px] text-xs leading-relaxed text-slate-500'>
						{description}
					</p>
				</div>
			</div>
			<button
				type='button'
				disabled={disabled}
				onClick={() => onChange(!value)}
				className={`relative h-6 w-11 shrink-0 rounded-full transition-all duration-200 ${
					value ? 'shadow-sm' : 'bg-slate-800 hover:bg-slate-700'
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
						value ? 'bg-white' : 'bg-slate-300'
					}`}
					style={
						value ? { boxShadow: `0 1px 3px rgba(var(--accent-rgb), 0.2)` } : undefined
					}
				/>
			</button>
		</div>
	)
}
