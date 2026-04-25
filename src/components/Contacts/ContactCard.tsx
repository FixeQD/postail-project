import { useState, useCallback, useMemo, memo } from 'react'
import { motion } from 'framer-motion'
import { format, formatDistanceToNow, parseISO } from 'date-fns'
import { Pencil, Trash2, ArrowRight, ArrowLeft, Phone, Building, Download } from 'lucide-react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { useAccountStore } from '@/stores/accountStore'
import { useMessageViewStore } from '@/stores/messageViewStore'
import { toast } from '@/components/ui/custom/Toaster'
import type { Contact } from '@/types/components/compose'
import type { MailHeader } from '@/types/mail'
import { useContactsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { EditContactDialog } from './EditContactDialog'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'

interface ContactCardProps {
	contact: Contact
}

export const ContactCard = memo(function ContactCard({ contact }: ContactCardProps) {
	const { t } = useContactsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const openMessage = useMessageViewStore((s) => s.openMessage)
	const queryClient = useQueryClient()

	const [isEditOpen, setIsEditOpen] = useState(false)
	const [isDeleteOpen, setIsDeleteOpen] = useState(false)
	const [isExporting, setIsExporting] = useState(false)

	// Notes state
	const [notes, setNotes] = useState(contact.notes ?? '')
	const [isSavingNotes, setIsSavingNotes] = useState(false)

	const handleExport = async () => {
		try {
			const selected = await save({
				filters: [{ name: 'vCard', extensions: ['vcf'] }],
				defaultPath: `${contact.name || contact.email.split('@')[0]}.vcf`
			})

			if (!selected) return

			setIsExporting(true)
			await invoke('export_contacts_vcf', {
				path: selected,
				id: contact.id
			})

			toast.success(t('contacts:export.success', { count: 1 }))
		} catch (error) {
			console.error('Failed to export contact:', error)
			toast.error(t('contacts:export.failed'))
		} finally {
			setIsExporting(false)
		}
	}

	// Fetch contact's recent messages
	const { data: messages, isLoading: messagesLoading } = useQuery({
		queryKey: ['contact-messages', activeAccount?.id, contact.email],
		queryFn: async () => {
			if (!activeAccount) return []
			return await invoke<MailHeader[]>('get_contact_messages', {
				accountId: activeAccount.id,
				email: contact.email,
				limit: 50,
			})
		},
		enabled: !!activeAccount,
	})

	// Auto-save notes on blur
	const handleNotesBlur = useCallback(async () => {
		if (!activeAccount) return
		setIsSavingNotes(true)
		try {
			await invoke('update_contact', {
				id: contact.id,
				name: contact.name ?? null,
				email: contact.email,
				phone: contact.phone ?? null,
				company: contact.company ?? null,
				notes: notes || null,
				avatar_url: contact.avatar_url ?? null,
				birthday: contact.birthday ?? null,
			})
			toast.success(t('common:status.success'))
		} catch (err) {
			toast.error(t('common:errors.saveFailed'))
			setNotes(contact.notes ?? '') // revert
		} finally {
			setIsSavingNotes(false)
		}
	}, [activeAccount, contact, notes, t])

	// Open message on click
	const handleMessageClick = useCallback(
		(uid: number, mailbox: string) => {
			if (!activeAccount) return
			openMessage(activeAccount.id, mailbox, uid)
		},
		[activeAccount, openMessage]
	)

	// Generate initials for avatar
	const initials = useMemo(() => {
		if (!contact.name) return contact.email.slice(0, 2).toUpperCase()
		const parts = contact.name.trim().split(/\s+/)
		if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase()
		return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase()
	}, [contact.name, contact.email])

	// Format date
	const formatDate = useCallback((dateStr: string) => {
		try {
			const date = parseISO(dateStr)
			if (format(date, 'yyyy') !== format(new Date(), 'yyyy')) {
				return format(date, 'MMM d, yyyy')
			}
			return formatDistanceToNow(date, { addSuffix: true })
		} catch {
			return ''
		}
	}, [])

	// Determine direction of message relative to active account
	const isSent = useCallback(
		(message: MailHeader) => {
			if (!activeAccount) return false
			const fromMatch = message.from.some((f) => f.includes(activeAccount.email))
			return fromMatch
		},
		[activeAccount]
	)

	if (!activeAccount) return null

	return (
		<div className='flex h-full flex-col overflow-hidden'>
			{/* Header with avatar and basic info */}
			<div
				className='relative overflow-hidden px-8 pt-10 pb-8'
				style={{
					background: `linear-gradient(to bottom, rgba(var(--accent-rgb), 0.03), transparent)`,
				}}>
				<div
					className='absolute top-0 left-0 h-px w-full'
					style={{
						background: `linear-gradient(90deg, transparent, var(--border-subtle), transparent)`,
					}}
				/>

				<div className='flex items-start gap-6'>
					<motion.div
						{...(animationsEnabled
							? {
									initial: { scale: 0.8, opacity: 0 },
									animate: { scale: 1, opacity: 1 },
									transition: { duration: 0.3, delay: 0.1 },
								}
							: {})}
						className='relative'>
						<div
							className='flex h-20 w-20 shrink-0 items-center justify-center rounded-2xl text-2xl font-bold shadow-xl'
							style={{
								background: `linear-gradient(135deg, rgba(var(--accent-rgb), 0.2) 0%, rgba(var(--accent-rgb), 0.08) 100%)`,
								color: 'rgb(var(--accent-rgb))',
								border: `1px solid rgba(var(--accent-rgb), 0.1)`,
							}}>
							{initials}
						</div>
						<div className='absolute -right-1 -bottom-1 h-5 w-5 rounded-full border-2 border-[var(--surface-panel)] bg-green-500 shadow-sm' />
					</motion.div>

					<div className='min-w-0 flex-1 pt-1'>
						<h2 className='text-[24px] font-extrabold tracking-tight text-[var(--text-primary)]'>
							{contact.name || contact.email}
						</h2>
						<div className='mt-1 flex items-center gap-3'>
							<p className='text-[14px] font-medium text-[var(--text-secondary)]'>
								{contact.email}
							</p>
							{contact.company && (
								<>
									<div className='h-1 w-1 rounded-full bg-[var(--text-tertiary)] opacity-30' />
									<p className='text-[14px] font-medium text-[var(--text-tertiary)]'>
										{contact.company}
									</p>
								</>
							)}
						</div>

						{/* Pills for phone and company */}
						<div className='mt-4 flex flex-wrap gap-2'>
							{contact.phone && (
								<button
									className='inline-flex items-center gap-1.5 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-active)] px-2.5 py-1 text-[12px] font-semibold text-[var(--text-secondary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
									onClick={() => {
										navigator.clipboard.writeText(contact.phone!)
										toast.success('Copied to clipboard')
									}}>
									<Phone className='h-3.5 w-3.5 opacity-60' />
									{contact.phone}
								</button>
							)}
							{contact.company && (
								<span className='inline-flex items-center gap-1.5 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-active)] px-2.5 py-1 text-[12px] font-semibold text-[var(--text-secondary)]'>
									<Building className='h-3.5 w-3.5 opacity-60' />
									{contact.company}
								</span>
							)}
						</div>
					</div>

					{/* Edit and Delete buttons */}
					<div className='flex items-center gap-1.5'>
						<button
							type='button'
							onClick={handleExport}
							disabled={isExporting}
							className='flex h-9 w-9 items-center justify-center rounded-xl bg-[var(--surface-active)] text-[var(--text-tertiary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[rgb(var(--accent-rgb))] disabled:opacity-50'
							title={t('contacts:toolbar.export')}>
							<Download className='h-4 w-4' />
						</button>
						<button
							type='button'
							onClick={() => setIsEditOpen(true)}
							className='flex h-9 w-9 items-center justify-center rounded-xl bg-[var(--surface-active)] text-[var(--text-tertiary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[rgb(var(--accent-rgb))]'
							title={t('common:actions.edit')}>
							<Pencil className='h-4 w-4' />
						</button>
						<button
							type='button'
							onClick={() => setIsDeleteOpen(true)}
							className='flex h-9 w-9 items-center justify-center rounded-xl bg-[var(--surface-active)] text-[var(--text-tertiary)] transition-all hover:bg-red-500/10 hover:text-red-500'
							title={t('common:actions.delete')}>
							<Trash2 className='h-4 w-4' />
						</button>
					</div>
				</div>
			</div>

			{/* Notes section */}
			<div className='border-b p-4' style={{ borderColor: 'var(--border-subtle)' }}>
				<label className='mb-1.5 block text-[12px] font-medium text-[var(--text-secondary)]'>
					{t('notes:label')}
				</label>
				<textarea
					value={notes}
					onChange={(e) => setNotes(e.target.value)}
					onBlur={handleNotesBlur}
					placeholder={t('notes:placeholder')}
					disabled={isSavingNotes}
					className='w-full resize-none rounded-lg bg-[var(--surface-active)] px-3 py-2 text-[13px] text-[var(--text-primary)] ring-1 ring-transparent transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:ring-[rgba(var(--accent-rgb),0.4)] disabled:opacity-60'
					rows={3}
				/>
			</div>

			{/* Recent Messages */}
			<div className='flex flex-1 flex-col overflow-hidden'>
				<div
					className='border-b px-4 py-2.5'
					style={{ borderColor: 'var(--border-subtle)' }}>
					<h3 className='text-[12px] font-semibold tracking-wider text-[var(--text-secondary)] uppercase'>
						{t('recentMessages:title')}
					</h3>
				</div>

				<div className='flex-1 overflow-y-auto p-2'>
					{messagesLoading ? (
						<div className='flex items-center justify-center py-8'>
							<div
								className='h-5 w-5 animate-spin rounded-full border-2 border-transparent'
								style={{ borderTopColor: 'rgb(var(--accent-rgb))' }}
							/>
						</div>
					) : messages && messages.length > 0 ? (
						<div className='flex flex-col gap-1'>
							{messages.map((msg: MailHeader) => {
								const sent = isSent(msg)
								return (
									<motion.div
										key={`${msg.uid}-${msg.mailbox}`}
										{...(animationsEnabled
											? {
													initial: { opacity: 0, y: 8 },
													animate: { opacity: 1, y: 0 },
												}
											: {})}
										className='group cursor-pointer rounded-lg px-3 py-2 transition-colors hover:bg-[var(--surface-hover)]'
										onClick={() => handleMessageClick(msg.uid, msg.mailbox)}>
										<div className='flex items-start gap-2.5'>
											<div
												className='mt-1 shrink-0'
												style={{
													color: sent
														? 'rgb(var(--accent-rgb))'
														: 'var(--text-tertiary)',
												}}>
												{sent ? (
													<ArrowRight className='h-4 w-4' />
												) : (
													<ArrowLeft className='h-4 w-4' />
												)}
											</div>
											<div className='min-w-0 flex-1'>
												<p className='truncate text-[13px] font-medium text-[var(--text-primary)]'>
													{msg.subject || '(No Subject)'}
												</p>
												<p className='mt-0.5 truncate text-[11px] text-[var(--text-tertiary)]'>
													{formatDate(msg.internal_date)}
												</p>
											</div>
										</div>
									</motion.div>
								)
							})}
						</div>
					) : (
						<div className='flex items-center justify-center py-8 text-[13px] text-[var(--text-tertiary)]'>
							{t('recentMessages:empty')}
						</div>
					)}
				</div>
			</div>

			<EditContactDialog
				open={isEditOpen}
				onOpenChange={setIsEditOpen}
				contact={contact}
			/>

			<ConfirmationDialog
				open={isDeleteOpen}
				onOpenChange={setIsDeleteOpen}
				title={t('contacts:delete.title')}
				description={t('contacts:delete.description')}
				confirmLabel={t('common:actions.delete')}
				cancelLabel={t('common:actions.cancel')}
				confirmClassName='bg-red-500 text-white hover:bg-red-600'
				onConfirm={async () => {
					try {
						await invoke('delete_contact', { id: contact.id })
						toast.success(t('common:status.success'))
						setIsDeleteOpen(false)
						window.dispatchEvent(new CustomEvent('app:contact-deleted', { detail: { id: contact.id } }))
						queryClient.invalidateQueries({ queryKey: ['contacts-list'] })
					} catch {
						toast.error(t('common:errors.saveFailed'))
					}
				}}
			/>
		</div>
	)
})
