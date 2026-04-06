import { useState, useRef, useEffect, useCallback } from 'react'
import { ChevronDown, Tag, Plus, X, Check } from 'lucide-react'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { parseAddresses } from '@/lib/parseAddress'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useThemeStore } from '@/stores/themeStore'
import i18n from '@/i18n'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import type { MessageViewMetaProps } from '@/types/components/shared'

// Deterministic hue from tag string so the same tag always has the same colour.
function tagHue(tag: string): number {
	let hash = 0
	for (let i = 0; i < tag.length; i++) hash = tag.charCodeAt(i) + ((hash << 5) - hash)
	return Math.abs(hash) % 360
}

const senderAvatar = (name: string, email: string) => {
	const initials = name
		? name
				.split(' ')
				.slice(0, 2)
				.map((w) => w[0])
				.join('')
				.toUpperCase()
		: email.slice(0, 2).toUpperCase()

	let hash = 0
	for (let i = 0; i < email.length; i++) hash = email.charCodeAt(i) + ((hash << 5) - hash)
	const hue = Math.abs(hash) % 360

	return { initials, hue }
}

const formatDate = (iso: string) =>
	new Date(iso).toLocaleString(i18n.t('app.languageCode'), {
		weekday: 'short',
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		hour12: false,
	})

// ── TagBadge ────────────────────────────────────────────────────────────────
const TagBadge = ({
	tag,
	onRemove,
	animationsEnabled,
}: {
	tag: string
	onRemove?: () => void
	animationsEnabled: boolean
}) => {
	const hue = tagHue(tag)
	return (
		<motion.span
			layout={animationsEnabled || false}
			initial={animationsEnabled ? { opacity: 0, scale: 0.8 } : undefined}
			{...(animationsEnabled ? { animate: { opacity: 1, scale: 1 } } : {})}
			exit={animationsEnabled ? { opacity: 0, scale: 0.8 } : undefined}
			transition={{ duration: 0.15 }}
			className='group/tag flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-semibold ring-1 transition-colors'
			style={{
				background: `hsl(${hue} 60% 18%)`,
				color: `hsl(${hue} 80% 75%)`,

				boxShadow: `inset 0 0 0 1px hsl(${hue} 60% 30%)`,
			}}>
			<Tag className='h-2.5 w-2.5' />
			{tag}
			{onRemove && (
				<button
					type='button'
					onClick={(e) => {
						e.stopPropagation()
						onRemove()
					}}
					className='ml-0.5 rounded-full opacity-50 transition-opacity hover:opacity-100 focus:outline-none'>
					<X className='h-2.5 w-2.5' />
				</button>
			)}
		</motion.span>
	)
}

// ── TagPicker ───────────────────────────────────────────────────────────────
const TagPicker = ({
	accountId,
	activeTags,
	onAdd,
	onRemove,
}: {
	accountId: string
	activeTags: string[]
	onAdd: (tag: string) => void
	onRemove: (tag: string) => void
}) => {
	const accentColor = useThemeStore((s) => s.accentColor)
	const [open, setOpen] = useState(false)
	const [input, setInput] = useState('')
	const popoverRef = useRef<HTMLDivElement>(null)
	const inputRef = useRef<HTMLInputElement>(null)
	const animationsEnabled = useAnimationsEnabled()
	const { t } = useTypedTranslation(['common', 'inbox'])

	const { data: allTags = [] } = useQuery<string[]>({
		queryKey: ['account-tags', accountId],
		queryFn: () => invoke<string[]>('get_account_tags', { accountId }),
		staleTime: 30_000,
	})

	// Close on outside click
	useEffect(() => {
		if (!open) return
		const handler = (e: MouseEvent) => {
			if (popoverRef.current && !popoverRef.current.contains(e.target as Node)) {
				setOpen(false)
				setInput('')
			}
		}
		document.addEventListener('mousedown', handler)
		return () => document.removeEventListener('mousedown', handler)
	}, [open])

	// Focus input when popover opens
	useEffect(() => {
		if (open) setTimeout(() => inputRef.current?.focus(), 50)
	}, [open])

	const trimmed = input.trim().toLowerCase()
	const suggestions = allTags.filter(
		(tag) => !activeTags.includes(tag) && tag.toLowerCase().includes(trimmed)
	)
	const canCreate =
		trimmed.length > 0 &&
		!allTags.some((tag) => tag.toLowerCase() === trimmed) &&
		!activeTags.some((tag) => tag.toLowerCase() === trimmed)

	const handleAdd = useCallback(
		(tag: string) => {
			let trimmedTag = tag.trim()
			if (!trimmedTag) return

			if (trimmedTag.includes(' ')) {
				trimmedTag = trimmedTag.replace(/ /g, '_')
				const { toast } = require('@/stores/toastStore')
				toast.info(t('inbox:messageView.tags.formatToast'), {
					description: t('inbox:messageView.tags.formatDesc'),
				})
			}

			onAdd(trimmedTag)
			setInput('')
		},
		[onAdd, t]
	)

	const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
		e.stopPropagation()
		if (e.key === 'Enter' && trimmed) {
			e.preventDefault()
			// If there's exactly one suggestion use it, otherwise create new
			if (suggestions.length === 1) {
				handleAdd(suggestions[0])
			} else if (canCreate) {
				handleAdd(trimmed)
			}
		}
		if (e.key === 'Escape') {
			setOpen(false)
			setInput('')
		}
	}

	return (
		<div className='relative' ref={popoverRef}>
			<button
				type='button'
				onClick={() => setOpen((v) => !v)}
				className='flex items-center gap-1.5 rounded-full border border-dashed border-[var(--border-subtle)] px-2.5 py-0.5 text-[10px] font-medium text-[var(--text-tertiary)] transition-all hover:border-[var(--border-stronger)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-secondary)] active:scale-95'
				aria-label={t('inbox:messageView.tags.addAria')}
				style={{
					borderColor: open ? accentColor : undefined,
					color: open ? accentColor : undefined,
				}}>
				<Plus className='h-3 w-3' />
				<span>{t('inbox:messageView.tags.add')}</span>
			</button>

			<AnimatePresence>
				{open && (
					<motion.div
						initial={animationsEnabled ? { opacity: 0, y: -4, scale: 0.96 } : undefined}
						{...(animationsEnabled ? { animate: { opacity: 1, y: 0, scale: 1 } } : {})}
						exit={animationsEnabled ? { opacity: 0, y: -4, scale: 0.96 } : undefined}
						transition={{ duration: 0.15, ease: [0.16, 1, 0.3, 1] }}
						className='absolute top-8 left-0 z-50 w-52 overflow-hidden rounded-xl border border-[var(--border-stronger,rgba(255,255,255,0.15))] bg-[#0d1117]/90 shadow-[0_20px_50px_rgba(0,0,0,0.8)] ring-1 ring-white/10'
						style={{ backdropFilter: 'blur(12px)' }}>
						{/* Search / create input */}
						<div className='flex items-center gap-2 border-b border-[var(--border-faint)] px-3 py-2'>
							<Tag className='h-3 w-3 shrink-0 text-[var(--text-tertiary)]' />
							<input
								ref={inputRef}
								type='text'
								value={input}
								onChange={(e) => setInput(e.target.value)}
								onKeyDown={handleKeyDown}
								placeholder={t('inbox:messageView.tags.placeholder')}
								maxLength={40}
								className='min-w-0 flex-1 bg-transparent text-xs text-[var(--text-primary)] placeholder:text-[var(--text-tertiary)] focus:outline-none'
							/>
						</div>

						<div className='max-h-48 overflow-y-auto py-1'>
							{/* Existing tags on this message */}
							{activeTags.length > 0 && (
								<>
									<p className='px-3 py-1 text-[9px] font-bold tracking-widest text-[var(--text-tertiary)] uppercase'>
										{t('inbox:messageView.tags.applied')}
									</p>
									{activeTags.map((tag) => (
										<button
											key={tag}
											type='button'
											onClick={() => onRemove(tag)}
											className='flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors hover:bg-[var(--surface-hover)]'>
											<Check
												className='h-3 w-3 shrink-0'
												style={{ color: `hsl(${tagHue(tag)} 70% 65%)` }}
											/>
											<span style={{ color: `hsl(${tagHue(tag)} 80% 75%)` }}>
												{tag}
											</span>
										</button>
									))}
								</>
							)}

							{/* Suggestions from other messages */}
							{suggestions.length > 0 && (
								<>
									<p className='px-3 py-1 text-[9px] font-bold tracking-widest text-[var(--text-tertiary)] uppercase'>
										{activeTags.length > 0
											? t('inbox:messageView.tags.more')
											: t('inbox:messageView.tags.title')}
									</p>
									{suggestions.map((tag) => (
										<button
											key={tag}
											type='button'
											onClick={() => handleAdd(tag)}
											className='flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors hover:bg-[var(--surface-hover)]'>
											<Tag className='h-3 w-3 shrink-0 text-[var(--text-tertiary)]' />
											<span style={{ color: `hsl(${tagHue(tag)} 80% 75%)` }}>
												{tag}
											</span>
										</button>
									))}
								</>
							)}

							{/* Create new tag */}
							{canCreate && (
								<button
									type='button'
									onClick={() => handleAdd(trimmed)}
									className='flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors hover:bg-[var(--surface-hover)]'>
									<Plus className='h-3 w-3 shrink-0 text-[var(--text-tertiary)]' />
									<span className='text-[var(--text-secondary)]'>
										{t('inbox:messageView.tags.create', { tag: trimmed })}
									</span>
								</button>
							)}

							{activeTags.length === 0 && suggestions.length === 0 && !canCreate && (
								<p className='px-3 py-2 text-xs text-[var(--text-tertiary)]'>
									{trimmed
										? t('inbox:messageView.tags.noMatch')
										: t('inbox:messageView.tags.empty')}
								</p>
							)}
						</div>
					</motion.div>
				)}
			</AnimatePresence>
		</div>
	)
}

// ── MessageViewMeta ──────────────────────────────────────────────────────────
export const MessageViewMeta = ({
	header,
	accountId = '',
	mailbox = '',
	onTagsChange,
}: MessageViewMetaProps) => {
	const { t } = useTypedTranslation(['common', 'inbox'])
	const animationsEnabled = useAnimationsEnabled()
	const [expanded, setExpanded] = useState(false)
	const queryClient = useQueryClient()

	const from = parseAddresses(header.from)[0]
	const to = parseAddresses(header.to)
	const cc = header.cc?.length ? parseAddresses(header.cc) : []
	const dateStr = formatDate(header.internal_date)
	const { initials, hue } = senderAvatar(from?.name || '', from?.email || '')

	const toStr = to.map((r) => r.name || r.email).join(', ')
	const recipientSummary = cc.length > 0 ? `${toStr} +${cc.length} more` : toStr

	const activeTags = (header.tags ?? []).filter((t) => t && t !== 'null')

	const invalidate = () => {
		queryClient.invalidateQueries({ queryKey: ['message', accountId, mailbox, header.uid] })
		queryClient.invalidateQueries({ queryKey: ['messages', accountId] })
		queryClient.invalidateQueries({ queryKey: ['account-tags', accountId] })
	}

	const handleAdd = async (tag: string) => {
		try {
			await invoke('add_message_tag', { accountId, mailbox, uid: header.uid, tag })
			const next = [...activeTags, tag]
			onTagsChange?.(next)
			invalidate()
		} catch (e) {
			console.error('Failed to add tag', e)
		}
	}

	const handleRemove = async (tag: string) => {
		try {
			await invoke('remove_message_tag', { accountId, mailbox, uid: header.uid, tag })
			const next = activeTags.filter((t) => t !== tag)
			onTagsChange?.(next)
			invalidate()
		} catch (e) {
			console.error('Failed to remove tag', e)
		}
	}

	return (
		<div className='flex items-start gap-3 px-5 py-4'>
			{/* Avatar */}
			<div
				className='mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-xs font-bold'
				style={{
					background: `linear-gradient(135deg, hsl(${hue} 55% 25%), hsl(${hue} 65% 15%))`,
					color: `hsl(${hue} 80% 85%)`,
					boxShadow: `inset 0 1px 0 hsl(0 0% 100% / 0.1), 0 2px 4px hsl(${hue} 55% 10% / 0.3), 0 0 0 1px hsl(${hue} 55% 30% / 0.5)`,
				}}>
				{initials}
			</div>

			{/* Sender + meta */}
			<div className='min-w-0 flex-1'>
				{/* Row 1: name + date */}
				<div className='flex items-baseline justify-between gap-3'>
					<div className='flex min-w-0 items-baseline gap-2'>
						<span className='truncate text-sm font-semibold text-[var(--text-primary)]'>
							{from?.name || from?.email}
						</span>
						{from?.name && (
							<span className='shrink-0 text-xs text-[var(--text-secondary)]'>
								{from.email}
							</span>
						)}
					</div>
					<span className='shrink-0 text-xs text-[var(--text-secondary)]'>{dateStr}</span>
				</div>

				{/* Row 2: to summary */}
				<div className='mt-1 flex items-center'>
					<button
						type='button'
						onClick={() => setExpanded((e) => !e)}
						className='flex items-center gap-1 text-xs text-[var(--text-secondary)] transition-colors hover:text-[var(--text-primary)]'>
						<span>{t('inbox:messageView.toLabel', { name: recipientSummary })}</span>
						<motion.div
							{...(animationsEnabled
								? { animate: { rotate: expanded ? 180 : 0 } }
								: {})}
							transition={{ duration: 0.2 }}>
							<ChevronDown className='h-3 w-3' />
						</motion.div>
					</button>
				</div>

				{/* Row 3: active tags + picker */}
				<div className='relative z-20 mt-1.5 flex flex-wrap items-center gap-2'>
					<AnimatePresence mode='popLayout'>
						{activeTags.map((tag) => (
							<TagBadge
								key={tag}
								tag={tag}
								onRemove={() => handleRemove(tag)}
								animationsEnabled={animationsEnabled}
							/>
						))}
					</AnimatePresence>

					<TagPicker
						accountId={accountId}
						activeTags={activeTags}
						onAdd={handleAdd}
						onRemove={handleRemove}
					/>
				</div>

				{/* Expanded details */}
				<AnimatePresence>
					{expanded && (
						<motion.div
							initial={
								animationsEnabled
									? { opacity: 0, height: 0, filter: 'blur(4px)' }
									: undefined
							}
							{...(animationsEnabled
								? { animate: { opacity: 1, height: 'auto', filter: 'blur(0px)' } }
								: {})}
							exit={
								animationsEnabled
									? { opacity: 0, height: 0, filter: 'blur(4px)' }
									: undefined
							}
							transition={{
								duration: animationsEnabled ? 0.25 : 0,
								ease: [0.16, 1, 0.3, 1],
							}}
							className='overflow-hidden'>
							<div className='mt-2.5 flex flex-col gap-1.5 rounded-lg border border-[var(--border-faint)] bg-[var(--surface-panel)] px-3 py-2.5 text-xs'>
								<MetaRow label={t('inbox:messageView.from')}>
									<span className='text-[var(--text-primary)]'>
										{from?.name}{' '}
										<span className='text-[var(--text-secondary)]'>
											&lt;{from?.email}&gt;
										</span>
									</span>
								</MetaRow>
								<MetaRow label={t('inbox:messageView.to')}>
									<span className='text-[var(--text-primary)]'>
										{to.map((r, i) => (
											<span key={i}>
												{r.name ? (
													<>
														{r.name}{' '}
														<span className='text-[var(--text-secondary)]'>
															&lt;{r.email}&gt;
														</span>
													</>
												) : (
													r.email
												)}
												{i < to.length - 1 && ', '}
											</span>
										))}
									</span>
								</MetaRow>
								{cc.length > 0 && (
									<MetaRow label={t('inbox:messageView.cc')}>
										<span className='text-[var(--text-primary)]'>
											{cc.map((r, i) => (
												<span key={i}>
													{r.name || r.email}
													{i < cc.length - 1 && ', '}
												</span>
											))}
										</span>
									</MetaRow>
								)}
								<MetaRow label={t('inbox:messageView.date')}>
									<span className='text-[var(--text-primary)]'>{dateStr}</span>
								</MetaRow>
							</div>
						</motion.div>
					)}
				</AnimatePresence>
			</div>
		</div>
	)
}

const MetaRow = ({ label, children }: { label: string; children: React.ReactNode }) => (
	<div className='flex items-baseline gap-2'>
		<span className='w-8 shrink-0 text-right text-[var(--text-secondary)]'>{label}</span>
		<div className='min-w-0 flex-1'>{children}</div>
	</div>
)
