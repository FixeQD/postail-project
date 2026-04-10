import { memo } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { format, isToday, isYesterday, isThisYear } from 'date-fns'
import { Search, Paperclip } from 'lucide-react'

// Keep unused imports
void format
void isToday
void isYesterday
void isThisYear
void Paperclip
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useThemeStore } from '@/stores/themeStore'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { HighlightedText } from './HighlightedText'
import type { SearchResult } from '@/types/mail'

interface SearchResultsListProps {
	results: SearchResult[]
	isLoading?: boolean
	error?: string | null
	query: string
	onMessageClick: (uid: number, mailbox: string) => void
}

const ResultRow = memo(function ResultRow({
	result,
	query,
	accentColor,
	animationsEnabled,
	index,
	onClick,
}: {
	result: SearchResult
	query: string
	accentColor: string
	animationsEnabled: boolean
	index: number
	onClick: () => void
}) {
	void accentColor
	const { t } = useTypedTranslation()

	return (
		<motion.button
			type='button'
			onClick={onClick}
			initial={animationsEnabled ? { opacity: 0, y: 8 } : {}}
			animate={animationsEnabled ? { opacity: 1, y: 0 } : {}}
			transition={
				animationsEnabled
					? { duration: 0.18, delay: index * 0.03, ease: [0.16, 1, 0.3, 1] }
					: {}
			}
			whileHover={animationsEnabled ? { backgroundColor: 'var(--surface-hover)', x: 2 } : {}}
			className='flex w-full flex-col gap-1 rounded-xl px-4 py-3 text-left transition-colors'>
			<div className='flex items-start justify-between gap-2'>
				<p className='min-w-0 flex-1 truncate text-sm font-medium text-[var(--text-primary)]'>
					{result.subject ? (
						<HighlightedText text={result.subject} query={query} />
					) : (
						t('inbox:messageView.noSubject')
					)}
				</p>
				<span className='shrink-0 text-[11px] text-[var(--text-tertiary)]'>
					{result.mailbox}
				</span>
			</div>

			{result.from_addr && (
				<p className='truncate text-xs text-[var(--text-secondary)]'>
					<HighlightedText text={result.from_addr} query={query} />
				</p>
			)}

			{result.snippet && (
				<p className='line-clamp-2 text-xs leading-relaxed text-[var(--text-tertiary)]'>
					<HighlightedText text={result.snippet} query={query} />
				</p>
			)}
		</motion.button>
	)
})

export function SearchResultsList({ results, query, onMessageClick }: SearchResultsListProps) {
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const { t } = useTypedTranslation()

	if (!results.length) {
		return (
			<motion.div
				initial={animationsEnabled ? { opacity: 0, scale: 0.96 } : {}}
				animate={animationsEnabled ? { opacity: 1, scale: 1 } : {}}
				transition={{ duration: 0.2 }}
				className='flex h-full flex-col items-center justify-center gap-3 text-center'>
				<div
					className='flex h-14 w-14 items-center justify-center rounded-2xl'
					style={{ backgroundColor: `${accentColor}18` }}>
					<Search className='h-6 w-6' style={{ color: accentColor }} />
				</div>
				<div>
					<p className='text-sm font-medium text-[var(--text-primary)]'>
						{t('inbox:search.results.empty')}
					</p>
					<p className='mt-0.5 text-xs text-[var(--text-tertiary)]'>{query}</p>
				</div>
			</motion.div>
		)
	}

	return (
		<div className='flex flex-col overflow-hidden'>
			{/* Header */}
			<div className='flex items-center justify-between border-b border-[var(--border-subtle)] px-4 py-2'>
				<p className='text-xs text-[var(--text-tertiary)]'>
					{t('inbox:search.results.count', { count: results.length })}
				</p>
				<p className='max-w-[60%] truncate text-[11px] text-[var(--text-tertiary)] italic'>
					&ldquo;{query}&rdquo;
				</p>
			</div>

			{/* Results */}
			<div className='flex-1 overflow-y-auto py-1'>
				<AnimatePresence initial={false}>
					{results.map((result, i) => (
						<ResultRow
							key={`${result.account_id}-${result.mailbox}-${result.uid}`}
							result={result}
							query={query}
							accentColor={accentColor}
							animationsEnabled={animationsEnabled}
							index={i}
							onClick={() => onMessageClick(result.uid, result.mailbox)}
						/>
					))}
				</AnimatePresence>
			</div>
		</div>
	)
}
