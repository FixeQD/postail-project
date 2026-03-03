import { useState, useEffect, useRef } from 'react'
import { createPortal } from 'react-dom'
import { invoke } from '@tauri-apps/api/core'
import {
	Inbox,
	Send,
	FileText,
	Trash2,
	Archive,
	AlertOctagon,
	Star,
	Layers,
	FolderOpen,
	Check,
	Loader2,
	ChevronDown,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useShellTransition } from '@/hooks/useShellTransition'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import type { Mailbox } from '@/types/mail'

const ROLE_ICONS = {
	inbox: Inbox,
	sent: Send,
	drafts: FileText,
	trash: Trash2,
	archive: Archive,
	junk: AlertOctagon,
	flagged: Star,
	all: Layers,
	other: FolderOpen,
} as const

type Role = keyof typeof ROLE_ICONS

function RoleIcon({ role, className }: { role: string; className?: string }) {
	const Icon = ROLE_ICONS[role as Role] ?? FolderOpen
	return <Icon className={className} />
}

function RoleSelect({ value, onChange }: { value: Role; onChange: (role: Role) => void }) {
	const { t } = useSettingsTranslation()
	const [open, setOpen] = useState(false)
	const [coords, setCoords] = useState({ top: 0, right: 0 })
	const buttonRef = useRef<HTMLButtonElement>(null)
	const roles = Object.keys(ROLE_ICONS) as Role[]

	const handleOpen = () => {
		if (buttonRef.current) {
			const rect = buttonRef.current.getBoundingClientRect()
			setCoords({ top: rect.bottom + 6, right: window.innerWidth - rect.right })
		}
		setOpen((o) => !o)
	}

	return (
		<div className='relative'>
			<button
				ref={buttonRef}
				type='button'
				onClick={handleOpen}
				className='flex items-center gap-2 rounded-lg border border-white/[0.08] bg-slate-800/60 px-3 py-1.5 text-sm text-slate-200 transition-colors hover:border-white/[0.14] hover:bg-slate-700/60'>
				<RoleIcon role={value} className='h-3.5 w-3.5 text-slate-400' />
				<span>{t(`settings:mailboxRoles.roles.${value}`)}</span>
				<ChevronDown
					className={`h-3.5 w-3.5 text-slate-500 transition-transform ${open ? 'rotate-180' : ''}`}
				/>
			</button>

			{open &&
				createPortal(
					<>
						<div className='fixed inset-0 z-50' onClick={() => setOpen(false)} />
						<div
							className='fixed z-50 w-44 overflow-hidden rounded-xl border border-white/[0.08] bg-slate-900 shadow-2xl'
							style={{ top: coords.top, right: coords.right }}>
							{roles.map((role) => {
								const Icon = ROLE_ICONS[role]
								return (
									<button
										key={role}
										type='button'
										onClick={() => {
											onChange(role)
											setOpen(false)
										}}
										className={`flex w-full items-center gap-2.5 px-3 py-2 text-sm transition-colors hover:bg-white/[0.06] ${
											role === value ? 'text-slate-100' : 'text-slate-400'
										}`}>
										<Icon className='h-3.5 w-3.5 shrink-0' />
										<span>{t(`settings:mailboxRoles.roles.${role}`)}</span>
										{role === value && (
											<Check className='ml-auto h-3.5 w-3.5 text-green-400' />
										)}
									</button>
								)
							})}
						</div>
					</>,
					document.body
				)}
		</div>
	)
}

interface MailboxRoleStepProps {
	accountId: string
	onDone: () => void
	initialMailboxes?: Mailbox[]
}

function buildRoles(mbs: Mailbox[]): Record<string, Role> {
	const initial: Record<string, Role> = {}
	for (const mb of mbs) {
		initial[mb.name] = (mb.role as Role) || 'other'
	}
	return initial
}

export function MailboxRoleStep({ accountId, onDone, initialMailboxes }: MailboxRoleStepProps) {
	const { t } = useSettingsTranslation()
	const [mailboxes, setMailboxes] = useState<Mailbox[]>(initialMailboxes ?? [])
	const [roles, setRoles] = useState<Record<string, Role>>(
		initialMailboxes ? buildRoles(initialMailboxes) : {}
	)
	const [loading, setLoading] = useState(!initialMailboxes)
	const [saving, setSaving] = useState(false)

	const { shellScope, contentScope, transition } = useShellTransition()

	useEffect(() => {
		if (initialMailboxes) return
		invoke<Mailbox[]>('fetch_mailboxes', { accountId })
			.then((mbs) => {
				setMailboxes(mbs)
				setRoles(buildRoles(mbs))
			})
			.catch(console.error)
			.finally(() => {
				transition(() => setLoading(false))
			})
	}, [accountId, transition, initialMailboxes])

	const handleRoleChange = (mailboxName: string, role: Role) => {
		setRoles((prev) => ({ ...prev, [mailboxName]: role }))
	}

	const handleSave = async () => {
		setSaving(true)
		try {
			const changed = mailboxes.filter((mb) => roles[mb.name] && roles[mb.name] !== mb.role)
			await Promise.all(
				changed.map((mb) =>
					invoke('update_mailbox_role', {
						accountId,
						mailboxName: mb.name,
						role: roles[mb.name],
					})
				)
			)
		} catch (err) {
			console.error('Failed to save mailbox roles:', err)
		} finally {
			setSaving(false)
			onDone()
		}
	}

	return (
		<div ref={shellScope} className='w-full'>
			<div ref={contentScope} className='flex flex-col gap-5'>
				<div>
					<h2 className='text-lg font-semibold text-slate-100'>
						{t('settings:mailboxRoles.title')}
					</h2>
					<p className='mt-1 text-sm text-slate-400'>
						{t('settings:mailboxRoles.subtitle')}
					</p>
				</div>

				<div
					className='max-h-[360px] overflow-y-auto pr-3'
					style={{ scrollbarGutter: 'stable' }}>
					<div className='flex flex-col gap-1.5'>
						{loading ? (
							<div className='flex items-center justify-center py-10'>
								<Loader2 className='h-5 w-5 animate-spin text-slate-500' />
							</div>
						) : mailboxes.length === 0 ? (
							<p className='py-6 text-center text-sm text-slate-500'>
								{t('settings:mailboxRoles.noMailboxes')}
							</p>
						) : (
							mailboxes.map((mb) => (
								<div
									key={mb.name}
									className='flex items-center justify-between gap-3 rounded-xl border border-white/[0.05] bg-white/[0.03] px-4 py-2.5'>
									<div className='flex min-w-0 items-center gap-3'>
										<RoleIcon
											role={roles[mb.name] ?? mb.role}
											className='h-4 w-4 shrink-0 text-slate-400'
										/>
										<span
											className='truncate text-sm text-slate-200'
											title={mb.display_name || mb.name}>
											{mb.display_name || mb.name}
										</span>
									</div>
									<RoleSelect
										value={(roles[mb.name] as Role) ?? 'other'}
										onChange={(role) => handleRoleChange(mb.name, role)}
									/>
								</div>
							))
						)}
					</div>
				</div>

				<div className='flex gap-3 border-t border-white/[0.06] pt-4'>
					<Button
						type='button'
						variant='outline'
						onClick={onDone}
						disabled={saving}
						className='flex-1 border-slate-700 bg-slate-800/50 hover:bg-slate-800'>
						{t('settings:mailboxRoles.skip')}
					</Button>
					<Button
						type='button'
						onClick={handleSave}
						disabled={saving || loading}
						className='flex-1'
						style={{
							background: `linear-gradient(to right, var(--accent-dark), var(--accent-color))`,
						}}>
						{saving ? (
							<>
								<Loader2 className='mr-2 h-4 w-4 animate-spin' />
								{t('settings:mailboxRoles.saving')}
							</>
						) : (
							<>
								<Check className='mr-2 h-4 w-4' />
								{t('settings:mailboxRoles.confirm')}
							</>
						)}
					</Button>
				</div>
			</div>
		</div>
	)
}
