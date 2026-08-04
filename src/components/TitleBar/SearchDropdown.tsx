import { motion, AnimatePresence } from 'framer-motion'
import { Clock, X, Bookmark, Trash2 } from 'lucide-react'
import type { SavedSearch } from '@/types/search'

interface SearchDropdownProps {
	open: boolean
	searchHistory: string[]
	savedSearches: SavedSearch[]
	accentColor: string
	animationsEnabled: boolean
	inputRef: React.RefObject<HTMLInputElement | null>
	setRawInput: (val: string) => void
	setHistoryOpen: (val: boolean) => void
	clearHistory: () => void
	removeFromHistory: (q: string) => void
	onActivateSaved: (saved: SavedSearch) => void
	onDeleteSaved: (e: React.MouseEvent, saved: SavedSearch) => void
	t: (key: string) => string
}

export function SearchDropdown({
	open,
	searchHistory,
	savedSearches,
	accentColor,
	animationsEnabled,
	inputRef,
	setRawInput,
	setHistoryOpen,
	clearHistory,
	removeFromHistory,
	onActivateSaved,
	onDeleteSaved,
	t,
}: SearchDropdownProps) {
	return (
		<AnimatePresence>
			{open && (
				<motion.div
					key='history'
					initial={animationsEnabled ? { opacity: 0, y: -6, scale: 0.98 } : {}}
					animate={animationsEnabled ? { opacity: 1, y: 0, scale: 1 } : {}}
					exit={animationsEnabled ? { opacity: 0, y: -6, scale: 0.98 } : {}}
					transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
					className='glass absolute top-[calc(100%+4px)] right-0 left-0 z-50 overflow-hidden rounded-xl border border-[var(--border-subtle)] py-1 shadow-xl backdrop-blur-xl'
					style={{
						boxShadow: `0 8px 32px rgba(0,0,0,0.2), 0 0 0 1px var(--border-subtle)`,
					}}>
					{/* Recent searches */}
					{searchHistory.length > 0 && (
						<>
							<div className='flex items-center justify-between px-3 py-1.5'>
								<span className='text-[10px] font-semibold tracking-wider text-[var(--text-tertiary)] uppercase'>
									{t('inbox:search.history.title')}
								</span>
								<button
									type='button'
									onClick={clearHistory}
									className='text-[10px] text-[var(--text-tertiary)] transition-colors hover:text-[var(--text-secondary)]'>
									{t('inbox:search.history.clearAll')}
								</button>
							</div>
							{searchHistory.map((q) => (
								<div key={q} className='group flex items-center gap-2 px-2'>
									<button
										type='button'
										onMouseDown={(e) => {
											e.preventDefault()
											setRawInput(q)
											setHistoryOpen(false)
											setTimeout(() => inputRef.current?.focus(), 0)
										}}
										className='flex flex-1 items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
										<Clock className='h-3.5 w-3.5 shrink-0 text-[var(--text-tertiary)]' />
										<span className='truncate'>{q}</span>
									</button>
									<button
										type='button'
										onMouseDown={(e) => {
											e.preventDefault()
											removeFromHistory(q)
										}}
										className='hidden h-5 w-5 shrink-0 items-center justify-center rounded text-[var(--text-tertiary)] transition-colors group-hover:flex hover:text-[var(--text-primary)]'>
										<X className='h-3 w-3' />
									</button>
								</div>
							))}
						</>
					)}

					{searchHistory.length > 0 && savedSearches.length > 0 && (
						<div className='mx-3 my-1 border-t border-[var(--border-subtle)]' />
					)}

					{/* Saved searches */}
					{savedSearches.length > 0 && (
						<>
							<div className='px-3 py-1.5'>
								<span className='text-[10px] font-semibold tracking-wider text-[var(--text-tertiary)] uppercase'>
									{t('inbox:search.savedSearches.title')}
								</span>
							</div>
							{savedSearches.map((saved) => (
								<div key={saved.id} className='group flex items-center gap-2 px-2'>
									<button
										type='button'
										onMouseDown={(e) => {
											e.preventDefault()
											onActivateSaved(saved)
										}}
										className='flex flex-1 items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
										<Bookmark
											className='h-3.5 w-3.5 shrink-0'
											style={{ color: accentColor }}
										/>
										<span className='truncate'>{saved.name}</span>
									</button>
									<button
										type='button'
										onMouseDown={(e) => onDeleteSaved(e, saved)}
										className='hidden h-5 w-5 shrink-0 items-center justify-center rounded text-[var(--text-tertiary)] transition-colors group-hover:flex hover:text-destructive'>
										<Trash2 className='h-3 w-3' />
									</button>
								</div>
							))}
						</>
					)}
				</motion.div>
			)}
		</AnimatePresence>
	)
}
