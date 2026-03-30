import { useRef, useState, useEffect, useMemo, memo } from 'react'
import { motion } from 'framer-motion'
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
} from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { useQuery } from '@tanstack/react-query'
import type { Mailbox } from '@/types/mail'
import type { AccountMeta } from '@/types/accounts'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore } from '@/stores/themeStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'

interface SidebarProps {
	activeAccount: AccountMeta | null
	activeMailbox: string
	onMailboxSelect: (mailbox: string) => void
	onCompose: () => void
}

const MIN_WIDTH = 80
const MAX_WIDTH = 320
const DEFAULT_WIDTH = 260

interface MailboxItemProps {
	mailbox: Mailbox
	isActive: boolean
	isCollapsed: boolean
	accentColor: string
	animationsEnabled: boolean
	onSelect: (name: string) => void
}

const listVariants = {
	hidden: {},
	visible: {
		transition: {
			staggerChildren: 0.05,
			delayChildren: 0.02,
		},
	},
}

const MailboxItem = memo(
	({
		mailbox,
		isActive,
		isCollapsed,
		accentColor,
		animationsEnabled,
		onSelect,
	}: MailboxItemProps) => {
		const getIcon = () => {
			const cls = 'h-[18px] w-[18px]'
			switch (mailbox.role) {
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
				default:
					return <File className={cls} />
			}
		}

		return (
			<motion.button
				type='button'
				onClick={() => onSelect(mailbox.name)}
				title={isCollapsed ? mailbox.display_name : undefined}
				{...(animationsEnabled
					? {
							whileTap: { scale: 0.97 },
							initial: { opacity: 0, x: -8 },
							animate: { opacity: 1, x: 0 },
							transition: { duration: 0.22, ease: [0.16, 1, 0.3, 1] },
						}
					: {})}
				className={`relative flex w-full items-center rounded-xl px-3.5 py-2.5 text-sm font-medium transition-all duration-200 ${
					isActive
						? ''
						: 'text-muted-foreground hover:text-foreground hover:bg-[var(--surface-hover)]'
				} ${isCollapsed ? 'justify-center px-0' : ''}`}
				style={isActive ? { color: accentColor } : undefined}>
				{/* Active background */}
				{isActive && (
					<div
						className='absolute inset-0 rounded-xl transition-all duration-200 ease-out'
						style={{
							backgroundColor: `rgba(var(--accent-rgb), 0.15)`,
							boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2), 0 2px 8px -2px rgba(var(--accent-rgb), 0.1)`,
						}}
					/>
				)}

				{/* Active left indicator */}
				{isActive && (
					<motion.div
						{...(animationsEnabled
							? {
									initial: { scaleY: 0, opacity: 0 },
									animate: { scaleY: 1, opacity: 1 },
									exit: { scaleY: 0, opacity: 0 },
									transition: {
										type: 'spring',
										stiffness: 200,
										damping: 25,
									},
								}
							: {})}
						className='absolute top-1/2 left-0 h-5 w-[3px] origin-center -translate-y-1/2 rounded-r-full'
						style={{ backgroundColor: accentColor }}
					/>
				)}

				<div
					className='relative shrink-0 transition-colors duration-200'
					style={isActive ? { color: accentColor } : undefined}>
					{getIcon()}
				</div>
				{!isCollapsed && (
					<div className='relative ml-3.5 flex flex-1 items-center justify-between truncate'>
						<span>{mailbox.display_name}</span>
					</div>
				)}
			</motion.button>
		)
	}
)

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
	const sidebarRef = useRef<HTMLDivElement>(null)

	const isCollapsed = width < 120

	const { data: mailboxes, isLoading } = useQuery({
		queryKey: ['mailboxes', activeAccount?.id],
		queryFn: () => invoke<Mailbox[]>('fetch_mailboxes', { accountId: activeAccount?.id }),
		enabled: !!activeAccount,
	})

	// Add virtual Drafts mailbox if not present
	const allMailboxes = mailboxes ? [...mailboxes] : []
	if (!allMailboxes.some((m) => m.role === 'drafts')) {
		allMailboxes.push({
			name: 'Drafts',
			display_name: t('inbox:sidebar.mailboxes.drafts'),
			role: 'drafts',
			uid_validity: undefined,
			highest_modseq: undefined,
			last_synced_uid: undefined,
		})
	}

	// Add virtual Starred mailbox if not present natively
	if (!allMailboxes.some((m) => m.role === 'flagged')) {
		allMailboxes.push({
			name: 'Virtual_Starred',
			display_name: t('inbox:sidebar.mailboxes.starred'),
			role: 'flagged',
			uid_validity: undefined,
			highest_modseq: undefined,
			last_synced_uid: undefined,
		})
	}

	const startResizing = (e: React.MouseEvent) => {
		e.preventDefault()
		setIsResizing(true)
	}

	useEffect(() => {
		const handleMouseMove = (e: MouseEvent) => {
			if (!isResizing) return
			let newWidth = e.clientX
			if (newWidth < MIN_WIDTH) newWidth = MIN_WIDTH
			if (newWidth > MAX_WIDTH) newWidth = MAX_WIDTH
			setWidth(newWidth)
		}

		const handleMouseUp = () => {
			setIsResizing(false)
		}

		if (isResizing) {
			window.addEventListener('mousemove', handleMouseMove)
			window.addEventListener('mouseup', handleMouseUp)
		}

		return () => {
			window.removeEventListener('mousemove', handleMouseMove)
			window.removeEventListener('mouseup', handleMouseUp)
		}
	}, [isResizing])

	const sortedMailboxes = useMemo(() => {
		const sorted = [...allMailboxes].sort((a, b) => {
			const roleOrder = ['inbox', 'flagged', 'sent', 'drafts', 'archive', 'junk', 'trash']
			const scoreA = roleOrder.indexOf(a.role)
			const scoreB = roleOrder.indexOf(b.role)

			if (scoreA !== -1 && scoreB !== -1) return scoreA - scoreB
			if (scoreA !== -1) return -1
			if (scoreB !== -1) return 1

			return a.name.localeCompare(b.name)
		})
		return sorted
	}, [allMailboxes])

	return (
		<>
			<motion.div
				ref={sidebarRef}
				style={{ width }}
				className='relative flex h-full flex-col p-3'>
				{/* Right edge gradient line */}
				<div className='pointer-events-none absolute top-0 right-0 bottom-0 w-px bg-gradient-to-b from-transparent via-black/[0.06] to-transparent dark:via-white/[0.06]' />

				{/* New Message Button */}
				<div className='mb-1 flex flex-col gap-3'>
					<motion.button
						type='button'
						onClick={onCompose}
						{...(animationsEnabled
							? { whileHover: { scale: 1.02 }, whileTap: { scale: 0.96 } }
							: {})}
						className={`text-accent-contrast group relative flex items-center overflow-hidden rounded-xl px-4 py-3 text-sm font-semibold shadow-lg transition-shadow hover:shadow-xl ${isCollapsed ? 'mx-auto aspect-square h-11 w-11 justify-center px-0' : 'w-full'}`}
						style={{
							background: `linear-gradient(to right, var(--accent-dark), var(--accent-color))`,
							boxShadow: `0 8px 20px -4px rgba(var(--accent-rgb), 0.15)`,
						}}>
						{/* Shimmer effect on hover */}
						<div className='absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/10 to-transparent transition-transform duration-700 group-hover:translate-x-full' />
						<Pencil className='relative h-4 w-4 shrink-0' />
						{!isCollapsed && (
							<span className='relative ml-3 truncate'>
								{t('inbox:sidebar.newMessage')}
							</span>
						)}
					</motion.button>

					{/* Separator */}
					<div className='relative mx-3 h-px'>
						<div className='absolute inset-0 bg-gradient-to-r from-transparent via-black/[0.08] to-transparent dark:via-white/[0.08]' />
					</div>
				</div>

				{/* Mailboxes */}
				<div className='hover-scrollbar flex-1 space-y-0.5 overflow-x-hidden overflow-y-auto pt-1'>
					{isLoading ? (
						<div className='flex flex-col gap-2 p-2'>
							{[1, 2, 3, 4, 5].map((i) => (
								<div
									key={i}
									className='relative h-10 overflow-hidden rounded-xl bg-[var(--surface-active)]'>
									<div className='skeleton-shimmer' />
								</div>
							))}
						</div>
					) : (
						<motion.div
							{...(animationsEnabled
								? { variants: listVariants, initial: 'hidden', animate: 'visible' }
								: {})}>
							{sortedMailboxes.map((mailbox) => (
								<MailboxItem
									key={mailbox.name}
									mailbox={mailbox}
									isActive={activeMailbox === mailbox.name}
									isCollapsed={isCollapsed}
									accentColor={accentColor}
									animationsEnabled={animationsEnabled}
									onSelect={onMailboxSelect}
								/>
							))}
						</motion.div>
					)}
				</div>

				{/* Resizer Handle */}
				<div
					className='group absolute top-0 right-[-3px] bottom-0 z-50 w-2 cursor-col-resize'
					onMouseDown={startResizing}>
					<div
						className={`absolute left-1/2 h-full w-[2px] -translate-x-1/2 transition-all duration-300 ${
							isResizing ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
						}`}
						style={{
							backgroundColor: accentColor,
							boxShadow: `0 0 12px ${accentColor}`,
						}}
					/>
				</div>
			</motion.div>
		</>
	)
}
