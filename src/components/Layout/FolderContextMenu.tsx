import { useState, useRef, useEffect, useCallback } from 'react'
import { createPortal } from 'react-dom'
import { invoke } from '@tauri-apps/api/core'
import { useQueryClient } from '@tanstack/react-query'
import { motion, AnimatePresence } from 'framer-motion'
import { FolderPlus, Pencil, Trash2, MoreHorizontal, EyeOff, Eye } from 'lucide-react'
import {
	Dialog,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
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

// ─── Folder Name Dialog ───────────────────────────────────────────────────────

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
		} catch {
			// stay open
		} finally {
			setLoading(false)
		}
	}

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent
				showCloseButton={false}
				className='border-[var(--border-subtle)] bg-[var(--surface-glass)] p-0 shadow-2xl sm:max-w-[320px]'>
				{/* Accent bar */}
				<div
					className='h-[2px] w-full rounded-t-lg'
					style={{
						background: `linear-gradient(90deg, var(--accent-color), var(--accent-light))`,
					}}
				/>

				<div className='px-4 pt-4 pb-4'>
					<p className='mb-3 text-[13px] font-semibold text-[var(--text-primary)]'>
						{title}
					</p>

					<input
						ref={inputRef}
						value={value}
						onChange={(e) => setValue(e.target.value)}
						placeholder={placeholder}
						className='w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] px-3 py-1.5 text-[13px] text-[var(--text-primary)] transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:border-[var(--accent-color)] focus:ring-1 focus:ring-[var(--accent-color)]/30'
						onKeyDown={(e) => {
							if (e.key === 'Enter') handleConfirm()
							if (e.key === 'Escape') onOpenChange(false)
						}}
					/>

					<AnimatePresence>
						{value.includes(separator) && (
							<motion.p
								initial={{ opacity: 0, height: 0, marginTop: 0 }}
								animate={{ opacity: 1, height: 'auto', marginTop: 8 }}
								exit={{ opacity: 0, height: 0, marginTop: 0 }}
								className='text-[11px] leading-snug text-amber-500/80'>
								{t('inbox:folderMenu.nestedWarning', {
									defaultValue: `"${separator}" creates a nested folder.`,
									separator,
								})}
							</motion.p>
						)}
					</AnimatePresence>

					<div className='mt-4 flex gap-2'>
						<button
							type='button'
							onClick={() => onOpenChange(false)}
							disabled={loading}
							className='flex-1 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] py-1.5 text-[12px] font-medium text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] disabled:opacity-50'>
							{t('inbox:folderMenu.cancel')}
						</button>
						<button
							type='button'
							onClick={handleConfirm}
							disabled={!value.trim() || loading}
							className='flex-1 rounded-lg py-1.5 text-[12px] font-semibold text-white transition-opacity disabled:opacity-40'
							style={{
								background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
								boxShadow: `0 3px 12px rgba(var(--accent-rgb), 0.3)`,
								color: 'var(--accent-text)',
							}}>
							{loading ? '…' : confirmLabel}
						</button>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	)
}

// ─── Custom floating menu ─────────────────────────────────────────────────────

interface MenuPos {
	top: number
	left: number
}

interface FloatingMenuProps {
	open: boolean
	anchorRef: React.RefObject<HTMLElement | null>
	onClose: () => void
	children: React.ReactNode
}

function FloatingMenu({ open, anchorRef, onClose, children }: FloatingMenuProps) {
	const menuRef = useRef<HTMLDivElement>(null)
	const [pos, setPos] = useState<MenuPos | null>(null)

	useLayoutEffect(() => {
		if (!open) {
			setPos(null)
			return
		}
		const anchor = anchorRef.current
		const menu = menuRef.current
		if (!anchor || !menu) return
		const r = anchor.getBoundingClientRect()
		const mh = menu.getBoundingClientRect().height || 160
		const mw = menu.getBoundingClientRect().width || 176
		const vw = window.innerWidth,
			vh = window.innerHeight
		const top = r.bottom + 4 + mh > vh ? r.top - mh - 4 : r.bottom + 4
		const left = Math.min(r.left, vw - mw - 8)
		setPos({ top, left })
	}, [open])

	useEffect(() => {
		if (!open) return
		const handler = (e: MouseEvent) => {
			if (menuRef.current?.contains(e.target as Node)) return
			if (anchorRef.current?.contains(e.target as Node)) return
			onClose()
		}
		const esc = (e: KeyboardEvent) => {
			if (e.key === 'Escape') onClose()
		}
		document.addEventListener('mousedown', handler)
		document.addEventListener('keydown', esc)
		return () => {
			document.removeEventListener('mousedown', handler)
			document.removeEventListener('keydown', esc)
		}
	}, [open, onClose])

	if (!open) return null

	return createPortal(
		<div
			ref={menuRef}
			style={
				pos
					? { position: 'fixed', top: pos.top, left: pos.left }
					: { position: 'fixed', top: -9999, left: -9999 }
			}
			className='z-[200] w-44 overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-glass)] p-1 shadow-2xl backdrop-blur-xl'>
			<motion.div
				initial={{ opacity: 0, scale: 0.95, y: -4 }}
				animate={{ opacity: 1, scale: 1, y: 0 }}
				transition={{ duration: 0.12, ease: [0.16, 1, 0.3, 1] }}>
				{children}
			</motion.div>
		</div>,
		document.body
	)
}

function useLayoutEffect(fn: () => void | (() => void), deps: any[]) {
	useEffect(fn, deps)
}

// ─── Menu items ───────────────────────────────────────────────────────────────

function MenuItem({
	icon,
	label,
	onClick,
	danger = false,
}: {
	icon: React.ReactNode
	label: string
	onClick: () => void
	danger?: boolean
}) {
	return (
		<button
			type='button'
			onClick={onClick}
			className={`flex w-full cursor-pointer items-center gap-2 rounded-lg px-2.5 py-[6px] text-left text-[12px] font-medium transition-colors ${
				danger
					? 'text-red-400 hover:bg-red-500/10'
					: 'text-[var(--text-secondary)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
			}`}>
			<span className={`shrink-0 ${danger ? 'text-red-400' : 'text-[var(--text-tertiary)]'}`}>
				{icon}
			</span>
			{label}
		</button>
	)
}

function MenuDivider() {
	return <div className='my-1 h-px bg-[var(--border-faint)]' />
}

// ─── Main component ───────────────────────────────────────────────────────────

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
	const [menuOpen, setMenuOpen] = useState(false)
	const [renameOpen, setRenameOpen] = useState(false)
	const [createSubOpen, setCreateSubOpen] = useState(false)
	const [deleteOpen, setDeleteOpen] = useState(false)
	const [deleting, setDeleting] = useState(false)
	const [isDragOver, setIsDragOver] = useState(false)
	const triggerRef = useRef<HTMLButtonElement | null>(null)
	const qc = useQueryClient()

	const isSystem = SYSTEM_ROLES.includes(mailbox.role)
	const childElement = children as any
	const shortName = propShortName || childElement?.props?.shortName || mailbox.display_name
	const parentPrefix = propParentPrefix ?? ''

	const invalidate = useCallback(() => {
		qc.invalidateQueries({ queryKey: ['mailboxes', accountId] })
	}, [qc, accountId])

	const closeMenu = useCallback(() => setMenuOpen(false), [])

	// ── Drag & drop ──────────────────────────────────────────────────────────
	const canDrop =
		mailbox.role !== 'sent' && mailbox.role !== 'drafts' && !mailbox.name.startsWith('Virtual_')

	const handleDragOver = (e: React.DragEvent) => {
		if (!canDrop) return
		if (!e.dataTransfer.types.includes('application/postail-message')) return
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
		if (!canDrop) return
		const raw = e.dataTransfer.getData('application/postail-message')
		if (!raw) return
		let payload: { accountId: string; mailbox: string; uid: number; message?: any }
		try {
			payload = JSON.parse(raw)
		} catch {
			return
		}
		if (payload.mailbox === mailbox.name) return
		try {
			qc.setQueryData(['messages', payload.accountId, payload.mailbox], (old: any) => {
				if (!old?.pages) return old
				return {
					...old,
					pages: old.pages.map((p: any[]) => p.filter((m: any) => m.uid !== payload.uid)),
				}
			})
			if (payload.message) {
				const moved = { ...payload.message, mailbox: mailbox.name, uid: -payload.uid }
				qc.setQueryData(['messages', payload.accountId, mailbox.name], (old: any) => {
					if (!old?.pages) return old
					const pages = [...old.pages]
					if (pages.length > 0) pages[0] = [moved, ...pages[0]]
					return { ...old, pages }
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

	// ── Actions ──────────────────────────────────────────────────────────────
	const handleRename = async (newName: string) => {
		const fullNewName = parentPrefix + newName
		try {
			await invoke('rename_folder', {
				accountId,
				oldName: mailbox.name,
				newName: fullNewName,
			})
			if (activeMailbox === mailbox.name) onMailboxSelect(fullNewName)
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
			if (!mailbox.hidden && activeMailbox === mailbox.name) onMailboxSelect('INBOX')
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
			if (activeMailbox === mailbox.name) onMailboxSelect('INBOX')
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
			<div
				className='group/folder-item relative w-full'
				onContextMenu={(e) => {
					e.preventDefault()
					setMenuOpen(true)
				}}
				onDragOver={handleDragOver}
				onDragLeave={handleDragLeave}
				onDrop={handleDrop}
				style={
					isDragOver
						? {
								outline: '1.5px solid rgba(var(--accent-rgb), 0.5)',
								outlineOffset: '-1px',
								backgroundColor: 'rgba(var(--accent-rgb), 0.06)',
								borderRadius: '8px',
							}
						: undefined
				}>
				{children}

				{/* ··· button */}
				<button
					ref={triggerRef}
					type='button'
					onClick={(e) => {
						e.stopPropagation()
						setMenuOpen((v) => !v)
					}}
					aria-label={t('inbox:folderMenu.ariaOptions')}
					className='absolute top-1/2 right-1 flex h-5 w-5 -translate-y-1/2 items-center justify-center rounded-md text-[var(--text-tertiary)] opacity-0 transition-all duration-100 group-hover/folder-item:opacity-100 hover:bg-[var(--surface-hover)] hover:text-[var(--text-secondary)]'>
					<MoreHorizontal className='h-3.5 w-3.5' />
				</button>
			</div>

			{/* Floating menu */}
			<FloatingMenu
				open={menuOpen}
				anchorRef={triggerRef as React.RefObject<HTMLElement>}
				onClose={closeMenu}>
				<MenuItem
					icon={<FolderPlus className='h-3.5 w-3.5' />}
					label={t('inbox:folderMenu.newSubfolder')}
					onClick={() => {
						closeMenu()
						setCreateSubOpen(true)
					}}
				/>
				{!isSystem && (
					<>
						<MenuItem
							icon={
								mailbox.hidden ? (
									<Eye className='h-3.5 w-3.5' />
								) : (
									<EyeOff className='h-3.5 w-3.5' />
								)
							}
							label={
								mailbox.hidden
									? t('inbox:folderMenu.show')
									: t('inbox:folderMenu.hide')
							}
							onClick={() => {
								closeMenu()
								handleToggleHidden()
							}}
						/>
						<MenuItem
							icon={<Pencil className='h-3.5 w-3.5' />}
							label={t('inbox:folderMenu.rename')}
							onClick={() => {
								closeMenu()
								setRenameOpen(true)
							}}
						/>
						<MenuDivider />
						<MenuItem
							icon={<Trash2 className='h-3.5 w-3.5' />}
							label={t('inbox:folderMenu.delete')}
							onClick={() => {
								closeMenu()
								setDeleteOpen(true)
							}}
							danger
						/>
					</>
				)}
			</FloatingMenu>

			{/* Dialogs */}
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
				description={t('inbox:folderMenu.deleteDescription', { name: shortName })}
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
