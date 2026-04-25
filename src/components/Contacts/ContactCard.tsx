import { useState, useCallback, memo } from 'react'
import { motion } from 'framer-motion'
import { format, formatDistanceToNow, parseISO } from 'date-fns'
import { Pencil, Trash2, ArrowRight, ArrowLeft, Phone, Building, Download, Plus, X, Mail, Globe, Calendar, MapPin, User, Info, Briefcase } from 'lucide-react'
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
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import type { ContactGroup } from '@/types/components/compose'
import { ContactAvatar } from './ContactAvatar'

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
				account_id: activeAccount.id,
				email: contact.email,
				limit: 50,
			})
		},
		enabled: !!activeAccount,
	})

	// Fetch contact's groups
	const { data: contactGroups = [] } = useQuery({
		queryKey: ['contact-groups', contact.id],
		queryFn: () => invoke<ContactGroup[]>('get_groups_for_contact', { contactId: contact.id })
	})

	// Fetch all groups for assignment
	const { data: allGroups = [] } = useQuery({
		queryKey: ['contact-groups'],
		queryFn: () => invoke<ContactGroup[]>('list_contact_groups')
	})

	const handleAddToGroup = async (groupId: number) => {
		try {
			await invoke('add_contact_to_group', { groupId, contactId: contact.id })
			queryClient.invalidateQueries({ queryKey: ['contact-groups'] })
			toast.success(t('contacts:groups.addSuccess'))
		} catch {
			toast.error(t('contacts:groups.addFailed'))
		}
	}

	const handleRemoveFromGroup = async (groupId: number) => {
		try {
			await invoke('remove_contact_from_group', { groupId, contactId: contact.id })
			queryClient.invalidateQueries({ queryKey: ['contact-groups'] })
			toast.success(t('contacts:groups.removeSuccess'))
		} catch {
			toast.error(t('contacts:groups.removeFailed'))
		}
	}

	// Auto-save notes on blur
	const handleNotesBlur = useCallback(async () => {
		if (!activeAccount) return
		setIsSavingNotes(true)
		try {
			await invoke('update_contact', {
				id: contact.id,
				email: contact.email,
				name: contact.name ?? null,
				first_name: contact.first_name ?? null,
				middle_name: contact.middle_name ?? null,
				last_name: contact.last_name ?? null,
				suffix: contact.suffix ?? null,
				nickname: contact.nickname ?? null,
				phone: contact.phone ?? null,
				phone_work: contact.phone_work ?? null,
				phone_home: contact.phone_home ?? null,
				phone_fax: contact.phone_fax ?? null,
				work_email: contact.work_email ?? null,
				company: contact.company ?? null,
				job_title: contact.job_title ?? null,
				department: contact.department ?? null,
				role: contact.role ?? null,
				website: contact.website ?? null,
				address_home: contact.address_home ?? null,
				address_work: contact.address_work ?? null,
				notes: notes || null,
				avatar_url: contact.avatar_url ?? null,
				birthday: contact.birthday ?? null,
				anniversary: contact.anniversary ?? null,
				gender: contact.gender ?? null,
			})
			toast.success(t('common:status.success'))
		} catch (err) {
			toast.error(t('common:errors.saveFailed'))
			setNotes(contact.notes ?? '')
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
				className='relative shrink-0 overflow-hidden px-8 pt-10 pb-8'
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
						<ContactAvatar 
							name={contact.name} 
							email={contact.email} 
							size="xl"
							className="rounded-2xl shadow-xl"
						/>
						<div className='absolute -right-1 -bottom-1 h-5 w-5 rounded-full border-2 border-[var(--surface-panel)] bg-green-500 shadow-sm' />
					</motion.div>

					<div className='min-w-0 flex-1 pt-1'>
						<h2 className='text-[24px] font-extrabold tracking-tight text-[var(--text-primary)]'>
							{contact.name || contact.email}
							{contact.nickname && (
								<span className='ml-2 text-[16px] font-medium text-[var(--text-tertiary)]'>
									({contact.nickname})
								</span>
							)}
						</h2>
						<div className='mt-1 flex items-center gap-3'>
							<p className='text-[14px] font-medium text-[var(--text-secondary)]'>
								{contact.email}
							</p>
							{(contact.company || contact.job_title) && (
								<>
									<div className='h-1 w-1 rounded-full bg-[var(--text-tertiary)] opacity-30' />
									<p className='text-[14px] font-medium text-[var(--text-tertiary)]'>
										{contact.job_title ? `${contact.job_title} at ${contact.company || '...'}` : contact.company}
									</p>
								</>
							)}
						</div>

						{/* Main Info Pills */}
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
							{contact.work_email && (
								<button
									className='inline-flex items-center gap-1.5 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-active)] px-2.5 py-1 text-[12px] font-semibold text-[var(--text-secondary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
									onClick={() => {
										navigator.clipboard.writeText(contact.work_email!)
										toast.success('Copied to clipboard')
									}}>
									<Mail className='h-3.5 w-3.5 opacity-60' />
									{contact.work_email}
								</button>
							)}
							{contact.website && (
								<a
									href={contact.website.startsWith('http') ? contact.website : `https://${contact.website}`}
									target='_blank'
									rel='noopener noreferrer'
									className='inline-flex items-center gap-1.5 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-active)] px-2.5 py-1 text-[12px] font-semibold text-[var(--text-secondary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[rgb(var(--accent-rgb))]'>
									<Globe className='h-3.5 w-3.5 opacity-60' />
									{contact.website.replace(/^https?:\/\//, '')}
								</a>
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

			<div className='flex-1 overflow-y-auto custom-scrollbar'>
				{/* Professional Section */}
				{(contact.company || contact.job_title || contact.department || contact.role || contact.phone_work || contact.work_email) && (
					<div className='border-b p-6 space-y-4' style={{ borderColor: 'var(--border-subtle)' }}>
						<div className='flex items-center gap-2'>
							<Briefcase className='h-4 w-4 text-[rgb(var(--accent-rgb))]' />
							<span className='text-[12px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>{t('contacts:sections.professional')}</span>
						</div>
						<div className='grid grid-cols-2 gap-y-4 gap-x-6'>
							{contact.company && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.company')}</p>
									<p className='text-[13px] font-semibold text-[var(--text-primary)]'>{contact.company}</p>
								</div>
							)}
							{contact.job_title && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.jobTitle')}</p>
									<p className='text-[13px] font-semibold text-[var(--text-primary)]'>{contact.job_title}</p>
								</div>
							)}
							{contact.department && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.department')}</p>
									<p className='text-[13px] text-[var(--text-secondary)]'>{contact.department}</p>
								</div>
							)}
							{contact.role && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.role')}</p>
									<p className='text-[13px] text-[var(--text-secondary)]'>{contact.role}</p>
								</div>
							)}
							{contact.phone_work && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.phoneWork')}</p>
									<p className='text-[13px] text-[var(--text-secondary)]'>{contact.phone_work}</p>
								</div>
							)}
							{contact.work_email && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.emailWork')}</p>
									<p className='text-[13px] text-[var(--text-secondary)]'>{contact.work_email}</p>
								</div>
							)}
						</div>
					</div>
				)}

				{/* Personal Section */}
				{(contact.birthday || contact.anniversary || contact.gender || contact.website || contact.phone_home) && (
					<div className='border-b p-6 space-y-4' style={{ borderColor: 'var(--border-subtle)' }}>
						<div className='flex items-center gap-2'>
							<User className='h-4 w-4 text-[rgb(var(--accent-rgb))]' />
							<span className='text-[12px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>{t('contacts:sections.personal')}</span>
						</div>
						<div className='grid grid-cols-2 gap-y-4 gap-x-6'>
							{contact.birthday && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.birthday')}</p>
									<p className='text-[13px] text-[var(--text-secondary)]'>{format(new Date(contact.birthday * 1000), 'MMMM d, yyyy')}</p>
								</div>
							)}
							{contact.anniversary && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.anniversary')}</p>
									<p className='text-[13px] text-[var(--text-secondary)]'>{format(new Date(contact.anniversary * 1000), 'MMMM d, yyyy')}</p>
								</div>
							)}
							{contact.gender && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.gender')}</p>
									<p className='text-[13px] text-[var(--text-secondary)]'>
										{contact.gender === 'M' ? t('contacts:fields.genderMale') : contact.gender === 'F' ? t('contacts:fields.genderFemale') : t('contacts:fields.genderOther')}
									</p>
								</div>
							)}
							{contact.phone_home && (
								<div>
									<p className='text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:fields.phoneHome')}</p>
									<p className='text-[13px] text-[var(--text-secondary)]'>{contact.phone_home}</p>
								</div>
							)}
						</div>
					</div>
				)}

				{/* Addresses Section */}
				{(contact.address_home || contact.address_work) && (
					<div className='border-b p-6 space-y-4' style={{ borderColor: 'var(--border-subtle)' }}>
						<div className='flex items-center gap-2'>
							<MapPin className='h-4 w-4 text-[rgb(var(--accent-rgb))]' />
							<span className='text-[12px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>{t('contacts:sections.addresses')}</span>
						</div>
						<div className='grid grid-cols-1 gap-4'>
							{contact.address_home && (
								<div className='rounded-xl bg-[var(--surface-active)] p-3'>
									<p className='mb-1 text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:labels.home')}</p>
									<p className='text-[13px] whitespace-pre-wrap text-[var(--text-secondary)]'>{contact.address_home}</p>
								</div>
							)}
							{contact.address_work && (
								<div className='rounded-xl bg-[var(--surface-active)] p-3'>
									<p className='mb-1 text-[11px] font-medium text-[var(--text-tertiary)] uppercase'>{t('contacts:labels.work')}</p>
									<p className='text-[13px] whitespace-pre-wrap text-[var(--text-secondary)]'>{contact.address_work}</p>
								</div>
							)}
						</div>
					</div>
				)}

				{/* Groups section */}
				<div className='border-b p-6' style={{ borderColor: 'var(--border-subtle)' }}>
					<div className='flex items-center justify-between mb-4'>
						<div className='flex items-center gap-2'>
							<Plus className='h-4 w-4 text-[rgb(var(--accent-rgb))]' />
							<span className='text-[12px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>Groups</span>
						</div>
						<Popover>
							<PopoverTrigger asChild>
								<button className='p-1 hover:bg-[var(--surface-active)] rounded-md transition-colors text-[var(--text-tertiary)] hover:text-[var(--text-primary)]'>
									<Plus className='h-3.5 w-3.5' />
								</button>
							</PopoverTrigger>
							<PopoverContent className='w-48 p-1' align='end'>
								<div className='flex flex-col gap-0.5'>
									{allGroups.length === 0 && (
										<div className='px-2 py-1.5 text-[12px] text-[var(--text-tertiary)]'>
											{t('contacts:groups.noGroups')}
										</div>
									)}
									{allGroups.map(group => {
										const isMember = contactGroups.some(cg => cg.id === group.id)
										return (
											<button
												key={group.id}
												onClick={() => isMember ? handleRemoveFromGroup(group.id) : handleAddToGroup(group.id)}
												className='flex items-center gap-2 px-2 py-1.5 rounded-md hover:bg-[var(--surface-hover)] text-[13px] text-left transition-colors'
											>
												<div className='flex-1 flex items-center gap-2 truncate'>
													<div className='h-2 w-2 rounded-full' style={{ backgroundColor: group.color || 'rgb(var(--accent-rgb))' }} />
													<span className='truncate'>{group.name}</span>
												</div>
												{isMember && <X className='h-3 w-3 text-red-500' />}
											</button>
										)
									})}
								</div>
							</PopoverContent>
						</Popover>
					</div>
					<div className='flex flex-wrap gap-2'>
						{contactGroups.length === 0 ? (
							<p className='text-[12px] text-[var(--text-tertiary)]'>
								{t('contacts:groups.none')}
							</p>
						) : (
							contactGroups.map(group => (
								<div 
									key={group.id}
									className='inline-flex items-center gap-2 rounded-full border border-[var(--border-subtle)] bg-[var(--surface-active)] pl-2 pr-1.5 py-1 text-[12px] font-medium text-[var(--text-secondary)] group'
								>
									<div className='h-2 w-2 rounded-full' style={{ backgroundColor: group.color || 'rgb(var(--accent-rgb))' }} />
									<span>{group.name}</span>
									<button 
										onClick={() => handleRemoveFromGroup(group.id)}
										className='p-0.5 hover:bg-red-500/10 hover:text-red-500 rounded-full transition-colors'
									>
										<X className='h-3 w-3' />
									</button>
								</div>
							))
						)}
					</div>
				</div>

				{/* Notes section */}
				<div className='p-6 border-b' style={{ borderColor: 'var(--border-subtle)' }}>
					<div className='flex items-center gap-2 mb-3'>
						<Info className='h-4 w-4 text-[rgb(var(--accent-rgb))]' />
						<span className='text-[12px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>
							{t('contacts:notes.label')}
						</span>
					</div>
					<textarea
						value={notes}
						onChange={(e) => setNotes(e.target.value)}
						onBlur={handleNotesBlur}
						placeholder={t('contacts:notes.placeholder')}
						disabled={isSavingNotes}
						className='w-full resize-none rounded-xl bg-[var(--surface-active)] px-4 py-3 text-[13px] text-[var(--text-primary)] ring-1 ring-transparent transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:ring-[rgba(var(--accent-rgb),0.4)] disabled:opacity-60'
						rows={3}
					/>
				</div>

				{/* Recent Messages */}
				<div className='flex flex-col'>
					<div
						className='border-b px-6 py-4'
						style={{ borderColor: 'var(--border-subtle)' }}>
						<h3 className='text-[12px] font-bold tracking-wider text-[var(--text-tertiary)] uppercase'>
							{t('contacts:recentMessages.title')}
						</h3>
					</div>

					<div className='p-2'>
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
											className='group cursor-pointer rounded-xl px-4 py-3 transition-colors hover:bg-[var(--surface-hover)]'
											onClick={() => handleMessageClick(msg.uid, msg.mailbox)}>
											<div className='flex items-start gap-3'>
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
													<p className='truncate text-[13px] font-semibold text-[var(--text-primary)]'>
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
							<div className='flex items-center justify-center py-12 text-[13px] text-[var(--text-tertiary)]'>
								{t('contacts:recentMessages.empty')}
							</div>
						)}
					</div>
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
				confirmClassName='w-full bg-red-500 text-white hover:bg-red-600'
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
