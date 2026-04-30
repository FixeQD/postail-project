import { useState } from 'react'

interface PanelFieldProps {
	icon: React.ReactNode
	label: string
	value: string
	onChange: (v: string) => void
	placeholder?: string
	type?: string
	accentColor: string
	className?: string
}

export function SearchField({
	icon,
	label,
	value,
	onChange,
	placeholder,
	type = 'text',
	accentColor,
	className,
}: PanelFieldProps) {
	const [focused, setFocused] = useState(false)

	return (
		<div className={`flex flex-col gap-1 ${className ?? ''}`}>
			<label className='flex items-center gap-1.5 text-[11px] font-medium text-[var(--text-secondary)]'>
				{icon}
				{label}
			</label>
			<input
				type={type}
				value={value}
				onChange={(e) => onChange(e.target.value)}
				onFocus={() => setFocused(true)}
				onBlur={() => setFocused(false)}
				placeholder={placeholder}
				className='h-8 rounded-lg border bg-[var(--surface-secondary)] px-3 text-xs text-[var(--text-primary)] placeholder-[var(--text-tertiary)] transition-all focus:outline-none'
				style={{
					borderColor: focused ? accentColor : 'var(--border-subtle)',
					boxShadow: focused ? `0 0 0 1px ${accentColor}` : 'none',
				}}
			/>
		</div>
	)
}
