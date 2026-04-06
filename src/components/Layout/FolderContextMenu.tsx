import { useState, useRef, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useQueryClient } from '@tanstack/react-query'
import { motion, AnimatePresence } from 'framer-motion'
import { FolderPlus, Pencil, Trash2, MoreHorizontal, EyeOff, Eye } from 'lucide-react'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
	Dialog,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { toast } from '@/stores/toastStore'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import type { Mailbox } from '@/types/mail'

const SYSTEM_ROLES = ['inbox', 'sent', 'drafts', 'trash', 'archive', 'junk', 'flagged', 'all']

interface FolderContextMenuProps {
	mailbox: Mailbox
	accountId: string
	activeMailbox: string
	onMailboxSelect: (name: string) => void
	shortName?: string
	parentPrefix?: string
	children: React.ReactNode
}

interface FolderNameDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	title: string
	initialValue?: string
	placeholder?: string
	confirmLabel: string
	onConfirm: (name: string) => Promise<void>
	separator?: string
}

export function FolderNameDialog({
	open,
	onOpenChange,
	title,
	initialValue = '',
	placeholder = 'Folder name',
	confirmLabel,
	onConfirm,
	separator = '/',
}: FolderNameDialogProps) {
	const { t } = useTypedTranslation('inbox')
	const [value, setValue] = useState(initialValue)
	const [loading, setLoading] = useState(false)
	const inputRef = useRef<HTMLInputElement>(null)

	useEffect(() => {
		if (open) {
			setValue(initialValue)
			setTimeout(() => inputRef.current?.focus(), 50)
		}
	}, [open, initialValue])

	const handleConfirm = async () => {
		const trimmed = value.trim()
		if (!trimmed) return
		setLoading(true)
		try {
			await onConfirm(trimmed)
			onOpenChange(false)
		} catch (e) {
			// Dialog stays open on error
		} finally {
			setLoading(false)
		}
	}

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className='border-[var(--border-subtle)] bg-[var(--surface-glass)] p-0 text-[var(--text-primary)] shadow-2xl sm:max-w-[340px]'>
				<AnimatePresence>
					{open && (
						<motion.div
							key='folder-name-dialog'
							initial={{ opacity: 0, scale: 0.95, y: -8 }}
							animate={{ opacity: 1, scale: 1, y: 0 }}
							transition={{ duration: 0.18, ease: 'circOut' }}>
							{/* accent bar */}
							<motion.div
								initial={{ scaleX: 0, opacity: 0 }}
								animate={{ scaleX: 1, opacity: 1 }}
								transition={{ duration: 0.3, ease: 'circOut' }}
								className='h-[3px] w-full origin-left rounded-t-lg'
								style={{
									background: `linear-gradient(90deg, var(--accent-color), var(--accent-light))`,
								}}
							/>

							<div className='px-5 pt-5 pb-5'>
								<DialogHeader className='mb-4'>
									<DialogTitle className='text-sm font-semibold tracking-tight text-[var(--text-primary)]'>
										{title}
									</DialogTitle>
								</DialogHeader>

								<Input
									ref={inputRef}
									value={value}
									onChange={(e) => setValue(e.target.value)}
									placeholder={placeholder}
									className='h-8 border-[var(--border-subtle)] bg-[var(--surface-panel)] text-sm text-[var(--text-primary)] placeholder:text-[var(--text-tertiary)] focus-visible:ring-1 focus-visible:ring-[var(--accent-color)]'
									onKeyDown={(e) => {
										if (e.key === 'Enter') handleConfirm()
										if (e.key === 'Escape') onOpenChange(false)
									}}
								/>

								<AnimatePresence>
									{value.includes(separator) && (
										<motion.div
											initial={{ opacity: 0, height: 0, marginTop: 0 }}
											animate={{ opacity: 1, height: 'auto', marginTop: 12 }}
											exit={{ opacity: 0, height: 0, marginTop: 0 }}
											className='text-[11px] leading-tight font-medium text-amber-500/90'>
											{t('inbox:folderMenu.nestedWarning', {
												defaultValue: `Using the delimiter ("${separator}") will create a nested folder structure.`,
												separator,
											})}
										</motion.div>
									)}
								</AnimatePresence>

								<DialogFooter className='mt-4 flex gap-2'>
									<Button
										variant='ghost'
										onClick={() => onOpenChange(false)}
										disabled={loading}
										className='flex-1 border border-[var(--border-subtle)] bg-[var(--surface-panel)] text-xs text-[var(--text-secondary)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
										{t('inbox:folderMenu.cancel')}
									</Button>
									<Button
										onClick={handleConfirm}
										disabled={!value.trim() || loading}
										className='flex-1 border-0 text-xs font-medium text-white shadow-lg disabled:opacity-40'
										style={{
											background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
											boxShadow: `0 4px 16px rgba(var(--accent-rgb), 0.3)`,
											color: 'var(--accent-text)',
										}}>
										{loading ? '…' : confirmLabel}
									</Button>
								</DialogFooter>
							</div>
						</motion.div>
					)}
				</AnimatePresence>
			</DialogContent>
		</Dialog>
	)
}

export function FolderContextMenu({
	mailbox,
	accountId,
	activeMailbox,
	onMailboxSelect,
	shortName: propShortName,
	parentPrefix: propParentPrefix,
	children,
}: FolderContextMenuProps) {
	const { t } = useTypedTranslation('inbox')
	const [open, setOpen] = useState(false)
	const [renameOpen, setRenameOpen] = useState(false)
	const [createSubOpen, setCreateSubOpen] = useState(false)
	const [deleteOpen, setDeleteOpen] = useState(false)
	const [deleting, setDeleting] = useState(false)
	const [isDragOver, setIsDragOver] = useState(false)
	const qc = useQueryClient()

	const isSystem = SYSTEM_ROLES.includes(mailbox.role)
	const childElement = children as any
	const shortName = propShortName || childElement?.props?.shortName || mailbox.display_name
	const parentPrefix = propParentPrefix ?? ''

	const invalidate = useCallback(() => {
		qc.invalidateQueries({ queryKey: ['mailboxes', accountId] })
	}, [qc, accountId])

	const handleDragOver = (e: React.DragEvent) => {
		if (!e.dataTransfer.types.includes('application/postail-message')) return
		if (
			mailbox.role === 'sent' ||
			mailbox.role === 'drafts' ||
			mailbox.name.startsWith('Virtual_')
		)
			return
		e.preventDefault()
		e.dataTransfer.dropEffect = 'move'
		setIsDragOver(true)
	}

	const handleDragLeave = (e: React.DragEvent<HTMLDivElement>) => {
		if (e.currentTarget.contains(e.relatedTarget as Node)) return
		setIsDragOver(false)
	}

	const handleDrop = async (e: React.DragEvent) => {
		e.preventDefault()
		setIsDragOver(false)
		if (
			mailbox.role === 'sent' ||
			mailbox.role === 'drafts' ||
			mailbox.name.startsWith('Virtual_')
		)
			return
		const raw = e.dataTransfer.getData('application/postail-message')
		if (!raw) return
		const payload = JSON.parse(raw) as {
			accountId: string
			mailbox: string
			uid: number
			message?: any
		}
		if (payload.mailbox === mailbox.name) return

		try {
			// Optimistically remove from source
			qc.setQueryData(['messages', payload.accountId, payload.mailbox], (old: any) => {
				if (!old?.pages) return old
				return {
					...old,
					pages: old.pages.map((page: any[]) =>
						page.filter((m: any) => m.uid !== payload.uid)
					),
				}
			})

			// Optimistically add to target
			if (payload.message) {
				const movedMsg = {
					...payload.message,
					mailbox: mailbox.name,
					uid: -payload.uid,
				}
				qc.setQueryData(['messages', payload.accountId, mailbox.name], (old: any) => {
					if (!old?.pages) return old
					const newPages = [...old.pages]
					if (newPages.length > 0) {
						newPages[0] = [movedMsg, ...newPages[0]]
					}
					return { ...old, pages: newPages }
				})
			}

			await invoke('move_messages', {
				accountId: payload.accountId,
				sourceMailbox: payload.mailbox,
				targetMailbox: mailbox.name,
				uids: [payload.uid],
			})

			toast.success(t('inbox:folderMenu.movedTo', { name: shortName }))
		} catch (err) {
			toast.error(String(err))
		} finally {
			qc.invalidateQueries({ queryKey: ['messages', payload.accountId, payload.mailbox] })
			qc.invalidateQueries({ queryKey: ['messages', payload.accountId, mailbox.name] })
		}
	}

	const handleRename = async (newName: string) => {
		const fullNewName = parentPrefix + newName
		try {
			await invoke('rename_folder', {
				accountId,
				oldName: mailbox.name,
				newName: fullNewName,
			})
			// if user was looking at the renamed folder, update selection
			if (activeMailbox === mailbox.name) {
				onMailboxSelect(fullNewName)
			}
			invalidate()
			toast.success(t('inbox:folderMenu.renamed', { name: newName }))
		} catch (e) {
			toast.error(String(e))
			return Promise.reject(e)
		}
	}

	const handleCreateSub = async (subName: string) => {
		try {
			await invoke('create_subfolder', {
				accountId,
				parentName: mailbox.name,
				childName: subName,
			})
			invalidate()
			toast.success(t('inbox:folderMenu.created', { name: subName }))
		} catch (e) {
			toast.error(String(e))
			return Promise.reject(e)
		}
	}

	const handleToggleHidden = async () => {
		try {
			await invoke('set_folder_hidden', {
				accountId,
				name: mailbox.name,
				hidden: !mailbox.hidden,
			})
			if (!mailbox.hidden && activeMailbox === mailbox.name) {
				onMailboxSelect('INBOX')
			}
			invalidate()
			toast.success(
				mailbox.hidden ? t('inbox:folderMenu.shown') : t('inbox:folderMenu.hidden')
			)
		} catch (err) {
			toast.error(String(err))
		}
	}

	const handleDelete = async () => {
		setDeleting(true)
		try {
			await invoke('delete_folder', { accountId, name: mailbox.name })
			// if deleted folder was active, go back to inbox
			if (activeMailbox === mailbox.name) {
				onMailboxSelect('INBOX')
			}
			invalidate()
			toast.success(t('inbox:folderMenu.deleted', { name: shortName }))
			setDeleteOpen(false)
		} catch (e) {
			toast.error(String(e))
		} finally {
			setDeleting(false)
		}
	}

	return (
		<>
			<DropdownMenu open={open} onOpenChange={setOpen}>
				<div
					className='group/folder-item relative w-full'
					onContextMenu={(e) => {
						e.preventDefault()
						setOpen(true)
					}}
					onDragOver={handleDragOver}
					onDragLeave={handleDragLeave}
					onDrop={handleDrop}
					style={
						isDragOver
							? {
									outline: '2px solid rgba(var(--accent-rgb), 0.6)',
									outlineOffset: '-2px',
									backgroundColor: 'rgba(var(--accent-rgb), 0.08)',
									borderRadius: '0.75rem',
								}
							: undefined
					}>
					{children}

					<DropdownMenuTrigger asChild>
						<motion.button
							type='button'
							onClick={(e) => {
								e.stopPropagation()
								setOpen(true)
							}}
							initial={false}
							className='absolute top-1/2 right-1.5 flex h-5 w-5 -translate-y-1/2 items-center justify-center rounded-md text-[var(--text-tertiary)] opacity-0 transition-all duration-150 group-hover/folder-item:opacity-100 hover:bg-[var(--surface-hover)] hover:text-[var(--text-secondary)]'
							aria-label={t('inbox:folderMenu.ariaOptions')}>
							<MoreHorizontal className='h-3.5 w-3.5' />
						</motion.button>
					</DropdownMenuTrigger>
				</div>

				<DropdownMenuContent
					className='w-44 overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-glass)] p-1 shadow-xl backdrop-blur-xl'
					align='start'
					sideOffset={2}>
					<DropdownMenuItem
						onClick={() => setCreateSubOpen(true)}
						className='flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-1.5 text-xs text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] focus:bg-[var(--surface-hover)] focus:text-[var(--text-primary)]'>
						<FolderPlus className='h-3.5 w-3.5 shrink-0 text-[var(--text-tertiary)]' />
						{t('inbox:folderMenu.newSubfolder')}
					</DropdownMenuItem>

					{!isSystem && (
						<>
							<DropdownMenuItem
								onClick={handleToggleHidden}
								className='flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-1.5 text-xs text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] focus:bg-[var(--surface-hover)] focus:text-[var(--text-primary)]'>
								{mailbox.hidden ? (
									<Eye className='h-3.5 w-3.5 shrink-0 text-[var(--text-tertiary)]' />
								) : (
									<EyeOff className='h-3.5 w-3.5 shrink-0 text-[var(--text-tertiary)]' />
								)}
								{mailbox.hidden
									? t('inbox:folderMenu.show')
									: t('inbox:folderMenu.hide')}
							</DropdownMenuItem>

							<DropdownMenuItem
								onClick={() => setRenameOpen(true)}
								className='flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-1.5 text-xs text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] focus:bg-[var(--surface-hover)] focus:text-[var(--text-primary)]'>
								<Pencil className='h-3.5 w-3.5 shrink-0 text-[var(--text-tertiary)]' />
								{t('inbox:folderMenu.rename')}
							</DropdownMenuItem>

							<DropdownMenuSeparator className='my-1 h-px bg-[var(--border-faint)]' />

							<DropdownMenuItem
								onClick={() => setDeleteOpen(true)}
								className='flex cursor-pointer items-center gap-2 rounded-lg px-2.5 py-1.5 text-xs text-red-400 transition-colors hover:bg-red-500/10 hover:text-red-400 focus:bg-red-500/10 focus:text-red-400'>
								<Trash2 className='h-3.5 w-3.5 shrink-0' />
								{t('inbox:folderMenu.delete')}
							</DropdownMenuItem>
						</>
					)}
				</DropdownMenuContent>
			</DropdownMenu>

			<FolderNameDialog
				open={renameOpen}
				onOpenChange={setRenameOpen}
				title={t('inbox:folderMenu.renameTitle', { name: shortName })}
				initialValue={shortName}
				placeholder={t('inbox:folderMenu.renamePlaceholder')}
				confirmLabel={t('inbox:folderMenu.renameConfirm')}
				onConfirm={handleRename}
				separator={mailbox.separator}
			/>

			<FolderNameDialog
				open={createSubOpen}
				onOpenChange={setCreateSubOpen}
				title={t('inbox:folderMenu.subfolderTitle', { name: shortName })}
				placeholder={t('inbox:folderMenu.subfolderPlaceholder')}
				confirmLabel={t('inbox:folderMenu.subfolderConfirm')}
				onConfirm={handleCreateSub}
				separator={mailbox.separator}
			/>

			<ConfirmationDialog
				open={deleteOpen}
				onOpenChange={setDeleteOpen}
				title={t('inbox:folderMenu.deleteTitle')}
				description={t('inbox:folderMenu.deleteDescription', {
					name: shortName,
				})}
				confirmLabel={
					deleting ? t('inbox:folderMenu.deleting') : t('inbox:folderMenu.deleteConfirm')
				}
				cancelLabel={t('inbox:folderMenu.cancel')}
				confirmClassName='w-full border-0 bg-red-500 font-medium text-white shadow-lg hover:bg-red-600'
				onConfirm={handleDelete}
			/>
		</>
	)
}
