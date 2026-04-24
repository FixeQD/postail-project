import { useState, useCallback, useMemo, memo } from 'react'
import { motion } from 'framer-motion'
import { format, formatDistanceToNow, parseISO } from 'date-fns'
import { Pencil, Trash2, ArrowRight, ArrowLeft, Mail } from 'lucide-react'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { useAccountStore } from '@/stores/accountStore'
import { useMessageViewStore } from '@/stores/messageViewStore'
import { toast } from '@/components/ui/custom/Toaster'
import type { Contact } from '@/types/components/compose'
import type { MailHeader } from '@/types/mail'
import { useContactsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'

interface ContactCardProps {
	contact: Contact
}

export const ContactCard = memo(function ContactCard({ contact }: ContactCardProps) {
	const { t } = useContactsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const openMessage = useMessageViewStore((s) => s.openMessage)

	// Notes state
	const [notes, setNotes] = useState(contact.notes ?? '')
	const [isSavingNotes, setIsSavingNotes] = useState(false)

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
				className='flex shrink-0 items-start gap-4 border-b p-6'
				style={{ borderColor: 'var(--border-subtle)' }}>
				<div
					className='flex h-16 w-16 shrink-0 items-center justify-center rounded-full text-lg font-semibold'
					style={{
						backgroundColor: 'rgba(var(--accent-rgb), 0.12)',
						color: 'rgb(var(--accent-rgb))',
					}}>
					{initials}
				</div>

				<div className='min-w-0 flex-1'>
					<h2 className='truncate text-[18px] font-semibold text-[var(--text-primary)]'>
						{contact.name || contact.email}
					</h2>
					<p className='mt-0.5 truncate text-[13px] text-[var(--text-secondary)]'>
						{contact.email}
					</p>
					{contact.company && (
						<p className='mt-1 truncate text-[12px] text-[var(--text-tertiary)]'>
							{contact.company}
						</p>
					)}

					{/* Pills for phone and company */}
					<div className='mt-2 flex flex-wrap gap-1.5'>
						{contact.phone && (
							<span
								className='inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium'
								style={{
									backgroundColor: 'rgba(var(--accent-rgb), 0.08)',
									color: 'rgb(var(--accent-rgb))',
								}}>
								<Mail className='h-3 w-3' />
								{contact.phone}
							</span>
						)}
					</div>
				</div>

				{/* Edit and Delete buttons */}
				<div className='flex items-center gap-1'>
					<button
						type='button'
						className='rounded-lg p-2 text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
						title={t('common:actions.edit')}>
						<Pencil className='h-4 w-4' />
					</button>
					<button
						type='button'
						className='rounded-lg p-2 text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-red-500'
						title={t('common:actions.delete')}>
						<Trash2 className='h-4 w-4' />
					</button>
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
							{messages.map((msg) => {
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
		</div>
	)
})
