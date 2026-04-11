import { useState, useRef, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion, AnimatePresence } from 'framer-motion'
import {
	Search,
	SlidersHorizontal,
	X,
	Calendar,
	Paperclip,
	User,
	AtSign,
	Type,
	AlignLeft,
	FolderOpen,
	Loader2,
	ChevronDown,
	Clock,
	Bookmark,
	Trash2,
	Check,
} from 'lucide-react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useThemeStore } from '@/stores/themeStore'
import { useAccountStore } from '@/stores/accountStore'
import type { Mailbox } from '@/types/mail'
import { parseSearchOperators, SEARCH_OPERATORS } from '@/lib/searchQueryParser'
import { Popover, PopoverContent, PopoverAnchor } from '@/components/ui/popover'
import { useSearchHistory } from '@/hooks/useSearchHistory'
import type { AdvancedSearchQuery, SavedSearch } from '@/types/search'

interface SearchBarProps {
	onSearch: (query: AdvancedSearchQuery | null) => void
	isSearching?: boolean
}

const EASE_OUT_EXPO: [number, number, number, number] = [0.16, 1, 0.3, 1]

export function SearchBar({ onSearch, isSearching }: SearchBarProps) {
	const { t } = useTypedTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const { activeAccount } = useAccountStore()
	const queryClient = useQueryClient()

	const [focused, setFocused] = useState(false)
	const [rawInput, setRawInput] = useState('')
	const [panelOpen, setPanelOpen] = useState(false)
	const [query, setQuery] = useState<AdvancedSearchQuery>({})
	const [hasActiveSearch, setHasActiveSearch] = useState(false)
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
		if (isSaveMode) {
			setTimeout(() => saveInputRef.current?.focus(), 50)
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

	const handleSubmit = useCallback(() => {
		const parsed = parseSearchOperators(rawInput)
		const finalQuery: AdvancedSearchQuery = {
			...query,
			...parsed,
		}

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
		addToHistory(rawInput.trim())
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
				setRawInput(saved.name)
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
					onOpenAutoFocus={(e) => e.preventDefault()}
					sideOffset={8}>
					<div className='mb-2 font-medium text-slate-900 dark:text-white'>
						{t('inbox:search.operatorHint')}
					</div>
					<div className='grid grid-cols-2 gap-2 text-slate-500 dark:text-slate-400'>
						<div>
							<kbd className='rounded bg-slate-100 px-1 font-mono text-xs dark:bg-slate-800'>
								from:
							</kbd>{' '}
							{t('inbox:search.operators.sender')}
						</div>
						<div>
							<kbd className='rounded bg-slate-100 px-1 font-mono text-xs dark:bg-slate-800'>
								to:
							</kbd>{' '}
							{t('inbox:search.operators.recipient')}
						</div>
						<div>
							<kbd className='rounded bg-slate-100 px-1 font-mono text-xs dark:bg-slate-800'>
								subject:
							</kbd>{' '}
							{t('inbox:search.operators.title')}
						</div>
						<div>
							<kbd className='rounded bg-slate-100 px-1 font-mono text-xs dark:bg-slate-800'>
								has:attachment
							</kbd>
						</div>
						<div>
							<kbd className='rounded bg-slate-100 px-1 font-mono text-xs dark:bg-slate-800'>
								before:
							</kbd>{' '}
							{t('inbox:search.operators.date')}
						</div>
						<div>
							<kbd className='rounded bg-slate-100 px-1 font-mono text-xs dark:bg-slate-800'>
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

					<input
						ref={inputRef}
						type='text'
						data-search-input
						value={rawInput}
						onChange={(e) => handleInputChange(e.target.value)}
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
						className='h-9 w-full rounded-xl border bg-slate-100/50 pr-16 pl-9 text-sm text-slate-900 placeholder-slate-500 transition-all duration-300 focus:bg-white focus:outline-none dark:bg-white/5 dark:text-white dark:placeholder-slate-500 dark:focus:bg-slate-800/80'
						style={{
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

					{/* History + Saved Searches dropdown */}
					<AnimatePresence>
						{historyOpen && !rawInput && hasDropdownItems && (
							<motion.div
								key='history'
								initial={
									animationsEnabled ? { opacity: 0, y: -6, scale: 0.98 } : {}
								}
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
											<div
												key={q}
												className='group flex items-center gap-2 px-2'>
												<button
													type='button'
													onMouseDown={(e) => {
														e.preventDefault()
														setRawInput(q)
														setHistoryOpen(false)
														setTimeout(
															() => inputRef.current?.focus(),
															0
														)
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

								{/* Separator between history and saved searches */}
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
											<div
												key={saved.id}
												className='group flex items-center gap-2 px-2'>
												<button
													type='button'
													onMouseDown={(e) => {
														e.preventDefault()
														handleActivateSavedSearch(saved)
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
													onMouseDown={(e) =>
														handleDeleteSavedSearch(e, saved)
													}
													className='hidden h-5 w-5 shrink-0 items-center justify-center rounded text-[var(--text-tertiary)] transition-colors group-hover:flex hover:text-red-400'>
													<Trash2 className='h-3 w-3' />
												</button>
											</div>
										))}
									</>
								)}
							</motion.div>
						)}
					</AnimatePresence>

					{/* Clear + Save + Advanced toggle inside input */}
					<div className='absolute inset-y-0 right-0 flex items-center gap-0.5 pr-1.5'>
						<AnimatePresence>
							{(rawInput || hasActiveSearch) && (
								<motion.button
									type='button'
									onClick={handleClear}
									initial={animationsEnabled ? { opacity: 0, scale: 0.7 } : {}}
									animate={animationsEnabled ? { opacity: 1, scale: 1 } : {}}
									exit={animationsEnabled ? { opacity: 0, scale: 0.7 } : {}}
									transition={{ duration: 0.15 }}
									className='flex h-5 w-5 items-center justify-center rounded-full text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-active)] hover:text-[var(--text-primary)]'>
									<X className='h-3 w-3' />
								</motion.button>
							)}
						</AnimatePresence>

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
			<AnimatePresence>
				{panelOpen && (
					<motion.div
						key='advanced-panel'
						initial={
							animationsEnabled
								? { opacity: 0, y: -8, scale: 0.97, filter: 'blur(4px)' }
								: {}
						}
						animate={
							animationsEnabled
								? { opacity: 1, y: 0, scale: 1, filter: 'blur(0px)' }
								: {}
						}
						exit={
							animationsEnabled
								? { opacity: 0, y: -8, scale: 0.97, filter: 'blur(4px)' }
								: {}
						}
						transition={{ duration: 0.22, ease: EASE_OUT_EXPO }}
						className='glass absolute top-[calc(100%+6px)] right-0 left-0 z-50 rounded-2xl border border-[var(--border-subtle)] p-4 shadow-2xl backdrop-blur-xl'
						style={{
							boxShadow: `0 20px 60px rgba(0,0,0,0.5), 0 0 0 1px var(--border-subtle)`,
							backgroundImage: `linear-gradient(to bottom, ${accentColor}0A, ${accentColor}1A)`,
						}}
						onMouseDown={(e) => e.stopPropagation()}>
						{/* Accent top bar */}
						<div
							className='absolute inset-x-0 top-0 h-[2px] rounded-t-2xl'
							style={{
								background: `linear-gradient(90deg, transparent, ${accentColor}, transparent)`,
							}}
						/>

						<div className='grid grid-cols-2 gap-3'>
							<PanelField
								icon={<User className='h-3.5 w-3.5' />}
								label={t('inbox:search.fields.from')}
								value={query.from ?? ''}
								onChange={(v) => updateField('from', v || undefined)}
								placeholder='sender@example.com'
								accentColor={accentColor}
							/>
							<PanelField
								icon={<AtSign className='h-3.5 w-3.5' />}
								label={t('inbox:search.fields.to')}
								value={query.to ?? ''}
								onChange={(v) => updateField('to', v || undefined)}
								placeholder='recipient@example.com'
								accentColor={accentColor}
							/>
							<PanelField
								icon={<Type className='h-3.5 w-3.5' />}
								label={t('inbox:search.fields.subject')}
								value={query.subject ?? ''}
								onChange={(v) => updateField('subject', v || undefined)}
								placeholder={t('inbox:search.fields.subject')}
								accentColor={accentColor}
								className='col-span-2'
							/>
							<PanelField
								icon={<AlignLeft className='h-3.5 w-3.5' />}
								label={t('inbox:search.fields.body')}
								value={query.body ?? ''}
								onChange={(v) => updateField('body', v || undefined)}
								placeholder={t('inbox:search.fields.body')}
								accentColor={accentColor}
								className='col-span-2'
							/>
							<PanelField
								icon={<Calendar className='h-3.5 w-3.5' />}
								label={t('inbox:search.fields.dateFrom')}
								value={query.dateFrom ?? ''}
								onChange={(v) => updateField('dateFrom', v || undefined)}
								type='date'
								accentColor={accentColor}
							/>
							<PanelField
								icon={<Calendar className='h-3.5 w-3.5' />}
								label={t('inbox:search.fields.dateTo')}
								value={query.dateTo ?? ''}
								onChange={(v) => updateField('dateTo', v || undefined)}
								type='date'
								accentColor={accentColor}
							/>

							{/* Folder select */}
							<div className='flex flex-col gap-1'>
								<label className='flex items-center gap-1.5 text-[11px] font-medium text-[var(--text-secondary)]'>
									<FolderOpen className='h-3.5 w-3.5' />
									{t('inbox:search.fields.folder')}
								</label>
								<div className='relative flex items-center'>
									<select
										value={query.folder ?? ''}
										onChange={(e) =>
											updateField('folder', e.target.value || undefined)
										}
										onFocus={(e) => {
											e.currentTarget.style.borderColor = accentColor
											e.currentTarget.style.boxShadow = `0 0 0 1px ${accentColor}`
										}}
										onBlur={(e) => {
											e.currentTarget.style.borderColor =
												'var(--border-subtle)'
											e.currentTarget.style.boxShadow = 'none'
										}}
										className='h-8 w-full appearance-none rounded-lg border bg-[var(--surface-secondary)] px-3 pr-8 text-xs text-[var(--text-primary)] transition-all focus:outline-none'
										style={{
											borderColor: 'var(--border-subtle)',
											backgroundColor: 'var(--surface-secondary)',
											color: 'var(--text-primary)',
										}}>
										<option value=''>
											{t('inbox:search.fields.allFolders')}
										</option>
										{mailboxes?.map((mb) => (
											<option key={mb.name} value={mb.name}>
												{mb.display_name || mb.name}
											</option>
										))}
									</select>
									<ChevronDown className='pointer-events-none absolute right-2.5 h-3.5 w-3.5 text-[var(--text-tertiary)]' />
								</div>
							</div>

							{/* Has attachment */}
							<div className='flex flex-col gap-1'>
								<label className='flex items-center gap-1.5 text-[11px] font-medium text-[var(--text-secondary)]'>
									<Paperclip className='h-3.5 w-3.5' />
									{t('inbox:search.fields.hasAttachment')}
								</label>
								<button
									type='button'
									onClick={() =>
										updateField(
											'hasAttachment',
											query.hasAttachment ? undefined : true
										)
									}
									className='flex h-8 items-center gap-2 rounded-lg border px-3 text-xs transition-all'
									style={{
										borderColor: query.hasAttachment
											? accentColor
											: 'var(--border-subtle)',
										backgroundColor: query.hasAttachment
											? `${accentColor}18`
											: 'var(--surface-secondary)',
										color: query.hasAttachment
											? accentColor
											: 'var(--text-secondary)',
									}}>
									<div
										className='flex h-3.5 w-3.5 items-center justify-center rounded-full border-2 transition-all'
										style={{
											borderColor: query.hasAttachment
												? accentColor
												: 'var(--border-strong)',
										}}>
										{query.hasAttachment && (
											<div
												className='h-2 w-2 rounded-full'
												style={{ backgroundColor: accentColor }}
											/>
										)}
									</div>
									{t('inbox:search.fields.hasAttachment')}
								</button>
							</div>
						</div>

						{/* Panel footer */}
						<div className='mt-3 flex items-center justify-between border-t border-[var(--border-subtle)] pt-3'>
							<button
								type='button'
								onClick={() => {
									setQuery({})
									setRawInput('')
								}}
								className='text-xs text-[var(--text-tertiary)] transition-colors hover:text-[var(--text-secondary)]'>
								{t('inbox:search.actions.clear')}
							</button>

							<div className='flex items-center gap-2'>
								<motion.button
									type='button'
									onClick={handleSubmit}
									{...motionProps}
									className='flex h-8 items-center gap-2 rounded-xl px-4 text-xs font-semibold text-white transition-all'
									style={{ backgroundColor: accentColor }}>
									<Search className='h-3.5 w-3.5' />
									{t('inbox:search.actions.search')}
								</motion.button>
							</div>
						</div>
					</motion.div>
				)}
			</AnimatePresence>
		</div>
	)
}

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

function PanelField({
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
