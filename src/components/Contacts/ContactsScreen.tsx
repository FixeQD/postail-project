import { useState, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { ArrowLeft, Users } from 'lucide-react'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useContactsTranslation } from '@/hooks/useTypedTranslation'

interface ContactsScreenProps {
	onBack: () => void
}

// ─── Empty states ─────────────────────────────────────────────────────────────

function NoContactSelected() {
	const { t } = useContactsTranslation()
	return (
		<div className='flex h-full flex-col items-center justify-center gap-3 px-8 text-center'>
			<div
				className='flex h-14 w-14 items-center justify-center rounded-2xl'
				style={{ backgroundColor: 'rgba(var(--accent-rgb), 0.08)' }}>
				<Users className='h-6 w-6' style={{ color: 'rgba(var(--accent-rgb), 0.6)' }} />
			</div>
			<div className='flex flex-col gap-1'>
				<p className='text-[13px] font-medium text-[var(--text-primary)]'>
					{t('contacts:empty.noContact.title')}
				</p>
				<p className='text-[12px] text-[var(--text-tertiary)]'>
					{t('contacts:empty.noContact.description')}
				</p>
			</div>
		</div>
	)
}

function NoContacts() {
	const { t } = useContactsTranslation()
	return (
		<div className='flex flex-1 flex-col items-center justify-center gap-3 px-6 text-center'>
			<div
				className='flex h-12 w-12 items-center justify-center rounded-2xl'
				style={{ backgroundColor: 'rgba(var(--accent-rgb), 0.06)' }}>
				<Users className='h-5 w-5' style={{ color: 'rgba(var(--accent-rgb), 0.5)' }} />
			</div>
			<div className='flex flex-col gap-1'>
				<p className='text-[13px] font-medium text-[var(--text-primary)]'>
					{t('contacts:empty.noContacts.title')}
				</p>
				<p className='text-[12px] leading-relaxed text-[var(--text-tertiary)]'>
					{t('contacts:empty.noContacts.description')}
				</p>
			</div>
		</div>
	)
}

// ─── Main screen ──────────────────────────────────────────────────────────────

export const ContactsScreen = ({ onBack }: ContactsScreenProps) => {
	const { t } = useContactsTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const [selectedContactId, setSelectedContactId] = useState<number | null>(null)

	const handleSelectContact = useCallback((id: number) => {
		setSelectedContactId(id)
	}, [])

	return (
		<div className='noise-overlay relative flex h-full overflow-hidden text-[var(--text-primary)]'>
			{/* Ambient accent orb */}
			<div
				className='pointer-events-none absolute top-[-10%] right-[-5%] h-[400px] w-[400px] rounded-full blur-[120px]'
				style={{ backgroundColor: `rgba(var(--accent-rgb), 0.04)` }}
			/>

			{/* ── Left column ── */}
			<motion.div
				{...(animationsEnabled
					? {
							initial: { opacity: 0, x: -12 },
							animate: { opacity: 1, x: 0 },
							transition: { duration: 0.3, ease: [0.16, 1, 0.3, 1] },
						}
					: {})}
				className='relative flex w-72 shrink-0 flex-col border-r'
				style={{
					borderColor: 'var(--border-subtle)',
					backgroundColor: 'var(--surface-panel)',
				}}>
				{/* Right edge fade */}
				<div className='pointer-events-none absolute top-0 right-0 bottom-0 w-px bg-gradient-to-b from-transparent via-black/[0.06] to-transparent dark:via-white/[0.06]' />

				{/* Header */}
				<div
					className='flex items-center gap-2 border-b px-4 py-3'
					style={{ borderColor: 'var(--border-subtle)' }}>
					<button
						type='button'
						onClick={onBack}
						className='group mr-1 flex items-center gap-1.5 rounded-lg px-2 py-1.5 text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
						<ArrowLeft className='h-3.5 w-3.5 transition-transform group-hover:-translate-x-0.5' />
					</button>
					<h1 className='text-[14px] font-semibold tracking-tight text-[var(--text-primary)]'>
						{t('contacts:title')}
					</h1>
				</div>

				{/* Search + toolbar */}
				<div className='flex flex-col gap-2 px-3 py-2.5'>
					<input
						type='text'
						placeholder={t('contacts:search.placeholder')}
						className='w-full rounded-lg bg-[var(--surface-active)] px-3 py-1.5 text-[13px] text-[var(--text-primary)] ring-1 ring-transparent transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:ring-[rgba(var(--accent-rgb),0.4)]'
					/>
					<div className='flex items-center gap-1.5'>
						<button
							type='button'
							className='flex flex-1 items-center justify-center rounded-lg px-3 py-1.5 text-[12px] font-medium text-white transition-all hover:brightness-110 active:scale-[0.97]'
							style={{
								background: `linear-gradient(115deg, var(--accent-dark) 0%, var(--accent-color) 100%)`,
								boxShadow: `0 3px 10px -3px rgba(var(--accent-rgb), 0.4)`,
							}}>
							{t('contacts:toolbar.newContact')}
						</button>
						<button
							type='button'
							className='rounded-lg bg-[var(--surface-active)] px-2.5 py-1.5 text-[12px] font-medium text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
							{t('contacts:toolbar.import')}
						</button>
						<button
							type='button'
							className='rounded-lg bg-[var(--surface-active)] px-2.5 py-1.5 text-[12px] font-medium text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
							{t('contacts:toolbar.export')}
						</button>
					</div>
				</div>

				{/* Contact list */}
				<div className='flex-1 overflow-y-auto px-2 pb-2'>
					<NoContacts />
				</div>
			</motion.div>

			{/* ── Right panel ── */}
			<AnimatePresence mode='wait'>
				<motion.div
					key={selectedContactId ?? 'empty'}
					{...(animationsEnabled
						? {
								initial: { opacity: 0 },
								animate: { opacity: 1 },
								exit: { opacity: 0 },
								transition: { duration: 0.15 },
							}
						: {})}
					className='flex flex-1 flex-col overflow-hidden'>
					{selectedContactId === null ? (
						<NoContactSelected />
					) : (
						// ContactCard from 17.3 goes here
						<div className='flex h-full items-center justify-center text-sm text-[var(--text-tertiary)]'>
							Contact #{selectedContactId}
						</div>
					)}
				</motion.div>
			</AnimatePresence>
		</div>
	)
}
