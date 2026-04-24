import { useState, useCallback, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { ArrowLeft, Users } from 'lucide-react'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useContactsTranslation } from '@/hooks/useTypedTranslation'
import type { Contact } from '@/types/components/compose'
import { ContactCard } from './ContactCard'
import { ContactList } from './ContactList'
import { EditContactDialog } from './EditContactDialog'

interface ContactsScreenProps {
	onBack: () => void
}

// ─── Empty states ─────────────────────────────────────────────────────────────

function NoContactSelected() {
	const { t } = useContactsTranslation()
	const animationsEnabled = useAnimationsEnabled()

	return (
		<div className='relative flex h-full flex-col items-center justify-center overflow-hidden px-8 text-center'>
			{/* Decorative background element */}
			<div className='absolute inset-0 flex items-center justify-center opacity-[0.03] pointer-events-none'>
				<Users className='h-[400px] w-[400px]' />
			</div>

			<motion.div
				{...(animationsEnabled
					? {
							initial: { opacity: 0, scale: 0.95 },
							animate: { opacity: 1, scale: 1 },
							transition: { duration: 0.4, ease: 'easeOut' },
						}
					: {})}
				className='relative z-10 flex flex-col items-center gap-4'>
				<div
					className='flex h-20 w-20 items-center justify-center rounded-[28%] shadow-2xl'
					style={{
						background: `linear-gradient(135deg, rgba(var(--accent-rgb), 0.15) 0%, rgba(var(--accent-rgb), 0.05) 100%)`,
						boxShadow: `0 20px 40px -10px rgba(var(--accent-rgb), 0.1)`,
					}}>
					<Users className='h-8 w-8' style={{ color: 'rgb(var(--accent-rgb))' }} />
				</div>
				<div className='flex flex-col gap-1.5'>
					<h3 className='text-[16px] font-bold tracking-tight text-[var(--text-primary)]'>
						{t('contacts:empty.noContact.title')}
					</h3>
					<p className='max-w-[240px] text-[13px] leading-relaxed text-[var(--text-tertiary)]'>
						{t('contacts:empty.noContact.description')}
					</p>
				</div>
			</motion.div>
		</div>
	)
}

// ─── Main screen ──────────────────────────────────────────────────────────────

export const ContactsScreen = ({ onBack }: ContactsScreenProps) => {
	const { t } = useContactsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const [selectedContact, setSelectedContact] = useState<Contact | null>(null)
	const [isCreateOpen, setIsCreateOpen] = useState(false)

	useEffect(() => {
		const handleDeleted = (e: CustomEvent<{ id: number }>) => {
			if (selectedContact?.id === e.detail.id) {
				setSelectedContact(null)
			}
		}
		window.addEventListener('app:contact-deleted', handleDeleted as EventListener)
		return () => {
			window.removeEventListener('app:contact-deleted', handleDeleted as EventListener)
		}
	}, [selectedContact])

	const handleSelectContact = useCallback((contact: Contact) => {
		setSelectedContact(contact)
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
				<div
					className='flex items-center gap-1.5 border-b px-3 py-2.5'
					style={{ borderColor: 'var(--border-subtle)' }}>
					<div className='flex flex-1 items-center gap-1.5'>
						<button
							type='button'
							onClick={() => setIsCreateOpen(true)}
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
				<div className='flex-1 overflow-hidden px-2 pb-2'>
					<ContactList selectedContact={selectedContact} onSelect={handleSelectContact} />
				</div>
			</motion.div>

			{/* ── Right panel ── */}
			<AnimatePresence mode='wait'>
				<motion.div
					key={selectedContact?.id ?? 'empty'}
					{...(animationsEnabled
						? {
								initial: { opacity: 0 },
								animate: { opacity: 1 },
								exit: { opacity: 0 },
								transition: { duration: 0.15 },
							}
						: {})}
					className='flex flex-1 flex-col overflow-hidden'>
					{selectedContact === null ? (
						<NoContactSelected />
					) : (
						<ContactCard contact={selectedContact} />
					)}
				</motion.div>
			</AnimatePresence>

			<EditContactDialog
				open={isCreateOpen}
				onOpenChange={setIsCreateOpen}
				contact={null}
			/>
		</div>
	)
}
