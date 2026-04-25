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
	Globe,
	Home,
	Briefcase,
	Calendar,
	Hash,
	ChevronDown,
	ChevronUp,
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
	const [firstName, setFirstName] = useState('')
	const [middleName, setMiddleName] = useState('')
	const [lastName, setLastName] = useState('')
	const [suffix, setSuffix] = useState('')
	const [nickname, setNickname] = useState('')
	const [phone, setPhone] = useState('')
	const [phoneWork, setPhoneWork] = useState('')
	const [phoneHome, setPhoneHome] = useState('')
	const [phoneFax, setPhoneFax] = useState('')
	const [workEmail, setWorkEmail] = useState('')
	const [company, setCompany] = useState('')
	const [jobTitle, setJobTitle] = useState('')
	const [department, setDepartment] = useState('')
	const [role, setRole] = useState('')
	const [website, setWebsite] = useState('')
	const [addressHome, setAddressHome] = useState('')
	const [addressWork, setAddressWork] = useState('')
	const [notes, setNotes] = useState('')
	const [birthday, setBirthday] = useState('')
	const [anniversary, setAnniversary] = useState('')
	const [gender, setGender] = useState('')
	const [isSaving, setIsSaving] = useState(false)
	const [showAdvanced, setShowAdvanced] = useState(false)

	useEffect(() => {
		if (open) {
			setName(contact?.name ?? '')
			setEmail(contact?.email ?? '')
			setFirstName(contact?.first_name ?? '')
			setMiddleName(contact?.middle_name ?? '')
			setLastName(contact?.last_name ?? '')
			setSuffix(contact?.suffix ?? '')
			setNickname(contact?.nickname ?? '')
			setPhone(contact?.phone ?? '')
			setPhoneWork(contact?.phone_work ?? '')
			setPhoneHome(contact?.phone_home ?? '')
			setPhoneFax(contact?.phone_fax ?? '')
			setWorkEmail(contact?.work_email ?? '')
			setCompany(contact?.company ?? '')
			setJobTitle(contact?.job_title ?? '')
			setDepartment(contact?.department ?? '')
			setRole(contact?.role ?? '')
			setWebsite(contact?.website ?? '')
			setAddressHome(contact?.address_home ?? '')
			setAddressWork(contact?.address_work ?? '')
			setNotes(contact?.notes ?? '')
			
			if (contact?.birthday) {
				const date = new Date(contact.birthday * 1000)
				setBirthday(date.toISOString().split('T')[0])
			} else {
				setBirthday('')
			}

			if (contact?.anniversary) {
				const date = new Date(contact.anniversary * 1000)
				setAnniversary(date.toISOString().split('T')[0])
			} else {
				setAnniversary('')
			}

			setGender(contact?.gender ?? '')
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

			let anniversaryTs: number | null = null
			if (anniversary) {
				const date = new Date(anniversary)
				if (!isNaN(date.getTime())) {
					anniversaryTs = Math.floor(date.getTime() / 1000)
				}
			}

			let newContactId: number | undefined = contact?.id

			const payload = {
				email,
				name: name || null,
				first_name: firstName || null,
				middle_name: middleName || null,
				last_name: lastName || null,
				suffix: suffix || null,
				nickname: nickname || null,
				phone: phone || null,
				phone_work: phoneWork || null,
				phone_home: phoneHome || null,
				phone_fax: phoneFax || null,
				work_email: workEmail || null,
				company: company || null,
				job_title: jobTitle || null,
				department: department || null,
				role: role || null,
				website: website || null,
				address_home: addressHome || null,
				address_work: addressWork || null,
				notes: notes || null,
				avatar_url: contact?.avatar_url || null,
				birthday: birthdayTs,
				anniversary: anniversaryTs,
				gender: gender || null,
			}

			if (contact) {
				await invoke('update_contact', { id: contact.id, ...payload })
			} else {
				newContactId = await invoke<number>('create_contact', payload)
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
								<div className='max-h-[60vh] overflow-y-auto pr-2 custom-scrollbar space-y-4'>
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
									</div>

									{/* Advanced Toggle */}
									<button
										type='button'
										onClick={() => setShowAdvanced(!showAdvanced)}
										className='flex w-full items-center justify-between rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-4 py-2 transition-colors hover:bg-[var(--surface-hover)]'>
										<span className='text-[12px] font-semibold text-[var(--text-secondary)]'>Advanced Fields</span>
										{showAdvanced ? <ChevronUp className='h-4 w-4' /> : <ChevronDown className='h-4 w-4' />}
									</button>

									<AnimatePresence>
										{showAdvanced && (
											<motion.div
												initial={{ height: 0, opacity: 0 }}
												animate={{ height: 'auto', opacity: 1 }}
												exit={{ height: 0, opacity: 0 }}
												transition={{ duration: 0.3 }}
												className='space-y-4 overflow-hidden pt-2'>
												
												{/* Name Breakdown */}
												<div className='grid grid-cols-2 gap-3'>
													<input
														type='text'
														value={firstName}
														onChange={(e) => setFirstName(e.target.value)}
														placeholder='First Name'
														className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
													<input
														type='text'
														value={lastName}
														onChange={(e) => setLastName(e.target.value)}
														placeholder='Last Name'
														className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
												</div>
												<div className='grid grid-cols-3 gap-3'>
													<input
														type='text'
														value={middleName}
														onChange={(e) => setMiddleName(e.target.value)}
														placeholder='Middle'
														className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
													<input
														type='text'
														value={suffix}
														onChange={(e) => setSuffix(e.target.value)}
														placeholder='Suffix'
														className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
													<input
														type='text'
														value={nickname}
														onChange={(e) => setNickname(e.target.value)}
														placeholder='Nickname'
														className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
												</div>

												{/* Work Details */}
												<div className='space-y-3 pt-2 border-t border-[var(--border-subtle)]'>
													<div className='flex items-center gap-2 px-1'>
														<Briefcase className='h-3.5 w-3.5 text-[var(--text-tertiary)]' />
														<span className='text-[11px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>Professional</span>
													</div>
													<div className='grid grid-cols-2 gap-3'>
														<input
															type='text'
															value={jobTitle}
															onChange={(e) => setJobTitle(e.target.value)}
															placeholder='Job Title'
															className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
														/>
														<input
															type='text'
															value={department}
															onChange={(e) => setDepartment(e.target.value)}
															placeholder='Department'
															className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
														/>
													</div>
													<input
														type='email'
														value={workEmail}
														onChange={(e) => setWorkEmail(e.target.value)}
														placeholder='Work Email'
														className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
													<input
														type='text'
														value={website}
														onChange={(e) => setWebsite(e.target.value)}
														placeholder='Website / URL'
														className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
												</div>

												{/* Additional Phones */}
												<div className='space-y-3 pt-2 border-t border-[var(--border-subtle)]'>
													<div className='flex items-center gap-2 px-1'>
														<Phone className='h-3.5 w-3.5 text-[var(--text-tertiary)]' />
														<span className='text-[11px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>Additional Phones</span>
													</div>
													<div className='grid grid-cols-2 gap-3'>
														<input
															type='tel'
															value={phoneWork}
															onChange={(e) => setPhoneWork(e.target.value)}
															placeholder='Work Phone'
															className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
														/>
														<input
															type='tel'
															value={phoneHome}
															onChange={(e) => setPhoneHome(e.target.value)}
															placeholder='Home Phone'
															className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
														/>
													</div>
													<input
														type='tel'
														value={phoneFax}
														onChange={(e) => setPhoneFax(e.target.value)}
														placeholder='Fax'
														className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
												</div>

												{/* Addresses */}
												<div className='space-y-3 pt-2 border-t border-[var(--border-subtle)]'>
													<div className='flex items-center gap-2 px-1'>
														<Home className='h-3.5 w-3.5 text-[var(--text-tertiary)]' />
														<span className='text-[11px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>Addresses</span>
													</div>
													<textarea
														value={addressHome}
														onChange={(e) => setAddressHome(e.target.value)}
														placeholder='Home Address'
														rows={2}
														className='w-full resize-none rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
													<textarea
														value={addressWork}
														onChange={(e) => setAddressWork(e.target.value)}
														placeholder='Work Address'
														rows={2}
														className='w-full resize-none rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all focus:border-[rgba(var(--accent-rgb),0.5)]'
													/>
												</div>

												{/* Personal */}
												<div className='space-y-3 pt-2 border-t border-[var(--border-subtle)]'>
													<div className='flex items-center gap-2 px-1'>
														<Calendar className='h-3.5 w-3.5 text-[var(--text-tertiary)]' />
														<span className='text-[11px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>Personal</span>
													</div>
													<div className='grid grid-cols-2 gap-3'>
														<div className='space-y-1'>
															<label className='px-1 text-[10px] text-[var(--text-tertiary)]'>{t('contacts:fields.birthday')}</label>
															<input
																type='date'
																value={birthday}
																onChange={(e) => setBirthday(e.target.value)}
																className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all [color-scheme:dark] focus:border-[rgba(var(--accent-rgb),0.5)]'
															/>
														</div>
														<div className='space-y-1'>
															<label className='px-1 text-[10px] text-[var(--text-tertiary)]'>{t('contacts:fields.anniversary')}</label>
															<input
																type='date'
																value={anniversary}
																onChange={(e) => setAnniversary(e.target.value)}
																className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-active)] px-3 py-2 text-[12px] text-[var(--text-primary)] outline-none transition-all [color-scheme:dark] focus:border-[rgba(var(--accent-rgb),0.5)]'
															/>
														</div>
													</div>
													<div className='space-y-1'>
														<label className='px-1 text-[10px] text-[var(--text-tertiary)]'>{t('contacts:fields.gender')}</label>
														<div className='flex gap-4 px-2'>
															{['M', 'F', 'O'].map((g) => (
																<label key={g} className='flex items-center gap-2 cursor-pointer'>
																	<input
																		type='radio'
																		name='gender'
																		value={g}
																		checked={gender === g}
																		onChange={(e) => setGender(e.target.value)}
																		className='accent-[rgb(var(--accent-rgb))]'
																	/>
																	<span className='text-[12px] text-[var(--text-secondary)]'>
																		{g === 'M' ? t('contacts:fields.genderMale') : g === 'F' ? t('contacts:fields.genderFemale') : t('contacts:fields.genderOther')}
																	</span>
																</label>
															))}
														</div>
													</div>
												</div>
											</motion.div>
										)}
									</AnimatePresence>

									{/* Notes Field (outside advanced) */}
									<div className='group relative pt-2 border-t border-[var(--border-subtle)]'>
										<div className='pointer-events-none absolute top-4 left-3'>
											<FileText className='h-4 w-4 text-[var(--text-tertiary)] transition-colors group-focus-within:text-[rgb(var(--accent-rgb))]' />
										</div>
										<textarea
											value={notes}
											onChange={(e) => setNotes(e.target.value)}
											placeholder={t('notes:placeholder')}
											rows={2}
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
										<span>{contact ? t('common:actions.save') : t('common:actions.create')}</span>
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
