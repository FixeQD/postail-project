import { useEffect, useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import {
	MoreVertical,
	RefreshCw,
	CheckCircle2,
	Mail,
	Trash2,
	AlertTriangle,
	Loader2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { cn } from '@/lib/utils'
import type { AccountMeta } from '@/types/accounts'
import { invoke } from '@tauri-apps/api/core'

interface AccountCardProps {
	account: AccountMeta
	onRemove: (id: string) => void
	onSync: (id: string) => void
}

type SyncStatus = 'Idle' | 'Syncing' | { Error: string }

export function AccountCard({ account, onRemove, onSync }: AccountCardProps) {
	const [status, setStatus] = useState<SyncStatus>('Idle')

	useEffect(() => {
		const fetchStatus = async () => {
			try {
				const res = await invoke<SyncStatus>('get_sync_status', { accountId: account.id })
				setStatus(res)
			} catch (e) {
				console.error('Failed to get sync status', e)
			}
		}

		fetchStatus()
		const interval = setInterval(fetchStatus, 2000)
		return () => clearInterval(interval)
	}, [account.id])

	const getProviderIcon = (type: string) => {
		const lower = type.toLowerCase()
		if (lower.includes('gmail')) return <Mail className='h-5 w-5 text-red-400' />
		if (lower.includes('outlook')) return <Mail className='h-5 w-5 text-blue-400' />
		return <Mail className='h-5 w-5 text-slate-400' />
	}

	const getProviderGlow = (type: string) => {
		const lower = type.toLowerCase()
		if (lower.includes('gmail')) return 'group-hover:ring-red-500/20'
		if (lower.includes('outlook')) return 'group-hover:ring-blue-500/20'
		return 'group-hover:ring-white/[0.12]'
	}

	const getStatusBadge = () => {
		if (status === 'Syncing') {
			return (
				<motion.span
					initial={{ opacity: 0, scale: 0.9 }}
					animate={{ opacity: 1, scale: 1 }}
					className='inline-flex items-center rounded-full bg-blue-500/10 px-2.5 py-0.5 text-[11px] font-semibold text-blue-400 ring-1 ring-blue-500/20 ring-inset'>
					<Loader2 className='mr-1 h-3 w-3 animate-spin' />
					Syncing...
				</motion.span>
			)
		}

		if (typeof status === 'object' && 'Error' in status) {
			return (
				<motion.span
					initial={{ opacity: 0, scale: 0.9 }}
					animate={{ opacity: 1, scale: 1 }}
					className='inline-flex items-center rounded-full bg-red-500/10 px-2.5 py-0.5 text-[11px] font-semibold text-red-400 ring-1 ring-red-500/20 ring-inset'>
					<AlertTriangle className='mr-1 h-3 w-3' />
					Error
				</motion.span>
			)
		}

		return (
			<span className='inline-flex items-center rounded-full bg-emerald-500/10 px-2.5 py-0.5 text-[11px] font-semibold text-emerald-400 ring-1 ring-emerald-500/20 ring-inset'>
				<CheckCircle2 className='mr-1 h-3 w-3' />
				Synced
			</span>
		)
	}

	return (
		<motion.div
			layout
			initial={{ opacity: 0, y: 20 }}
			animate={{ opacity: 1, y: 0 }}
			exit={{ opacity: 0, scale: 0.95 }}
			transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}>
			<Card className='group relative overflow-hidden border-white/[0.06] bg-white/[0.03] backdrop-blur-md transition-all duration-300 hover:border-white/[0.1] hover:bg-white/[0.06] hover:shadow-xl hover:shadow-black/30'>
				{/* Hover gradient overlay */}
				<div className='pointer-events-none absolute inset-0 bg-gradient-to-br from-white/[0.04] via-transparent to-transparent opacity-0 transition-opacity duration-500 group-hover:opacity-100' />

				{/* Subtle top highlight */}
				<div className='pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-white/[0.08] to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100' />

				<div className='relative flex items-center justify-between p-5'>
					<div className='flex items-center gap-4'>
						<div
							className={cn(
								'flex h-12 w-12 items-center justify-center rounded-xl bg-slate-900/60 ring-1 ring-white/[0.08] transition-all duration-300 group-hover:scale-105',
								getProviderGlow(account.provider_type)
							)}>
							{getProviderIcon(account.provider_type)}
						</div>

						<div className='flex flex-col'>
							<h3 className='text-[15px] font-semibold text-slate-100 transition-colors group-hover:text-white'>
								{account.name}
							</h3>
							<p className='text-sm text-slate-400'>{account.email}</p>
							<div className='mt-1.5 flex items-center gap-2'>
								<AnimatePresence mode='wait'>
									<motion.div key={typeof status === 'string' ? status : 'error'}>
										{getStatusBadge()}
									</motion.div>
								</AnimatePresence>
								<span className='text-[11px] text-slate-600'>
									{account.auth_type}
								</span>
							</div>
						</div>
					</div>

					<div className='flex items-center gap-1'>
						<motion.div whileHover={{ scale: 1.1 }} whileTap={{ scale: 0.85 }}>
							<Button
								variant='ghost'
								size='icon'
								className={cn(
									'h-8 w-8 text-slate-500 hover:bg-white/[0.06] hover:text-slate-200',
									status === 'Syncing' && 'animate-spin'
								)}
								onClick={() => onSync(account.id)}
								disabled={status === 'Syncing'}>
								<RefreshCw className='h-4 w-4' />
							</Button>
						</motion.div>

						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<Button
									variant='ghost'
									size='icon'
									className='h-8 w-8 text-slate-500 hover:bg-white/[0.06] hover:text-slate-200'>
									<MoreVertical className='h-4 w-4' />
								</Button>
							</DropdownMenuTrigger>
							<DropdownMenuContent
								align='end'
								className='w-48 border-white/[0.06] bg-slate-900/95 text-slate-200 backdrop-blur-xl'>
								<DropdownMenuLabel className='text-slate-400'>
									Actions
								</DropdownMenuLabel>
								<DropdownMenuSeparator className='bg-white/[0.06]' />
								<DropdownMenuItem className='cursor-pointer focus:bg-white/[0.06] focus:text-slate-100'>
									Edit settings
								</DropdownMenuItem>
								<DropdownMenuItem
									className='cursor-pointer text-red-400 focus:bg-red-500/10 focus:text-red-300'
									onClick={() => onRemove(account.id)}>
									<Trash2 className='mr-2 h-4 w-4' />
									Remove account
								</DropdownMenuItem>
							</DropdownMenuContent>
						</DropdownMenu>
					</div>
				</div>
			</Card>
		</motion.div>
	)
}
