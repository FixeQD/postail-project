import { cn } from '@/lib/utils'

interface SubjectInputProps {
	value: string
	onChange: (value: string) => void
	placeholder?: string
	className?: string
	autoFocus?: boolean
}

export function SubjectInput({
	value,
	onChange,
	placeholder,
	className,
	autoFocus,
}: SubjectInputProps) {
	return (
		<div
			className={cn(
				'flex h-11 w-full items-center gap-2 border-b border-zinc-900 bg-transparent px-0 transition-colors focus-within:border-zinc-700',
				className
			)}>
			<input
				type='text'
				value={value}
				onChange={(e) => onChange(e.target.value)}
				placeholder={placeholder}
				autoFocus={autoFocus}
				className='w-full bg-transparent py-1 text-sm font-medium text-zinc-100 outline-none placeholder:text-zinc-600'
			/>
		</div>
	)
}
