import { useRef, useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { Inbox, Send, Trash2, Archive, File, Pencil } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { useQuery } from '@tanstack/react-query'
import type { Mailbox } from '../../types/mail'
import type { AccountMeta } from '../../types/accounts'
import { useTypedTranslation } from '../../hooks/useTypedTranslation'

interface SidebarProps {
	activeAccount: AccountMeta | null
	activeMailbox: string
	onMailboxSelect: (mailbox: string) => void
	onCompose: () => void
}

const MIN_WIDTH = 80
const MAX_WIDTH = 320
const DEFAULT_WIDTH = 260

export const Sidebar = ({
	activeAccount,
	activeMailbox,
	onMailboxSelect,
	onCompose,
}: SidebarProps) => {
	const { t } = useTypedTranslation()
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

	const getIconForMailbox = (mailbox: Mailbox) => {
		switch (mailbox.role) {
			case 'inbox':
				return <Inbox className='h-5 w-5' />
			case 'sent':
				return <Send className='h-5 w-5' />
			case 'trash':
				return <Trash2 className='h-5 w-5' />
			case 'archive':
				return <Archive className='h-5 w-5' />
			case 'drafts':
				return <File className='h-5 w-5' />
			case 'junk':
				return <File className='h-5 w-5' />
			default:
				return <File className='h-5 w-5' />
		}
	}

	const getMailboxLabel = (mailbox: Mailbox) => {
		// Translation keys match roles usually
		switch (mailbox.role) {
			case 'inbox':
				return t('inbox:sidebar.mailboxes.inbox')
			case 'sent':
				return t('inbox:sidebar.mailboxes.sent')
			case 'drafts':
				return t('inbox:sidebar.mailboxes.drafts')
			case 'trash':
				return t('inbox:sidebar.mailboxes.trash')
			case 'archive':
				return t('inbox:sidebar.mailboxes.archive')
			default:
				return mailbox.display_name
		}
	}

	return (
		<>
			<motion.div
				ref={sidebarRef}
				style={{ width }}
				className='relative flex h-full flex-col bg-slate-950 p-4'>
				<div className='absolute top-0 right-0 bottom-0 w-px bg-gradient-to-b from-transparent via-slate-800 to-transparent' />
				{/* New Message Button */}
				<div className='mb-2 flex flex-col gap-4'>
					<button
						type='button'
						onClick={onCompose}
						className={`group flex items-center rounded-xl bg-slate-800 px-4 py-3 text-sm font-semibold text-slate-200 shadow-sm transition-all hover:bg-slate-700 hover:shadow-md active:scale-[0.98] ${isCollapsed ? 'mx-auto aspect-square h-12 w-12 justify-center px-0' : 'w-full'}`}>
						<Pencil className='h-4 w-4 shrink-0 transition-transform group-hover:scale-110' />
						{!isCollapsed && (
							<span className='ml-3 truncate'>{t('inbox:sidebar.newMessage')}</span>
						)}
					</button>

					{/* Separator */}
					<div className='relative mx-2 h-px'>
						<div className='absolute inset-0 bg-gradient-to-r from-transparent via-orange-500 to-transparent opacity-30 blur-[1px]' />
						<div className='absolute inset-0 bg-gradient-to-r from-transparent via-slate-700 to-transparent' />
					</div>
				</div>

				{/* Mailboxes */}
				<div className='flex-1 space-y-1 overflow-x-hidden overflow-y-auto'>
					{isLoading ? (
						<div className='flex flex-col gap-2 p-2'>
							{[1, 2, 3].map((i) => (
								<div
									key={i}
									className='h-10 animate-pulse rounded-full bg-slate-900'
								/>
							))}
						</div>
					) : (
						<>
							{allMailboxes
								.sort((a, b) => {
									if (a.name.toLowerCase() === 'inbox') return -1
									if (a.role === 'drafts') return 1
									if (b.role === 'drafts') return -1
									return a.name.localeCompare(b.name)
								})
								.map((mailbox) => {
									const isActive = activeMailbox === mailbox.name
									return (
										<button
											type='button'
											key={mailbox.name}
											onClick={() => onMailboxSelect(mailbox.name)}
											title={isCollapsed ? mailbox.display_name : undefined}
											className={`relative flex w-full items-center rounded-l-3xl rounded-r-3xl px-4 py-3 text-sm font-medium transition-all ${
												isActive
													? 'bg-orange-500/10 text-orange-500'
													: 'text-slate-400 hover:bg-slate-900 hover:text-slate-200'
											} ${isCollapsed ? 'justify-center px-0' : ''}`}>
											<div
												className={`shrink-0 ${isActive ? 'text-orange-500' : 'text-slate-400'}`}>
												{getIconForMailbox(mailbox)}
											</div>
											{!isCollapsed && (
												<div className='ml-4 flex flex-1 items-center justify-between truncate'>
													<span>{getMailboxLabel(mailbox)}</span>
												</div>
											)}
										</button>
									)
								})}
						</>
					)}
				</div>

				{/* Resizer Handle */}
				<div
					className={`absolute top-0 right-0 h-full w-1 cursor-col-resize transition-colors hover:bg-orange-500/50 active:bg-orange-500 ${isResizing ? 'bg-orange-500' : 'bg-transparent'}`}
					onMouseDown={startResizing}
				/>
			</motion.div>
		</>
	)
}
