import { useMemo } from 'react'
import { cn } from '@/lib/utils'

interface ContactAvatarProps {
	name?: string | null
	email: string
	size?: 'sm' | 'md' | 'lg' | 'xl'
	className?: string
}

export function ContactAvatar({ name, email, size = 'md', className }: ContactAvatarProps) {
	const initials = useMemo(() => {
		if (!name) return email.slice(0, 2).toUpperCase()
		const parts = name.trim().split(/\s+/)
		if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase()
		return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase()
	}, [name, email])

	// Deterministic color based on email
	const colorIndex = useMemo(() => {
		let hash = 0
		for (let i = 0; i < email.length; i++) {
			hash = email.charCodeAt(i) + ((hash << 5) - hash)
		}
		return Math.abs(hash) % 5
	}, [email])

	const bgColors = [
		'bg-blue-500/10 text-blue-500 border-blue-500/20',
		'bg-indigo-500/10 text-indigo-500 border-indigo-500/20',
		'bg-purple-500/10 text-purple-500 border-purple-500/20',
		'bg-emerald-500/10 text-emerald-500 border-emerald-500/20',
		'bg-rose-500/10 text-rose-500 border-rose-500/20',
	]

	const sizeClasses = {
		sm: 'h-7 w-7 text-[10px]',
		md: 'h-9 w-9 text-[11px]',
		lg: 'h-12 w-12 text-[14px]',
		xl: 'h-20 w-20 text-[24px]',
	}

	return (
		<div
			className={cn(
				'flex shrink-0 items-center justify-center rounded-full font-bold border transition-colors',
				bgColors[colorIndex],
				sizeClasses[size],
				className
			)}>
			{initials}
		</div>
	)
}
