import { useState } from 'react'
import { motion } from 'framer-motion'
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
	const [isFocused, setIsFocused] = useState(false)
	const [isHovered, setIsHovered] = useState(false)

	return (
		<motion.div
			onHoverStart={() => setIsHovered(true)}
			onHoverEnd={() => setIsHovered(false)}
			className={cn(
				'relative flex h-11 w-full items-center gap-2 border-b border-zinc-900 bg-transparent px-0 transition-colors',
				className
			)}>
			<input
				type='text'
				value={value}
				onChange={(e) => onChange(e.target.value)}
				onFocus={() => setIsFocused(true)}
				onBlur={() => setIsFocused(false)}
				placeholder={placeholder}
				autoFocus={autoFocus}
				className='w-full bg-transparent py-1 text-sm font-medium text-zinc-100 outline-none placeholder:text-zinc-600'
			/>
			<motion.div
				initial={{ scaleX: 0 }}
				animate={{ scaleX: isFocused || isHovered ? 1 : 0 }}
				transition={{ duration: 0.25, ease: 'easeOut' }}
				className='pointer-events-none absolute bottom-0 left-0 h-[1px] w-full origin-center bg-zinc-500'
			/>
		</motion.div>
	)
}
