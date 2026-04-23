import { useRef, useState, useEffect, useMemo, memo, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import {
	Inbox,
	Send,
	Trash2,
	Archive,
	File,
	Pencil,
	Star,
	AlertTriangle,
	Layers,
	Tag,
	Folder,
	Plus,
	ChevronRight,
	BookmarkCheck,
	Users,
} from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import type { Mailbox } from '@/types/mail'
import type { AccountMeta } from '@/types/accounts'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { FolderContextMenu, FolderNameDialog } from './FolderContextMenu'
import { toast } from '@/stores/toastStore'

interface SidebarProps {
	activeAccount: AccountMeta | null
	activeMailbox: string
	onMailboxSelect: (mailbox: string) => void
	onCompose: () => void
}

const MIN_WIDTH = 64
const MAX_WIDTH = 300
const DEFAULT_WIDTH = 240

const ROLE_ORDER = [
	'inbox',
	'flagged',
	'sent',
	'drafts',
	'archive',
	'junk',
	'trash',
	'all',
	'important',
]

interface MailboxItemProps {
	mailbox: Mailbox
	isActive: boolean
	isCollapsed: boolean
	accentColor: string
	animationsEnabled: boolean
	onSelect: (name: string) => void
	depth?: number
	shortName?: string
	index?: number
}

function getIcon(role: string, isActive: boolean) {
	const cls = 'h-[15px] w-[15px] shrink-0'
	switch (role) {
		case 'inbox':
			return <Inbox className={cls} />
		case 'sent':
			return <Send className={cls} />
		case 'trash':
			return <Trash2 className={cls} />
		case 'archive':
			return <Archive className={cls} />
		case 'drafts':
			return <File className={cls} />
		case 'junk':
			return <AlertTriangle className={cls} />
		case 'flagged':
			return <Star className={cls} />
		case 'all':
			return <Layers className={cls} />
		case 'important':
			return <BookmarkCheck className={cls} />
		case 'tag':
			return <Tag className={cls} />
		default:
			return <Folder className={cls} />
	}
}

const MailboxItem = memo(
	({
		mailbox,
		isActive,
		isCollapsed,
		accentColor,
		animationsEnabled,
		onSelect,
		depth = 0,
		shortName,
		index = 0,
	}: MailboxItemProps) => {
		const label = shortName || mailbox.display_name

		return (
			<motion.button
				type='button'
				onClick={() => onSelect(mailbox.name)}
				title={isCollapsed ? label : undefined}
				{...(animationsEnabled
					? {
							initial: { opacity: 0, x: -6 },
							animate: { opacity: 1, x: 0 },
							transition: {
								duration: 0.18,
								ease: [0.23, 1, 0.32, 1],
								delay: index * 0.03,
							},
							whileTap: { scale: 0.97 },
						}
					: {})}
				className={`group relative flex w-full items-center rounded-lg transition-all duration-150 ${
					isCollapsed ? 'justify-center px-0 py-2' : 'py-[5px] pr-2'
				}`}
				style={{
					paddingLeft: isCollapsed ? 0 : depth > 0 ? `${depth * 14 + 10}px` : '10px',
				}}>
				{/* Active background */}
				{isActive && (
					<motion.div
						layoutId='sidebar-active'
						className='absolute inset-0 rounded-lg'
						style={{
							background: `linear-gradient(105deg, rgba(var(--accent-rgb), 0.18) 0%, rgba(var(--accent-rgb), 0.08) 100%)`,
							boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.18)`,
						}}
						{...(animationsEnabled
							? { transition: { type: 'spring', stiffness: 380, damping: 32 } }
							: {})}
					/>
				)}

				{/* Hover background */}
				{!isActive && (
					<div className='absolute inset-0 rounded-lg bg-[var(--surface-hover)] opacity-0 transition-opacity duration-100 group-hover:opacity-100' />
				)}

				{/* Icon */}
				<div
					className='relative z-10 flex items-center justify-center transition-all duration-150'
					style={isActive ? { color: accentColor } : undefined}>
					{depth > 0 ? (
						<div className='mr-1.5 flex items-center opacity-30'>
							<ChevronRight className='h-2.5 w-2.5' />
						</div>
					) : null}
					<div
						className={`flex items-center justify-center rounded-md transition-all duration-150 ${
							isActive
								? ''
								: 'text-[var(--text-tertiary)] group-hover:text-[var(--text-secondary)]'
						}`}>
						{getIcon(mailbox.role, isActive)}
					</div>
				</div>

				{/* Label */}
				{!isCollapsed && (
					<span
						className={`relative z-10 ml-2.5 truncate text-[13px] font-medium transition-colors duration-150 ${
							isActive
								? 'text-[var(--text-primary)]'
								: 'text-[var(--text-secondary)] group-hover:text-[var(--text-primary)]'
						}`}>
						{label}
					</span>
				)}
			</motion.button>
		)
	}
)

// ─── Section label ────────────────────────────────────────────────────────────

function SectionLabel({
	label,
	action,
	onAction,
	collapsed,
}: {
	label: string
	action?: string
	onAction?: () => void
	collapsed?: boolean
}) {
	if (collapsed) return null
	return (
		<div className='group/sh mt-3 mb-0.5 flex items-center px-[10px]'>
			<span className='flex-1 text-[10px] font-semibold tracking-[0.08em] text-[var(--text-tertiary)] uppercase opacity-60'>
				{label}
			</span>
			{onAction && (
				<button
					type='button'
					onClick={onAction}
					title={action}
					className='flex h-4 w-4 items-center justify-center rounded text-[var(--text-tertiary)] opacity-0 transition-all duration-150 group-hover/sh:opacity-60 hover:bg-[var(--surface-hover)] hover:opacity-100'>
					<Plus className='h-3 w-3' />
				</button>
			)}
		</div>
	)
}

// ─── Main Sidebar ─────────────────────────────────────────────────────────────

export const Sidebar = ({
	activeAccount,
	activeMailbox,
	onMailboxSelect,
	onCompose,
}: SidebarProps) => {
	const { t } = useTypedTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const [width, setWidth] = useState(DEFAULT_WIDTH)
	const [isResizing, setIsResizing] = useState(false)
	const [newFolderOpen, setNewFolderOpen] = useState(false)
	const [creatingFolder, setCreatingFolder] = useState(false)
	const qc = useQueryClient()

	const isCollapsed = width < 110

	const { data: mailboxes, isLoading } = useQuery({
		queryKey: ['mailboxes', activeAccount?.id],
		queryFn: () => invoke<Mailbox[]>('fetch_mailboxes', { accountId: activeAccount?.id }),
		enabled: !!activeAccount,
	})

	const { data: tags } = useQuery({
		queryKey: ['account-tags', activeAccount?.id],
		queryFn: () => invoke<string[]>('get_account_tags', { accountId: activeAccount?.id }),
		enabled: !!activeAccount,
	})

	const allMailboxes = mailboxes ? [...mailboxes] : []
	if (!allMailboxes.some((m) => m.role === 'drafts')) {
		allMailboxes.push({
			name: 'Drafts',
			display_name: t('inbox:sidebar.mailboxes.drafts'),
			role: 'drafts',
			uid_validity: undefined,
			highest_modseq: undefined,
			last_synced_uid: undefined,
			separator: '/',
		})
	}
	if (!allMailboxes.some((m) => m.role === 'flagged')) {
		allMailboxes.push({
			name: 'Virtual_Starred',
			display_name: t('inbox:sidebar.mailboxes.starred'),
			role: 'flagged',
			uid_validity: undefined,
			highest_modseq: undefined,
			last_synced_uid: undefined,
			separator: '/',
		})
	}

	// Resize
	const startResizing = (e: React.MouseEvent) => {
		e.preventDefault()
		setIsResizing(true)
	}
	useEffect(() => {
		const onMove = (e: MouseEvent) => {
			if (!isResizing) return
			setWidth(Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, e.clientX)))
		}
		const onUp = () => setIsResizing(false)
		if (isResizing) {
			window.addEventListener('mousemove', onMove)
			window.addEventListener('mouseup', onUp)
		}
		return () => {
			window.removeEventListener('mousemove', onMove)
			window.removeEventListener('mouseup', onUp)
		}
	}, [isResizing])

	const { systemMailboxes, customMailboxes } = useMemo(() => {
		const system: Mailbox[] = []
		const custom: Mailbox[] = []
		const systemRoots = allMailboxes.filter((m) => ROLE_ORDER.includes(m.role) && !m.hidden)

		for (const mb of allMailboxes) {
			if (mb.hidden) continue
			if (ROLE_ORDER.includes(mb.role)) {
				system.push(mb)
			} else if (mb.role !== 'tag') {
				const isSystemSub = systemRoots.some((root) =>
					mb.name.startsWith(root.name + mb.separator)
				)
				if (isSystemSub) system.push(mb)
				else custom.push(mb)
			}
		}

		system.sort((a, b) => {
			const getRootRole = (m: Mailbox) => {
				if (ROLE_ORDER.includes(m.role)) return m.role
				const root = systemRoots.find((r) => m.name.startsWith(r.name + m.separator))
				return root ? root.role : m.role
			}
			const rootA = getRootRole(a),
				rootB = getRootRole(b)
			if (rootA !== rootB) return ROLE_ORDER.indexOf(rootA) - ROLE_ORDER.indexOf(rootB)
			return a.name.localeCompare(b.name)
		})
		custom.sort((a, b) => a.name.localeCompare(b.name))
		return { systemMailboxes: system, customMailboxes: custom }
	}, [allMailboxes])

	const handleCreateFolder = useCallback(
		async (name: string) => {
			if (!activeAccount) return
			setCreatingFolder(true)
			try {
				await invoke('create_folder', { accountId: activeAccount.id, name })
				qc.invalidateQueries({ queryKey: ['mailboxes', activeAccount.id] })
				toast.success(t('inbox:sidebar.folders.createSuccess', { name }))
			} catch (e) {
				toast.error(String(e))
				return Promise.reject(e)
			} finally {
				setCreatingFolder(false)
			}
		},
		[activeAccount, qc, t]
	)

	return (
		<>
			<div style={{ width }} className='relative flex h-full flex-col'>
				{/* Compose button */}
				<div className='px-2 pt-2 pb-1.5'>
					<motion.button
						type='button'
						onClick={onCompose}
						{...(animationsEnabled
							? { whileHover: { scale: 1.02 }, whileTap: { scale: 0.96 } }
							: {})}
						className={`group relative flex items-center overflow-hidden rounded-xl text-white transition-shadow duration-200 hover:shadow-lg ${
							isCollapsed ? 'mx-auto h-9 w-9 justify-center' : 'h-9 w-full px-3.5'
						}`}
						style={{
							background: `linear-gradient(115deg, var(--accent-dark) 0%, var(--accent-color) 60%, color-mix(in srgb, var(--accent-color) 80%, white) 100%)`,
							boxShadow: `0 4px 14px -3px rgba(var(--accent-rgb), 0.35)`,
						}}>
						<div className='absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/15 to-transparent transition-transform duration-500 ease-out group-hover:translate-x-full' />
						<Pencil className='relative h-3.5 w-3.5 shrink-0' strokeWidth={2.5} />
						{!isCollapsed && (
							<span className='relative ml-2.5 truncate text-[13px] font-semibold tracking-tight'>
								{t('inbox:sidebar.newMessage')}
							</span>
						)}
					</motion.button>
				</div>

				{/* Nav list */}
				<div className='hover-scrollbar flex-1 overflow-x-hidden overflow-y-auto px-2'>
					{isLoading ? (
						<div className='flex flex-col gap-1 pt-1'>
							{[0.9, 0.7, 0.8, 0.6, 0.75].map((op, i) => (
								<div
									key={i}
									className='relative h-7 overflow-hidden rounded-lg bg-[var(--surface-active)]'
									style={{ opacity: op }}>
									<div className='skeleton-shimmer' />
								</div>
							))}
						</div>
					) : (
						<div>
							{/* System mailboxes */}
							<div className='space-y-px pt-1'>
								{systemMailboxes.map((mailbox, i) => {
									const displayParts = mailbox.display_name.split(
										mailbox.separator
									)
									const shortName =
										displayParts[displayParts.length - 1] ||
										mailbox.display_name
									return (
										<FolderContextMenu
											key={mailbox.name}
											mailbox={mailbox}
											accountId={activeAccount?.id ?? ''}
											activeMailbox={activeMailbox}
											onMailboxSelect={onMailboxSelect}
											shortName={shortName}
											parentPrefix=''>
											<MailboxItem
												mailbox={mailbox}
												isActive={activeMailbox === mailbox.name}
												isCollapsed={isCollapsed}
												accentColor={accentColor}
												animationsEnabled={animationsEnabled}
												onSelect={onMailboxSelect}
												depth={0}
												shortName={shortName}
												index={i}
											/>
										</FolderContextMenu>
									)
								})}
							</div>

							{/* Custom folders */}
							{customMailboxes.length > 0 && (
								<>
									<SectionLabel
										label={t('inbox:sidebar.folders.sectionTitle')}
										action={t('inbox:sidebar.folders.new')}
										onAction={() => setNewFolderOpen(true)}
										collapsed={isCollapsed}
									/>
									<div className='space-y-px'>
										{customMailboxes.map((mailbox, i) => {
											const parts = mailbox.name.split(mailbox.separator)
											const depth = Math.max(0, parts.length - 1)
											const displayParts = mailbox.display_name.split(
												mailbox.separator
											)
											const shortName =
												depth === 0
													? mailbox.display_name
													: displayParts[displayParts.length - 1] ||
														mailbox.display_name
											const parentPrefix =
												depth > 0
													? mailbox.name.substring(
															0,
															mailbox.name.length - shortName.length
														)
													: ''
											return (
												<FolderContextMenu
													key={mailbox.name}
													mailbox={mailbox}
													accountId={activeAccount?.id ?? ''}
													activeMailbox={activeMailbox}
													onMailboxSelect={onMailboxSelect}
													shortName={shortName}
													parentPrefix={parentPrefix}>
													<MailboxItem
														mailbox={mailbox}
														isActive={activeMailbox === mailbox.name}
														isCollapsed={isCollapsed}
														accentColor={accentColor}
														animationsEnabled={animationsEnabled}
														onSelect={onMailboxSelect}
														depth={depth}
														shortName={shortName}
														index={systemMailboxes.length + i}
													/>
												</FolderContextMenu>
											)
										})}
									</div>
								</>
							)}

							{/* Tags */}
							{tags && tags.length > 0 && (
								<>
									<SectionLabel label='Tags' collapsed={isCollapsed} />
									<div className='space-y-px'>
										{tags.map((tag, i) => (
											<MailboxItem
												key={tag}
												mailbox={
													{
														name: `Virtual_Tag:${tag}`,
														display_name: tag,
														role: 'tag',
													} as any
												}
												isActive={activeMailbox === `Virtual_Tag:${tag}`}
												isCollapsed={isCollapsed}
												accentColor={accentColor}
												animationsEnabled={animationsEnabled}
												onSelect={onMailboxSelect}
												index={
													systemMailboxes.length +
													customMailboxes.length +
													i
												}
											/>
										))}
									</div>
								</>
							)}

							{/* Contacts */}
							<div className='mt-3 mb-0.5'>
								<button
									type='button'
									onClick={() => window.dispatchEvent(new CustomEvent('app:open-contacts'))}
									title={isCollapsed ? t('contacts:sidebar.contacts') : undefined}
									className='group relative flex w-full items-center rounded-lg transition-all duration-150 hover:bg-[var(--surface-hover)]'
									style={{ padding: isCollapsed ? '8px 0' : '5px 8px 5px 10px', justifyContent: isCollapsed ? 'center' : undefined }}>
									<div className='flex items-center justify-center text-[var(--text-tertiary)] transition-colors duration-150 group-hover:text-[var(--text-secondary)]'>
										<Users className='h-[15px] w-[15px] shrink-0' />
									</div>
									{!isCollapsed && (
										<span className='relative z-10 ml-2.5 truncate text-[13px] font-medium text-[var(--text-secondary)] transition-colors duration-150 group-hover:text-[var(--text-primary)]'>
											{t('contacts:sidebar.contacts')}
										</span>
									)}
								</button>
							</div>

							{/* Bottom spacer */}
							<div className='h-4' />
						</div>
					)}
				</div>

				{/* Resize handle */}
				<div
					className='group absolute top-0 right-[-3px] bottom-0 z-50 w-2 cursor-col-resize'
					onMouseDown={startResizing}>
					<div
						className={`absolute left-1/2 h-full w-[1.5px] -translate-x-1/2 transition-all duration-300 ${
							isResizing ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
						}`}
						style={{
							backgroundColor: accentColor,
							boxShadow: `0 0 8px ${accentColor}60`,
						}}
					/>
				</div>
			</div>

			<FolderNameDialog
				open={newFolderOpen}
				onOpenChange={setNewFolderOpen}
				title={t('inbox:sidebar.folders.new')}
				placeholder={t('inbox:sidebar.folders.newPlaceholder')}
				confirmLabel={
					creatingFolder
						? t('inbox:sidebar.folders.creating')
						: t('inbox:sidebar.folders.create')
				}
				onConfirm={handleCreateFolder}
				separator={allMailboxes[0]?.separator || '/'}
			/>
		</>
	)
}
