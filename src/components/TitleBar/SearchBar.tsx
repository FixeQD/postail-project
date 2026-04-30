import { useState, useRef, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion, AnimatePresence } from 'framer-motion'
import {
	Search,
	SlidersHorizontal,
	X,
	Loader2,
	Bookmark,
	Check,
} from 'lucide-react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useThemeStore } from '@/stores/themeStore'
import { useAccountStore } from '@/stores/accountStore'
import type { Mailbox } from '@/types/mail'
import {
	parseSearchOperators,
	serializeSearchQuery,
	SEARCH_OPERATORS,
	SEARCH_SPLIT_REGEX,
	SEARCH_MATCH_REGEX,
} from '@/lib/searchQueryParser'
import { Popover, PopoverContent, PopoverAnchor } from '@/components/ui/popover'
import { useSearchHistory } from '@/hooks/useSearchHistory'
import type { AdvancedSearchQuery, SavedSearch } from '@/types/search'
import { useSearchBarStore } from '@/stores/searchStore'
import { SearchDropdown } from './SearchDropdown'
import { SearchPanel } from './SearchPanel'

interface SearchBarProps {
	onSearch: (query: AdvancedSearchQuery | null) => void
	isSearching?: boolean
}

const EASE_OUT_EXPO: [number, number, number, number] = [0.16, 1, 0.3, 1]

export { useSearchBarStore }

export function SearchBar({ onSearch, isSearching }: SearchBarProps) {
	const { t } = useTypedTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const { activeAccount } = useAccountStore()
	const queryClient = useQueryClient()

	const [focused, setFocused] = useState(false)
	const { rawInput, setRawInput, query, setQuery, hasActiveSearch, setHasActiveSearch } =
		useSearchBarStore()
	const [panelOpen, setPanelOpen] = useState(false)
	const [historyOpen, setHistoryOpen] = useState(false)

	const [isSaveMode, setIsSaveMode] = useState(false)
	const [saveName, setSaveName] = useState('')
	const [isSaving, setIsSaving] = useState(false)

	const {
		queries: searchHistory,
		addQuery: addToHistory,
		removeQuery: removeFromHistory,
		clearHistory,
	} = useSearchHistory()

	const { data: savedSearches = [] } = useQuery<SavedSearch[]>({
		queryKey: ['saved_searches', activeAccount?.id],
		queryFn: () =>
			invoke<SavedSearch[]>('get_saved_searches', { accountId: activeAccount!.id }),
		enabled: !!activeAccount?.id,
	})

	const inputRef = useRef<HTMLInputElement>(null)
	const overlayRef = useRef<HTMLDivElement>(null)
	const saveInputRef = useRef<HTMLInputElement>(null)
	const containerRef = useRef<HTMLDivElement>(null)

	const { data: mailboxes } = useQuery<Mailbox[]>({
		queryKey: ['mailboxes', activeAccount?.id],
		queryFn: () => invoke<Mailbox[]>('fetch_mailboxes', { accountId: activeAccount?.id }),
		enabled: !!activeAccount?.id && panelOpen,
	})

	const hasDropdownItems = searchHistory.length > 0 || savedSearches.length > 0

	useEffect(() => {
		function handleClick(e: MouseEvent) {
			if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
				setPanelOpen(false)
				setIsSaveMode(false)
			}
		}
		document.addEventListener('mousedown', handleClick)
		return () => document.removeEventListener('mousedown', handleClick)
	}, [])

	useEffect(() => {
		function handleKey(e: KeyboardEvent) {
			if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
				e.preventDefault()
				inputRef.current?.focus()
			}
		}
		window.addEventListener('keydown', handleKey)
		return () => window.removeEventListener('keydown', handleKey)
	}, [])

	useEffect(() => {
		let timeoutId: ReturnType<typeof setTimeout>
		if (isSaveMode) {
			timeoutId = setTimeout(() => saveInputRef.current?.focus(), 50)
		}
		return () => {
			if (timeoutId) clearTimeout(timeoutId)
		}
	}, [isSaveMode])

	const handleInputChange = useCallback(
		(value: string) => {
			setRawInput(value)
			if (!value.trim()) {
				setQuery({})
				if (hasDropdownItems) setHistoryOpen(true)
			} else {
				setHistoryOpen(false)
			}
		},
		[hasDropdownItems]
	)

	const handleClear = useCallback(() => {
		setRawInput('')
		setQuery({})
		setHasActiveSearch(false)
		setPanelOpen(false)
		setHistoryOpen(false)
		setIsSaveMode(false)
		setSaveName('')
		onSearch(null)
		window.dispatchEvent(new CustomEvent('postail:search', { detail: null }))
		inputRef.current?.blur()
	}, [onSearch])

	useEffect(() => {
		const onClear = () => handleClear()
		window.addEventListener('postail:search:clear', onClear)
		return () => window.removeEventListener('postail:search:clear', onClear)
	}, [handleClear])

	const handleSubmit = useCallback(() => {
		const parsed = parseSearchOperators(rawInput)
		const finalQuery: AdvancedSearchQuery = { ...query, ...parsed }

		const isEmpty =
			!finalQuery.from &&
			!finalQuery.to &&
			!finalQuery.subject &&
			!finalQuery.body &&
			!finalQuery.dateFrom &&
			!finalQuery.dateTo &&
			!finalQuery.hasAttachment &&
			!finalQuery.rawQuery

		if (isEmpty) {
			handleClear()
			return
		}

		setHasActiveSearch(true)
		setPanelOpen(false)
		setHistoryOpen(false)
		setIsSaveMode(false)
		addToHistory(serializeSearchQuery(finalQuery))
		onSearch(finalQuery)
		window.dispatchEvent(new CustomEvent('postail:search', { detail: finalQuery }))
	}, [query, rawInput, onSearch, addToHistory, handleClear])

	const handleKeyDown = useCallback(
		(e: React.KeyboardEvent) => {
			if (e.key === 'Enter') handleSubmit()
			if (e.key === 'Escape') {
				if (panelOpen) setPanelOpen(false)
				else handleClear()
			}
		},
		[handleSubmit, handleClear, panelOpen]
	)

	const updateField = useCallback(
		<K extends keyof AdvancedSearchQuery>(field: K, value: AdvancedSearchQuery[K]) => {
			setQuery((prev) => ({ ...prev, [field]: value }))
		},
		[]
	)

	const handleActivateSavedSearch = useCallback(
		(saved: SavedSearch) => {
			try {
				const parsedQuery = JSON.parse(saved.query_json) as AdvancedSearchQuery
				setQuery(parsedQuery)
				setRawInput(serializeSearchQuery(parsedQuery))
				setHasActiveSearch(true)
				setHistoryOpen(false)
				setPanelOpen(false)
				onSearch(parsedQuery)
				window.dispatchEvent(
					new CustomEvent('postail:activateSavedSearch', {
						detail: { id: saved.id, name: saved.name, query: parsedQuery },
					})
				)
			} catch {
				// malformed query_json
			}
		},
		[onSearch]
	)

	const handleSaveSearch = useCallback(async () => {
		if (!activeAccount?.id || !saveName.trim() || isSaving) return

		const parsed = parseSearchOperators(rawInput)
		const finalQuery: AdvancedSearchQuery = { ...query, ...parsed }

		const isEmpty = Object.values(finalQuery).every((v) => !v)
		if (isEmpty) return

		const queryJson = JSON.stringify(finalQuery)

		setIsSaving(true)
		try {
			await invoke('create_saved_search', {
				accountId: activeAccount.id,
				name: saveName.trim(),
				queryJson,
				icon: 'bookmark',
			})
			await queryClient.invalidateQueries({
				queryKey: ['saved_searches', activeAccount.id],
			})
			setIsSaveMode(false)
			setSaveName('')
		} finally {
			setIsSaving(false)
		}
	}, [activeAccount?.id, saveName, rawInput, query, isSaving, queryClient])

	const handleDeleteSavedSearch = useCallback(
		async (e: React.MouseEvent, saved: SavedSearch) => {
			e.stopPropagation()
			if (!activeAccount?.id) return
			try {
				await invoke('delete_saved_search', { id: saved.id, accountId: activeAccount.id })
				await queryClient.invalidateQueries({
					queryKey: ['saved_searches', activeAccount.id],
				})
			} catch {
				// ignore
			}
		},
		[activeAccount?.id, queryClient]
	)

	const hasAdvancedFilled =
		!!query.from ||
		!!query.to ||
		!!query.subject ||
		!!query.body ||
		!!query.dateFrom ||
		!!query.dateTo ||
		!!query.hasAttachment ||
		!!query.folder

	const motionProps = animationsEnabled
		? { whileHover: { scale: 1.05 }, whileTap: { scale: 0.95 } }
		: {}

	const lastWord = rawInput.split(/\s+/).pop()?.toLowerCase() || ''
	const isTypingOperator =
		lastWord.length > 0 && SEARCH_OPERATORS.some((op) => op.startsWith(lastWord))

	return (
		<div
			ref={containerRef}
			className='relative w-full max-w-xl'
			onMouseDown={(e) => e.stopPropagation()}>
			<Popover
				open={focused && !panelOpen && !hasActiveSearch && isTypingOperator}
				onOpenChange={() => {}}>
				<PopoverAnchor className='absolute top-10 left-0 w-full' />
				<PopoverContent
					className='w-[var(--radix-popover-trigger-width)] p-3 text-sm'
					align='start'
					sideOffset={8}>
					<div className='mb-2 text-[12px] font-semibold text-[var(--text-primary)]'>
						{t('inbox:search.operatorHint')}
					</div>
					<div className='grid grid-cols-2 gap-1.5 text-[11px] text-[var(--text-secondary)]'>
						<div>
							<kbd className='rounded bg-[var(--surface-active)] px-1 font-mono text-[10px] text-[var(--text-primary)]'>
								from:
							</kbd>{' '}
							{t('inbox:search.operators.sender')}
						</div>
						<div>
							<kbd className='rounded bg-[var(--surface-active)] px-1 font-mono text-[10px] text-[var(--text-primary)]'>
								to:
							</kbd>{' '}
							{t('inbox:search.operators.recipient')}
						</div>
						<div>
							<kbd className='rounded bg-[var(--surface-active)] px-1 font-mono text-[10px] text-[var(--text-primary)]'>
								subject:
							</kbd>{' '}
							{t('inbox:search.operators.title')}
						</div>
						<div>
							<kbd className='rounded bg-[var(--surface-active)] px-1 font-mono text-[10px] text-[var(--text-primary)]'>
								has:attachment
							</kbd>
						</div>
						<div>
							<kbd className='rounded bg-[var(--surface-active)] px-1 font-mono text-[10px] text-[var(--text-primary)]'>
								before:
							</kbd>{' '}
							{t('inbox:search.operators.date')}
						</div>
						<div>
							<kbd className='rounded bg-[var(--surface-active)] px-1 font-mono text-[10px] text-[var(--text-primary)]'>
								after:
							</kbd>{' '}
							{t('inbox:search.operators.date')}
						</div>
					</div>
				</PopoverContent>
			</Popover>

			{/* Main input row */}
			<div className='relative flex items-center gap-1.5'>
				<div className='relative flex-1'>
					<div className='pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3'>
						{isSearching ? (
							<Loader2
								className='h-4 w-4 animate-spin'
								style={{ color: accentColor }}
							/>
						) : (
							<motion.div
								animate={
									animationsEnabled
										? {
												scale: focused ? 1.1 : 1,
												color:
													focused || hasActiveSearch
														? accentColor
														: 'var(--text-tertiary)',
											}
										: {}
								}
								transition={{ duration: 0.15 }}>
								<Search className='h-4 w-4' />
							</motion.div>
						)}
					</div>

					<div className='relative w-full'>
						<div
							ref={overlayRef}
							className='pointer-events-none absolute inset-0 z-[1] flex items-center overflow-hidden pr-16 pl-8 text-[13px] whitespace-pre'
							aria-hidden='true'>
							{rawInput.split(SEARCH_SPLIT_REGEX).map((part, i) => {
								const isOp = SEARCH_MATCH_REGEX.test(part)
								return (
									<span
										key={i}
										style={
											isOp
												? { color: accentColor, fontWeight: 600 }
												: { color: 'var(--text-primary)' }
										}>
										{part}
									</span>
								)
							})}
						</div>
						<input
							ref={inputRef}
							type='text'
							data-search-input
							value={rawInput}
							onChange={(e) => handleInputChange(e.target.value)}
							onScroll={(e) => {
								if (overlayRef.current) {
									overlayRef.current.scrollLeft = e.currentTarget.scrollLeft
								}
							}}
							onFocus={() => {
								setFocused(true)
								if (!rawInput.trim() && hasDropdownItems) setHistoryOpen(true)
							}}
							onBlur={() => {
								setFocused(false)
								setTimeout(() => setHistoryOpen(false), 150)
							}}
							onKeyDown={handleKeyDown}
							placeholder={t('inbox:search.placeholder')}
							className='relative z-[2] h-8 w-full rounded-lg border bg-[var(--surface-hover)] pr-16 pl-8 text-[13px] text-transparent transition-all duration-200 outline-none placeholder:text-[var(--text-tertiary)]'
							style={{
								caretColor: 'var(--text-primary)',
								borderColor: hasActiveSearch
									? accentColor
									: focused
										? accentColor
										: 'transparent',
								boxShadow: focused
									? `0 0 0 1px ${accentColor}, 0 4px 16px ${accentColor}1A`
									: hasActiveSearch
										? `0 0 0 1px ${accentColor}66`
										: 'none',
							}}
						/>
					</div>

					{/* History + Saved Searches dropdown */}
					<SearchDropdown
						open={historyOpen && !rawInput.trim() && hasDropdownItems}
						searchHistory={searchHistory}
						savedSearches={savedSearches}
						accentColor={accentColor}
						animationsEnabled={animationsEnabled}
						inputRef={inputRef}
						setRawInput={setRawInput}
						setHistoryOpen={setHistoryOpen}
						clearHistory={clearHistory}
						removeFromHistory={removeFromHistory}
						onActivateSaved={handleActivateSavedSearch}
						onDeleteSaved={handleDeleteSavedSearch}
						t={t}
					/>

					{/* Save + Advanced toggle inside input */}
					<div className='absolute inset-y-0 right-0 flex items-center gap-0.5 pr-1.5'>
						<AnimatePresence>
							{hasActiveSearch && (
								<motion.button
									type='button'
									onClick={() => setIsSaveMode((o) => !o)}
									initial={animationsEnabled ? { opacity: 0, scale: 0.7 } : {}}
									animate={animationsEnabled ? { opacity: 1, scale: 1 } : {}}
									exit={animationsEnabled ? { opacity: 0, scale: 0.7 } : {}}
									transition={{ duration: 0.15 }}
									className='flex h-5 w-5 items-center justify-center rounded-full transition-colors hover:bg-[var(--surface-active)]'
									style={{
										color: isSaveMode ? accentColor : 'var(--text-tertiary)',
									}}>
									<Bookmark className='h-3 w-3' />
								</motion.button>
							)}
						</AnimatePresence>

						<motion.button
							type='button'
							onClick={() => setPanelOpen((o) => !o)}
							{...motionProps}
							className='flex h-6 w-6 items-center justify-center rounded-lg transition-colors'
							style={{
								color:
									panelOpen || hasAdvancedFilled
										? accentColor
										: 'var(--text-tertiary)',
								backgroundColor:
									panelOpen || hasAdvancedFilled
										? `${accentColor}18`
										: 'transparent',
							}}>
							<SlidersHorizontal className='h-3.5 w-3.5' />
						</motion.button>
					</div>
				</div>

				<AnimatePresence>
					{(rawInput || hasActiveSearch) && (
						<motion.button
							type='button'
							onClick={handleClear}
							initial={animationsEnabled ? { opacity: 0, scale: 0.7 } : {}}
							animate={animationsEnabled ? { opacity: 1, scale: 1 } : {}}
							exit={animationsEnabled ? { opacity: 0, scale: 0.7 } : {}}
							transition={{ duration: 0.15 }}
							className='flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
							title={t('inbox:search.actions.clear')}>
							<X className='h-4 w-4' />
						</motion.button>
					)}
				</AnimatePresence>

				{/* Save-mode name input dropdown */}
				<AnimatePresence>
					{isSaveMode && (
						<motion.div
							key='save-dropdown'
							initial={animationsEnabled ? { opacity: 0, y: -4, scale: 0.98 } : {}}
							animate={animationsEnabled ? { opacity: 1, y: 0, scale: 1 } : {}}
							exit={animationsEnabled ? { opacity: 0, y: -4, scale: 0.98 } : {}}
							transition={{ duration: 0.15, ease: EASE_OUT_EXPO }}
							className='glass absolute top-[calc(100%+4px)] right-0 z-50 flex items-center gap-1.5 rounded-xl border border-[var(--border-subtle)] px-2 py-2 shadow-xl backdrop-blur-xl'
							style={{
								boxShadow: `0 8px 32px rgba(0,0,0,0.2), 0 0 0 1px var(--border-subtle)`,
							}}
							onMouseDown={(e) => e.stopPropagation()}>
							<Bookmark
								className='h-3.5 w-3.5 shrink-0'
								style={{ color: accentColor }}
							/>
							<input
								ref={saveInputRef}
								type='text'
								value={saveName}
								onChange={(e) => setSaveName(e.target.value)}
								onKeyDown={(e) => {
									if (e.key === 'Enter') handleSaveSearch()
									if (e.key === 'Escape') {
										setIsSaveMode(false)
										setSaveName('')
									}
								}}
								placeholder={t('inbox:search.savedSearches.namePlaceholder')}
								className='h-7 w-40 bg-transparent pl-1 text-xs text-[var(--text-primary)] placeholder-[var(--text-tertiary)] focus:outline-none'
							/>
							<button
								type='button'
								onClick={handleSaveSearch}
								disabled={!saveName.trim() || isSaving}
								className='flex h-6 w-6 shrink-0 items-center justify-center rounded-lg transition-colors disabled:opacity-40'
								style={{ backgroundColor: accentColor, color: 'white' }}>
								{isSaving ? (
									<Loader2 className='h-3 w-3 animate-spin' />
								) : (
									<Check className='h-3 w-3' />
								)}
							</button>
							<button
								type='button'
								onClick={() => {
									setIsSaveMode(false)
									setSaveName('')
								}}
								className='flex h-6 w-6 shrink-0 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-colors hover:text-[var(--text-primary)]'>
								<X className='h-3 w-3' />
							</button>
						</motion.div>
					)}
				</AnimatePresence>
			</div>

			{/* Advanced panel */}
			<SearchPanel
				open={panelOpen}
				query={query}
				accentColor={accentColor}
				animationsEnabled={animationsEnabled}
				mailboxes={mailboxes}
				updateField={updateField}
				onClear={handleClear}
				onSubmit={handleSubmit}
				t={t}
			/>
		</div>
	)
}
