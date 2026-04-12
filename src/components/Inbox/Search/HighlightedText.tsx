import { memo } from 'react'

interface HighlightedTextProps {
	text: string
	query: string
	className?: string
}

export const HighlightedText = memo(function HighlightedText({
	text,
	query,
	className,
}: HighlightedTextProps) {
	if (!query.trim()) {
		return <span className={className}>{text}</span>
	}

	const terms = query
		.split(/\s+/)
		.filter((t) => t.length > 2 && !t.includes(':'))
		.map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))

	if (!terms.length) {
		return <span className={className}>{text}</span>
	}

	const regex = new RegExp(`(${terms.join('|')})`, 'gi')
	const matchRegex = new RegExp(`^(${terms.join('|')})$`, 'i')
	const parts = text.split(regex)

	return (
		<span className={className}>
			{parts.map((part, i) =>
				matchRegex.test(part) ? (
					<mark
						key={i}
						className='rounded-[2px] bg-yellow-300/30 text-inherit dark:bg-yellow-500/25'>
						{part}
					</mark>
				) : (
					part
				)
			)}
		</span>
	)
})
