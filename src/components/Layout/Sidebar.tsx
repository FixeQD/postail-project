import { useState } from 'react'
import { motion } from 'framer-motion'
import { Inbox, Send, Trash2, Archive, File, Menu, Settings, LogOut, RefreshCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { useQuery } from '@tanstack/react-query'
import type { Mailbox } from '../../types/mail'
import type { AccountMeta } from '../../types/accounts'

interface SidebarProps {
	activeAccount: AccountMeta | null
	activeMailbox: string
	onMailboxSelect: (mailbox: string) => void
    onOpenSettings: () => void
    onLogout: () => void
    collapsed?: boolean
    onToggleCollapse?: () => void
}

export const Sidebar = ({ 
    activeAccount, 
    activeMailbox, 
    onMailboxSelect,
    onOpenSettings,
    onLogout,
    collapsed = false,
    onToggleCollapse
}: SidebarProps) => {
	const { t } = useTranslation()

	const { data: mailboxes, isLoading } = useQuery({
		queryKey: ['mailboxes', activeAccount?.id],
		queryFn: () => invoke<Mailbox[]>('fetch_mailboxes', { accountId: activeAccount?.id }),
		enabled: !!activeAccount,
	})

    // Standard folders mapping to icons
    const getIconForMailbox = (name: string) => {
        const lower = name.toLowerCase()
        if (lower === 'inbox') return <Inbox className="h-4 w-4" />
        if (lower.includes('sent')) return <Send className="h-4 w-4" />
        if (lower.includes('trash') || lower.includes('bin')) return <Trash2 className="h-4 w-4" />
        if (lower.includes('archive')) return <Archive className="h-4 w-4" />
        if (lower.includes('drafts')) return <File className="h-4 w-4" />
        if (lower.includes('junk') || lower.includes('spam')) return <Trash2 className="h-4 w-4" /> // Use different icon?
        return <File className="h-4 w-4 opacity-50" />
    }

	return (
		<motion.div 
            animate={{ width: collapsed ? 64 : 240 }}
            className='flex h-full flex-col border-r border-slate-800 bg-slate-900/50 backdrop-blur-sm'
        >
            {/* Header / Account Selector */}
            <div className='flex items-center gap-3 border-b border-slate-800 p-4'>
                <div className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-orange-500 font-bold text-white shadow-lg shadow-orange-500/20`}>
                    {activeAccount?.name.charAt(0).toUpperCase()}
                </div>
                {!collapsed && (
                    <div className='flex-1 overflow-hidden'>
                        <h2 className='truncate text-sm font-semibold text-slate-100'>
                            {activeAccount?.name}
                        </h2>
                        <p className='truncate text-xs text-slate-400'>
                            {activeAccount?.email}
                        </p>
                    </div>
                )}
            </div>

            {/* Mailboxes */}
			<div className='flex-1 overflow-y-auto px-2 py-4'>
                {isLoading ? (
                     <div className="flex flex-col gap-2">
                        {[1, 2, 3, 4].map(i => (
                            <div key={i} className="h-8 animate-pulse rounded-md bg-slate-800/50" />
                        ))}
                     </div>
                ) : (
                    <div className='space-y-1'>
                        {/* Always show Inbox first if present */}
                        {mailboxes?.sort((a, b) => {
                             if (a.name.toLowerCase() === 'inbox') return -1
                             if (b.name.toLowerCase() === 'inbox') return 1
                             return a.name.localeCompare(b.name)
                        }).map((mailbox) => (
                            <button
                                key={mailbox.name}
                                onClick={() => onMailboxSelect(mailbox.name)}
                                className={`group flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${
                                    activeMailbox === mailbox.name
                                        ? 'bg-orange-500/10 text-orange-400'
                                        : 'text-slate-400 hover:bg-slate-800 hover:text-slate-100'
                                }`}
                                title={collapsed ? mailbox.name : undefined}
                            >
                                {getIconForMailbox(mailbox.name)}
                                {!collapsed && <span>{mailbox.name}</span>}
                            </button>
                        ))}
                    </div>
                )}
			</div>

            {/* Footer Actions */}
            <div className='border-t border-slate-800 p-2'>
                <button
                    onClick={onOpenSettings}
                    className='flex w-full items-center gap-3 rounded-lg p-2 text-sm font-medium text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-100'
                >
                    <Settings className='h-4 w-4' />
                    {!collapsed && <span>{t('Accounts')}</span>}
                </button>
                <button
                    onClick={onLogout}
                    className='flex w-full items-center gap-3 rounded-lg p-2 text-sm font-medium text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-100'
                >
                    <LogOut className='h-4 w-4' />
                    {!collapsed && <span>Logout</span>}
                </button>
            </div>
		</motion.div>
	)
}
