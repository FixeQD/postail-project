import { useState } from 'react'
import { motion, AnimatePresence, type Variants } from 'framer-motion'
import { Keyboard, Globe, PenLine, Inbox, ChevronDown } from 'lucide-react'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import {
	defaultShortcuts,
	shortcutDescriptions,
	formatShortcutKey,
	type ShortcutDefinition,
} from '@/config/shortcuts'

// ─── Key Badge ────────────────────────────────────────────────────────────────

function KeyBadge({ part }: { part: string }) {
	const accentColor = useThemeStore((s) => s.accentColor)
	const isModifier = ['Ctrl', 'Shift', 'Alt'].includes(part)

	return (
		<span
			className='inline-flex h-6 min-w-[1.5rem] items-center justify-center rounded-md px-1.5 font-mono text-[11px] leading-none font-semibold tracking-tight ring-1 transition-all duration-200 ring-inset'
			style={
				isModifier
					? {
							backgroundColor: `rgba(var(--accent-rgb), 0.08)`,
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

function ShortcutKeys({ keyStr }: { keyStr: string }) {
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
					<KeyBadge part={part} />
				</span>
			))}
		</div>
	)
}

// ─── Shortcut Row ──────────────────────────────────────────────────────────────

function ShortcutRow({ shortcut, index }: { shortcut: ShortcutDefinition; index: number }) {
	const animationsEnabled = useAnimationsEnabled()
	const description = shortcutDescriptions[shortcut.action] ?? shortcut.action

	return (
		<motion.div
			{...(animationsEnabled
				? {
						initial: { opacity: 0, x: -8 },
						animate: { opacity: 1, x: 0 },
						transition: {
							duration: 0.2,
							delay: index * 0.03,
							ease: 'easeOut' as const,
						},
					}
				: {})}
			className='group relative flex items-center justify-between gap-4 overflow-hidden rounded-xl border border-[var(--border-faint)] bg-[var(--surface-panel)] px-4 py-3 transition-all duration-200 hover:border-[var(--border-subtle)] hover:bg-[var(--surface-hover)]'>
			{animationsEnabled && (
				<div
					className='pointer-events-none absolute inset-x-0 bottom-0 h-px opacity-0 transition-opacity duration-300 group-hover:opacity-100'
					style={{
						background: `linear-gradient(90deg, transparent, rgba(var(--accent-rgb), 0.3), transparent)`,
					}}
				/>
			)}
			<span className='group-hover:text-foreground relative z-10 text-sm text-[var(--text-primary)] transition-colors duration-200'>
				{description}
			</span>
			<div className='relative z-10 flex shrink-0 items-center gap-1'>
				<ShortcutKeys keyStr={shortcut.key} />
			</div>
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
	defaultOpen = true,
}: {
	scope: 'global' | 'compose' | 'inbox'
	shortcuts: ShortcutDefinition[]
	defaultOpen?: boolean
}) {
	const [open, setOpen] = useState(defaultOpen)
	const animationsEnabled = useAnimationsEnabled()
	const accentColor = useThemeStore((s) => s.accentColor)
	const meta = SCOPE_META[scope]
	const Icon = meta.icon

	return (
		<div className='overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--surface-panel)]'>
			<button
				type='button'
				onClick={() => setOpen((v) => !v)}
				className='group flex w-full items-center gap-4 px-5 py-4 transition-colors duration-200 hover:bg-[var(--surface-hover)]'>
				<div
					className='flex h-9 w-9 shrink-0 items-center justify-center rounded-xl ring-1 ring-[var(--border-faint)] transition-all duration-300 group-hover:ring-[var(--border-subtle)]'
					style={{ backgroundColor: `rgba(var(--accent-rgb), 0.06)` }}>
					<Icon
						className='h-4 w-4 transition-colors duration-200'
						style={{ color: accentColor }}
					/>
				</div>
				<div className='flex-1 text-left'>
					<p className='text-sm font-semibold text-[var(--text-primary)]'>{meta.label}</p>
					<p className='text-xs text-[var(--text-secondary)]'>{meta.description}</p>
				</div>
				<div className='flex items-center gap-3'>
					<span
						className='rounded-full px-2.5 py-0.5 text-[11px] font-bold'
						style={{
							backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
							color: accentColor,
						}}>
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
							{shortcuts.map((s, i) => (
								<ShortcutRow key={s.action} shortcut={s} index={i} />
							))}
						</div>
					</motion.div>
				)}
			</AnimatePresence>
		</div>
	)
}

// ─── Coming Soon Banner ────────────────────────────────────────────────────────

function ComingSoonBanner() {
	const accentColor = useThemeStore((s) => s.accentColor)
	return (
		<div
			className='flex items-center gap-3 rounded-2xl border px-5 py-4'
			style={{
				borderColor: `rgba(var(--accent-rgb), 0.2)`,
				backgroundColor: `rgba(var(--accent-rgb), 0.04)`,
			}}>
			<Keyboard className='h-5 w-5 shrink-0' style={{ color: accentColor }} />
			<div>
				<p className='text-sm font-semibold text-[var(--text-primary)]'>
					Custom keybinds - coming soon
				</p>
				<p className='mt-0.5 text-xs text-[var(--text-secondary)]'>
					You'll be able to rebind any shortcut below. For now, these are the defaults.
				</p>
			</div>
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
	show: { opacity: 1, y: 0, transition: { duration: 0.3, ease: 'easeOut' } },
}

export function KeyboardShortcutsSettings() {
	const animationsEnabled = useAnimationsEnabled()

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
				<motion.div
					{...(animationsEnabled ? { variants: itemVariants } : {})}
					className='space-y-1'>
					<div className='flex items-center gap-3'>
						<Keyboard className='h-5 w-5 text-[var(--text-secondary)]' />
						<h1 className='text-lg font-bold text-[var(--text-primary)]'>
							Keyboard Shortcuts
						</h1>
					</div>
					<p className='ml-8 text-sm text-[var(--text-secondary)]'>
						All shortcuts currently active in Postail.
					</p>
				</motion.div>

				<motion.div {...(animationsEnabled ? { variants: itemVariants } : {})}>
					<ComingSoonBanner />
				</motion.div>

				{(['global', 'compose', 'inbox'] as const).map((scope, i) => (
					<motion.div
						key={scope}
						{...(animationsEnabled ? { variants: itemVariants } : {})}>
						<ScopeSection
							scope={scope}
							shortcuts={byScope[scope]}
							defaultOpen={i === 0}
						/>
					</motion.div>
				))}
			</motion.div>
		</div>
	)
}
