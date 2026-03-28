import { useState, useEffect, useRef, useCallback } from 'react'
import { motion, AnimatePresence, type Variants } from 'framer-motion'
import {
	Keyboard,
	Globe,
	PenLine,
	Inbox,
	ChevronDown,
	RotateCcw,
	AlertTriangle,
	Check,
} from 'lucide-react'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { dispatchShortcutsUpdated } from '@/hooks/useShortcutKeys'
import {
	defaultShortcuts,
	shortcutDescriptions,
	formatShortcutKey,
	loadShortcutOverrides,
	saveShortcutOverride,
	resetShortcutOverride,
	resetAllShortcutOverrides,
	eventToShortcutKey,
	type ShortcutDefinition,
} from '@/config/shortcuts'

// ─── Key Badge ────────────────────────────────────────────────────────────────

function KeyBadge({ part, accent = false }: { part: string; accent?: boolean }) {
	const accentColor = useThemeStore((s) => s.accentColor)

	return (
		<span
			className='inline-flex h-6 min-w-[1.5rem] items-center justify-center rounded-md px-1.5 font-mono text-[11px] leading-none font-semibold tracking-tight ring-1 ring-inset'
			style={
				accent
					? {
							backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
							color: accentColor,
							boxShadow: `inset 0 1px 0 rgba(255,255,255,0.06), 0 1px 2px rgba(0,0,0,0.2)`,
						}
					: {
							backgroundColor: 'var(--surface-active)',
							color: 'var(--text-primary)',
							boxShadow: `inset 0 1px 0 rgba(255,255,255,0.06), 0 1px 2px rgba(0,0,0,0.2)`,
						}
			}>
			{part}
		</span>
	)
}

function ShortcutKeys({ keyStr, accent = false }: { keyStr: string; accent?: boolean }) {
	const primary = keyStr.split(',')[0].trim()
	const formatted = formatShortcutKey(primary)
	const parts = formatted.split('+')

	return (
		<div className='flex items-center gap-0.5'>
			{parts.map((part, i) => (
				<span key={i} className='flex items-center gap-0.5'>
					{i > 0 && (
						<span className='mx-0.5 text-[10px] text-[var(--text-tertiary)] select-none'>
							+
						</span>
					)}
					<KeyBadge
						part={part}
						accent={accent || ['Ctrl', 'Shift', 'Alt'].includes(part)}
					/>
				</span>
			))}
		</div>
	)
}

// ─── Recording Overlay ────────────────────────────────────────────────────────

function RecordingInput({
	onCapture,
	onCancel,
}: {
	onCapture: (key: string) => void
	onCancel: () => void
}) {
	const accentColor = useThemeStore((s) => s.accentColor)
	const [captured, setCaptured] = useState<string | null>(null)
	const ref = useRef<HTMLDivElement>(null)

	useEffect(() => {
		ref.current?.focus()
	}, [])

	useEffect(() => {
		const handler = (e: KeyboardEvent) => {
			e.preventDefault()
			e.stopPropagation()

			if (e.key === 'Escape') {
				onCancel()
				return
			}

			const combo = eventToShortcutKey(e)
			if (combo) setCaptured(combo)
		}

		const upHandler = (e: KeyboardEvent) => {
			const combo = eventToShortcutKey(e)
			if (combo && combo === captured) {
				onCapture(combo)
			}
		}

		window.addEventListener('keydown', handler, true)
		window.addEventListener('keyup', upHandler, true)
		return () => {
			window.removeEventListener('keydown', handler, true)
			window.removeEventListener('keyup', upHandler, true)
		}
	}, [captured, onCapture, onCancel])

	return (
		<motion.div
			ref={ref}
			tabIndex={-1}
			initial={{ opacity: 0, scale: 0.95 }}
			animate={{ opacity: 1, scale: 1 }}
			exit={{ opacity: 0, scale: 0.95 }}
			transition={{ duration: 0.15, ease: 'easeOut' }}
			className='flex items-center gap-3 rounded-xl border px-4 py-2.5 outline-none'
			style={{
				borderColor: accentColor,
				backgroundColor: `rgba(var(--accent-rgb), 0.06)`,
				boxShadow: `0 0 0 3px rgba(var(--accent-rgb), 0.12)`,
			}}>
			<motion.div
				animate={{ opacity: [1, 0.3, 1] }}
				transition={{ duration: 1, repeat: Infinity, ease: 'easeInOut' }}
				className='h-2 w-2 rounded-full'
				style={{ backgroundColor: accentColor }}
			/>
			{captured ? (
				<ShortcutKeys keyStr={captured} accent />
			) : (
				<span className='text-xs text-[var(--text-secondary)]'>Press a key combo…</span>
			)}
			<span className='ml-auto text-[10px] text-[var(--text-tertiary)]'>Esc to cancel</span>
		</motion.div>
	)
}

// ─── Shortcut Row ──────────────────────────────────────────────────────────────

function ShortcutRow({
	shortcut,
	index,
	overrideKey,
	conflict,
	onEdit,
	onReset,
}: {
	shortcut: ShortcutDefinition
	index: number
	overrideKey: string | undefined
	conflict: boolean
	onEdit: () => void
	onReset: () => void
}) {
	const animationsEnabled = useAnimationsEnabled()
	const accentColor = useThemeStore((s) => s.accentColor)
	const description = shortcutDescriptions[shortcut.action] ?? shortcut.action
	const currentKey = overrideKey ?? shortcut.key
	const isModified = !!overrideKey

	return (
		<motion.div
			{...(animationsEnabled
				? {
						initial: { opacity: 0, x: -8 },
						animate: { opacity: 1, x: 0 },
						transition: {
							duration: 0.18,
							delay: index * 0.025,
							ease: 'easeOut' as const,
						},
					}
				: {})}
			className={`group relative flex items-center gap-3 overflow-hidden rounded-xl border px-4 py-3 transition-all duration-200 ${
				conflict
					? 'border-red-500/40 bg-red-500/5'
					: isModified
						? 'border-[var(--border-subtle)] bg-[var(--surface-panel)]'
						: 'border-[var(--border-faint)] bg-[var(--surface-panel)] hover:border-[var(--border-subtle)] hover:bg-[var(--surface-hover)]'
			}`}>
			{/* Accent line for modified */}
			{isModified && !conflict && (
				<div
					className='absolute top-0 left-0 h-full w-[2px] rounded-l-xl'
					style={{ backgroundColor: accentColor }}
				/>
			)}

			{/* Description */}
			<span className='relative z-10 min-w-0 flex-1 text-sm text-[var(--text-primary)]'>
				{description}
			</span>

			{/* Conflict warning */}
			{conflict && (
				<span className='flex items-center gap-1 text-[11px] font-medium text-red-400'>
					<AlertTriangle className='h-3 w-3' />
					Conflict
				</span>
			)}

			{/* Key display / edit button */}
			<button
				type='button'
				onClick={() => onEdit()}
				className='group/btn relative flex items-center gap-1.5 rounded-lg px-2 py-1 transition-all duration-150 hover:bg-[var(--surface-hover)]'
				title='Click to rebind'>
				<ShortcutKeys keyStr={currentKey} />
				<span className='text-[10px] text-[var(--text-tertiary)] opacity-0 transition-opacity duration-150 group-hover/btn:opacity-100'>
					edit
				</span>
			</button>

			{/* Reset button */}
			<AnimatePresence>
				{isModified && (
					<motion.button
						initial={{ opacity: 0, width: 0 }}
						animate={{ opacity: 1, width: 'auto' }}
						exit={{ opacity: 0, width: 0 }}
						transition={{ duration: 0.15 }}
						type='button'
						onClick={() => onReset()}
						className='flex items-center justify-center rounded-lg p-1.5 text-[var(--text-tertiary)] transition-colors duration-150 hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
						title='Reset to default'>
						<RotateCcw className='h-3.5 w-3.5' />
					</motion.button>
				)}
			</AnimatePresence>
		</motion.div>
	)
}

// ─── Scope Section ─────────────────────────────────────────────────────────────

const SCOPE_META = {
	global: { label: 'Global', icon: Globe, description: 'Available everywhere in the app' },
	compose: { label: 'Compose', icon: PenLine, description: 'Active when composing a message' },
	inbox: { label: 'Inbox', icon: Inbox, description: 'Active when browsing your inbox' },
} as const

function ScopeSection({
	scope,
	shortcuts,
	overrides,
	editingAction,
	conflicts,
	defaultOpen,
	onEdit,
	onCapture,
	onCancelEdit,
	onReset,
}: {
	scope: 'global' | 'compose' | 'inbox'
	shortcuts: ShortcutDefinition[]
	overrides: Record<string, string>
	editingAction: string | null // format: "scope:action"
	conflicts: Set<string>
	defaultOpen: boolean
	onEdit: (scopedKey: string) => void
	onCapture: (action: string, key: string) => void
	onCancelEdit: () => void
	onReset: (scopedKey: string) => void
}) {
	const [open, setOpen] = useState(defaultOpen)
	const accentColor = useThemeStore((s) => s.accentColor)
	const meta = SCOPE_META[scope]
	const Icon = meta.icon
	const modifiedCount = shortcuts.filter((s) => overrides[`${scope}:${s.action}`]).length
	const conflictCount = shortcuts.filter((s) => conflicts.has(`${scope}:${s.action}`)).length

	return (
		<div className='overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--surface-panel)]'>
			<button
				type='button'
				onClick={() => setOpen((v) => !v)}
				className='group flex w-full items-center gap-4 px-5 py-4 transition-colors duration-200 hover:bg-[var(--surface-hover)]'>
				<div
					className='flex h-9 w-9 shrink-0 items-center justify-center rounded-xl ring-1 ring-[var(--border-faint)] transition-all duration-300 group-hover:ring-[var(--border-subtle)]'
					style={{ backgroundColor: `rgba(var(--accent-rgb), 0.06)` }}>
					<Icon className='h-4 w-4' style={{ color: accentColor }} />
				</div>
				<div className='flex-1 text-left'>
					<p className='text-sm font-semibold text-[var(--text-primary)]'>{meta.label}</p>
					<p className='text-xs text-[var(--text-secondary)]'>{meta.description}</p>
				</div>
				<div className='flex items-center gap-2'>
					{conflictCount > 0 && (
						<span className='flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-bold text-red-400 ring-1 ring-red-500/30'>
							<AlertTriangle className='h-3 w-3' />
							{conflictCount}
						</span>
					)}
					{modifiedCount > 0 && (
						<span
							className='rounded-full px-2.5 py-0.5 text-[11px] font-bold'
							style={{
								backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
								color: accentColor,
							}}>
							{modifiedCount} custom
						</span>
					)}
					<span className='rounded-full px-2.5 py-0.5 text-[11px] font-bold text-[var(--text-tertiary)] ring-1 ring-[var(--border-faint)]'>
						{shortcuts.length}
					</span>
					<motion.div
						animate={{ rotate: open ? 180 : 0 }}
						transition={{ duration: 0.2, ease: 'easeInOut' }}>
						<ChevronDown className='h-4 w-4 text-[var(--text-tertiary)]' />
					</motion.div>
				</div>
			</button>

			<AnimatePresence initial={false}>
				{open && (
					<motion.div
						key='content'
						initial={{ height: 0, opacity: 0 }}
						animate={{ height: 'auto', opacity: 1 }}
						exit={{ height: 0, opacity: 0 }}
						transition={{ duration: 0.25, ease: 'easeInOut' }}
						className='overflow-hidden'>
						<div className='space-y-1.5 border-t border-[var(--border-faint)] px-4 py-4'>
							{shortcuts.map((s, i) =>
								editingAction === `${scope}:${s.action}` ? (
									<RecordingInput
										key={`${scope}:${s.action}`}
										onCapture={(key) => onCapture(`${scope}:${s.action}`, key)}
										onCancel={onCancelEdit}
									/>
								) : (
									<ShortcutRow
										key={`${scope}:${s.action}`}
										shortcut={s}
										index={i}
										overrideKey={overrides[`${scope}:${s.action}`]}
										conflict={conflicts.has(`${scope}:${s.action}`)}
										onEdit={() => onEdit(`${scope}:${s.action}`)}
										onReset={() => onReset(`${scope}:${s.action}`)}
									/>
								)
							)}
						</div>
					</motion.div>
				)}
			</AnimatePresence>
		</div>
	)
}

// ─── Main Component ────────────────────────────────────────────────────────────

const containerVariants: Variants = {
	hidden: {},
	show: { transition: { staggerChildren: 0.07, delayChildren: 0.05 } },
}

const itemVariants: Variants = {
	hidden: { opacity: 0, y: 12 },
	show: { opacity: 1, y: 0, transition: { duration: 0.28, ease: 'easeOut' } },
}

export function KeyboardShortcutsSettings() {
	const animationsEnabled = useAnimationsEnabled()
	const [overrides, setOverrides] = useState<Record<string, string>>(() =>
		loadShortcutOverrides()
	)
	const [editingAction, setEditingAction] = useState<string | null>(null)
	const [savedBadge, setSavedBadge] = useState(false)

	// Detect conflicts
	const conflicts = useCallback(() => {
		const result = new Set<string>()
		const keyMap: Record<string, string[]> = {}

		for (const s of defaultShortcuts) {
			const scopedKey = `${s.scope}:${s.action}`
			const key = overrides[scopedKey] ?? s.key
			const primary = key.split(',')[0].trim()
			const mapKey = `${s.scope}:${primary}`
			if (!keyMap[mapKey]) keyMap[mapKey] = []
			keyMap[mapKey].push(scopedKey)
		}

		for (const scopedActions of Object.values(keyMap)) {
			if (scopedActions.length > 1) scopedActions.forEach((a) => result.add(a))
		}

		return result
	}, [overrides])

	const handleEdit = (scopedKey: string) => setEditingAction(scopedKey)
	const handleCancelEdit = () => setEditingAction(null)

	const handleCapture = (scopedKey: string, key: string) => {
		saveShortcutOverride(scopedKey, key)
		setOverrides(loadShortcutOverrides())
		setEditingAction(null)
		setSavedBadge(true)
		dispatchShortcutsUpdated()
		setTimeout(() => setSavedBadge(false), 1800)
	}

	const handleReset = (scopedKey: string) => {
		resetShortcutOverride(scopedKey)
		setOverrides(loadShortcutOverrides())
		dispatchShortcutsUpdated()
	}

	const handleResetAll = () => {
		resetAllShortcutOverrides()
		setOverrides({})
		setSavedBadge(true)
		dispatchShortcutsUpdated()
		setTimeout(() => setSavedBadge(false), 1800)
	}

	const currentConflicts = conflicts()
	const totalModified = Object.keys(overrides).length

	const byScope = {
		global: defaultShortcuts.filter((s) => s.scope === 'global'),
		compose: defaultShortcuts.filter((s) => s.scope === 'compose'),
		inbox: defaultShortcuts.filter((s) => s.scope === 'inbox'),
	}

	return (
		<div className='h-full overflow-y-auto'>
			<motion.div
				{...(animationsEnabled
					? { variants: containerVariants, initial: 'hidden', animate: 'show' }
					: {})}
				className='mx-auto max-w-2xl space-y-6 p-8'>
				{/* Header */}
				<motion.div
					{...(animationsEnabled ? { variants: itemVariants } : {})}
					className='flex items-start justify-between gap-4'>
					<div className='space-y-1'>
						<div className='flex items-center gap-3'>
							<Keyboard className='h-5 w-5 text-[var(--text-secondary)]' />
							<h1 className='text-lg font-bold text-[var(--text-primary)]'>
								Keyboard Shortcuts
							</h1>
						</div>
						<p className='ml-8 text-sm text-[var(--text-secondary)]'>
							Click any shortcut to rebind it. Press Esc to cancel.
						</p>
					</div>

					{/* Status + Reset all */}
					<div className='flex shrink-0 items-center gap-2'>
						<AnimatePresence>
							{savedBadge && (
								<motion.span
									initial={{ opacity: 0, y: 4 }}
									animate={{ opacity: 1, y: 0 }}
									exit={{ opacity: 0, y: -4 }}
									transition={{ duration: 0.2 }}
									className='flex items-center gap-1 rounded-full px-2.5 py-1 text-[11px] font-semibold text-green-400 ring-1 ring-green-500/30'>
									<Check className='h-3 w-3' />
									Saved
								</motion.span>
							)}
						</AnimatePresence>

						{totalModified > 0 && (
							<button
								type='button'
								onClick={handleResetAll}
								className='flex items-center gap-1.5 rounded-xl border border-[var(--border-subtle)] px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)] transition-all duration-150 hover:border-red-500/40 hover:bg-red-500/5 hover:text-red-400'>
								<RotateCcw className='h-3 w-3' />
								Reset all ({totalModified})
							</button>
						)}
					</div>
				</motion.div>

				{/* Conflict banner */}
				<AnimatePresence>
					{currentConflicts.size > 0 && (
						<motion.div
							initial={{ opacity: 0, height: 0 }}
							animate={{ opacity: 1, height: 'auto' }}
							exit={{ opacity: 0, height: 0 }}
							transition={{ duration: 0.2 }}
							className='flex items-center gap-3 overflow-hidden rounded-2xl border border-red-500/30 bg-red-500/5 px-5 py-4'>
							<AlertTriangle className='h-4 w-4 shrink-0 text-red-400' />
							<p className='text-sm text-red-400'>
								<span className='font-semibold'>
									{currentConflicts.size} shortcut
									{currentConflicts.size > 1 ? 's' : ''} conflict.
								</span>{' '}
								Multiple actions share the same key in the same scope.
							</p>
						</motion.div>
					)}
				</AnimatePresence>

				{/* Scope sections */}
				{(['global', 'compose', 'inbox'] as const).map((scope, i) => (
					<motion.div
						key={scope}
						{...(animationsEnabled ? { variants: itemVariants } : {})}>
						<ScopeSection
							scope={scope}
							shortcuts={byScope[scope]}
							overrides={overrides}
							editingAction={editingAction}
							conflicts={currentConflicts}
							defaultOpen={i === 0}
							onEdit={handleEdit}
							onCapture={handleCapture}
							onCancelEdit={handleCancelEdit}
							onReset={handleReset}
						/>
					</motion.div>
				))}
			</motion.div>
		</div>
	)
}
