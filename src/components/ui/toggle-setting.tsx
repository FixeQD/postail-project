import { motion } from 'framer-motion'
import type { LucideIcon } from 'lucide-react'

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
	return (
		<div
			className={`flex items-center justify-between p-4 rounded-2xl bg-white/5 border border-white/5 transition-colors ${
				disabled ? 'opacity-50 cursor-not-allowed' : 'hover:bg-white/10'
			}`}>
			<div className='flex items-center gap-4'>
				<div className='flex h-10 w-10 items-center justify-center rounded-xl bg-slate-900 ring-1 ring-white/10'>
					<Icon className='h-5 w-5 text-slate-400' />
				</div>
				<div>
					<h3 className='text-sm font-semibold text-slate-200'>{label}</h3>
					<p className='text-xs text-slate-500 max-w-[400px]'>{description}</p>
				</div>
			</div>
			<button
				type='button'
				disabled={disabled}
				onClick={() => onChange(!value)}
				className={`relative h-6 w-11 rounded-full transition-colors ${
					value ? 'bg-blue-600' : 'bg-slate-800'
				} ${disabled ? 'cursor-not-allowed' : ''}`}>
				<motion.div
					transition={{
						type: 'spring',
						stiffness: 500,
						damping: 30,
					}}
					animate={{ x: value ? 22 : 2 }}
					className='absolute top-1 left-0 h-4 w-4 rounded-full bg-white shadow-sm'
				/>
			</button>
		</div>
	)
}
