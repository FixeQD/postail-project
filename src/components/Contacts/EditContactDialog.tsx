import { useState, useEffect } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { motion, AnimatePresence } from 'framer-motion'
import {
	User,
	Mail,
	Phone,
	Building,
	Cake,
	FileText,
	X,
	Save,
	ArrowRight,
} from 'lucide-react'
import {
	Dialog,
	DialogContent,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/custom/Toaster'
import { useContactsTranslation } from '@/hooks/useTypedTranslation'
import type { Contact } from '@/types/components/compose'
import { useAnimationsEnabled } from '@/hooks/useMotion'

interface EditContactDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	contact?: Contact | null
	onSuccess?: (contactId: number) => void
}

export function EditContactDialog({
	open,
	onOpenChange,
	contact,
	onSuccess,
}: EditContactDialogProps) {
	const { t } = useContactsTranslation()
	const queryClient = useQueryClient()
	const animationsEnabled = useAnimationsEnabled()

	const [name, setName] = useState('')
	const [email, setEmail] = useState('')
	const [phone, setPhone] = useState('')
	const [company, setCompany] = useState('')
	const [notes, setNotes] = useState('')
	const [birthday, setBirthday] = useState('')
	const [isSaving, setIsSaving] = useState(false)

	useEffect(() => {
		if (open) {
			setName(contact?.name ?? '')
			setEmail(contact?.email ?? '')
			setPhone(contact?.phone ?? '')
			setCompany(contact?.company ?? '')
			setNotes(contact?.notes ?? '')
			if (contact?.birthday) {
				const date = new Date(contact.birthday * 1000)
				setBirthday(date.toISOString().split('T')[0])
			} else {
				setBirthday('')
			}
		}
	}, [open, contact])

	const handleSave = async (e: React.FormEvent) => {
		e.preventDefault()
		if (!email.includes('@')) {
			toast.error(t('common:errors.invalidEmail'))
			return
		}

		setIsSaving(true)
		try {
			let birthdayTs: number | null = null
			if (birthday) {
				const date = new Date(birthday)
				if (!isNaN(date.getTime())) {
					birthdayTs = Math.floor(date.getTime() / 1000)
				}
			}

			let newContactId: number | undefined = contact?.id

			if (contact) {
				await invoke('update_contact', {
					id: contact.id,
					name: name || null,
					email: email,
					phone: phone || null,
					company: company || null,
					notes: notes || null,
					avatarUrl: contact.avatar_url || null,
					birthday: birthdayTs,
				})
			} else {
				newContactId = await invoke<number>('create_contact', {
					name: name || null,
					email: email,
					phone: phone || null,
					company: company || null,
					notes: notes || null,
					avatarUrl: null,
					birthday: birthdayTs,
				})
			}

			queryClient.invalidateQueries({ queryKey: ['contacts-list'] })
			if (contact) {
				queryClient.invalidateQueries({ queryKey: ['contact', contact.id] })
			}

			toast.success(t('common:status.success'))
			onOpenChange(false)
			if (onSuccess && newContactId) {
				onSuccess(newContactId)
			}
		} catch (err) {
			toast.error(t('common:errors.saveFailed'))
		} finally {
			setIsSaving(false)
		}
	}

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent
				showCloseButton={false}
				className='max-w-[440px] overflow-hidden border-[var(--border-subtle)] bg-[var(--surface-glass)] p-0 shadow-2xl backdrop-blur-xl'>
				<AnimatePresence>
					{open && (
						<motion.div
							{...(animationsEnabled
								? {
										initial: { opacity: 0, y: 10, scale: 0.98 },
										animate: { opacity: 1, y: 0, scale: 1 },
										exit: { opacity: 0, y: 10, scale: 0.98 },
										transition: { duration: 0.2, ease: [0.16, 1, 0.3, 1] },
									}
								: {})}>
							{/* Premium Header with Accent Line */}
							<div className='relative overflow-hidden px-6 pt-6 pb-5'>
								<div
									className='absolute top-0 left-0 h-[3px] w-full'
									style={{
										background: `linear-gradient(90deg, var(--accent-color), var(--accent-light))`,
									}}
								/>
								<div className='flex items-center justify-between'>
									<div>
										<h2 className='text-[18px] font-bold tracking-tight text-[var(--text-primary)]'>
											{contact ? t('contacts:edit.title') : t('contacts:create.title')}
										</h2>
										<p className='mt-1 text-[12px] text-[var(--text-tertiary)]'>
											{contact ? 'Update contact details' : 'Add a new person to your network'}
										</p>
									</div>
									<button
										onClick={() => onOpenChange(false)}
										className='group flex h-8 w-8 items-center justify-center rounded-full bg-[var(--surface-active)] transition-all hover:bg-[var(--surface-hover)]'>
										<X className='h-4 w-4 text-[var(--text-tertiary)] transition-colors group-hover:text-[var(--text-primary)]' />
									</button>
								</div>
							</div>

							<form onSubmit={handleSave} className='px-6 pb-6'>
								<div className='space-y-4'>
									{/* Name Field */}
									<div className='group relative'>
										<div className='pointer-events-none absolute top-1/2 left-3 -translate-y-1/2'>
											<User className='h-4 w-4 text-[var(--text-tertiary)] transition-colors group-focus-within:text-[rgb(var(--accent-rgb))]' />
										</div>
										<input
											type='text'
											value={name}
											onChange={(e) => setName(e.target.value)}
											placeholder={t('contacts:fields.name')}
											className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] py-2.5 pr-4 pl-10 text-[13px] text-[var(--text-primary)] transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:border-[rgba(var(--accent-rgb),0.5)] focus:ring-4 focus:ring-[rgba(var(--accent-rgb),0.1)]'
										/>
									</div>

									{/* Email Field */}
									<div className='group relative'>
										<div className='pointer-events-none absolute top-1/2 left-3 -translate-y-1/2'>
											<Mail className='h-4 w-4 text-[var(--text-tertiary)] transition-colors group-focus-within:text-[rgb(var(--accent-rgb))]' />
										</div>
										<input
											type='email'
											required
											value={email}
											onChange={(e) => setEmail(e.target.value)}
											placeholder={t('contacts:fields.email') + ' *'}
											className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] py-2.5 pr-4 pl-10 text-[13px] text-[var(--text-primary)] transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:border-[rgba(var(--accent-rgb),0.5)] focus:ring-4 focus:ring-[rgba(var(--accent-rgb),0.1)]'
										/>
									</div>

									<div className='grid grid-cols-2 gap-4'>
										{/* Phone Field */}
										<div className='group relative'>
											<div className='pointer-events-none absolute top-1/2 left-3 -translate-y-1/2'>
												<Phone className='h-4 w-4 text-[var(--text-tertiary)] transition-colors group-focus-within:text-[rgb(var(--accent-rgb))]' />
											</div>
											<input
												type='tel'
												value={phone}
												onChange={(e) => setPhone(e.target.value)}
												placeholder={t('contacts:fields.phone')}
												className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] py-2.5 pr-4 pl-10 text-[13px] text-[var(--text-primary)] transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:border-[rgba(var(--accent-rgb),0.5)] focus:ring-4 focus:ring-[rgba(var(--accent-rgb),0.1)]'
											/>
										</div>

										{/* Birthday Field */}
										<div className='group relative'>
											<div className='pointer-events-none absolute top-1/2 left-3 -translate-y-1/2'>
												<Cake className='h-4 w-4 text-[var(--text-tertiary)] transition-colors group-focus-within:text-[rgb(var(--accent-rgb))]' />
											</div>
											<input
												type='date'
												value={birthday}
												onChange={(e) => setBirthday(e.target.value)}
												className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] py-2.5 pr-4 pl-10 text-[13px] text-[var(--text-primary)] transition-all outline-none [color-scheme:dark] focus:border-[rgba(var(--accent-rgb),0.5)] focus:ring-4 focus:ring-[rgba(var(--accent-rgb),0.1)]'
											/>
										</div>
									</div>

									{/* Company Field */}
									<div className='group relative'>
										<div className='pointer-events-none absolute top-1/2 left-3 -translate-y-1/2'>
											<Building className='h-4 w-4 text-[var(--text-tertiary)] transition-colors group-focus-within:text-[rgb(var(--accent-rgb))]' />
										</div>
										<input
											type='text'
											value={company}
											onChange={(e) => setCompany(e.target.value)}
											placeholder={t('contacts:fields.company')}
											className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] py-2.5 pr-4 pl-10 text-[13px] text-[var(--text-primary)] transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:border-[rgba(var(--accent-rgb),0.5)] focus:ring-4 focus:ring-[rgba(var(--accent-rgb),0.1)]'
										/>
									</div>

									{/* Notes Field */}
									<div className='group relative'>
										<div className='pointer-events-none absolute top-4 left-3'>
											<FileText className='h-4 w-4 text-[var(--text-tertiary)] transition-colors group-focus-within:text-[rgb(var(--accent-rgb))]' />
										</div>
										<textarea
											value={notes}
											onChange={(e) => setNotes(e.target.value)}
											placeholder={t('notes:placeholder')}
											rows={3}
											className='w-full resize-none rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] py-2.5 pr-4 pl-10 text-[13px] text-[var(--text-primary)] transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:border-[rgba(var(--accent-rgb),0.5)] focus:ring-4 focus:ring-[rgba(var(--accent-rgb),0.1)]'
										/>
									</div>
								</div>

								{/* Action Buttons */}
								<div className='mt-8 flex items-center justify-end gap-3'>
									<button
										type='button'
										onClick={() => onOpenChange(false)}
										className='rounded-xl px-5 py-2.5 text-[13px] font-semibold text-[var(--text-secondary)] transition-all hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
										{t('common:actions.cancel')}
									</button>
									<button
										type='submit'
										disabled={isSaving || !email}
										className='group relative flex items-center gap-2 overflow-hidden rounded-xl px-6 py-2.5 text-[13px] font-bold text-white shadow-xl transition-all hover:brightness-110 active:scale-[0.98] disabled:opacity-50'
										style={{
											background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
										}}>
										<div className='absolute inset-0 bg-gradient-to-r from-white/0 via-white/10 to-white/0 translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-700' />
										{isSaving ? (
											<div className='h-4 w-4 animate-spin rounded-full border-2 border-white/20 border-t-white' />
										) : (
											<Save className='h-4 w-4' />
										)}
										<span>{contact ? t('common:actions.save') : 'Create Contact'}</span>
										<ArrowRight className='h-3.5 w-3.5 opacity-50 transition-transform group-hover:translate-x-0.5' />
									</button>
								</div>
							</form>
						</motion.div>
					)}
				</AnimatePresence>
			</DialogContent>
		</Dialog>
	)
}
