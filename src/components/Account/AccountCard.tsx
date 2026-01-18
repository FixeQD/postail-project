import { useEffect, useState } from 'react'
import { motion } from 'framer-motion'
import { MoreVertical, RefreshCw, CheckCircle2, Mail, Trash2, AlertTriangle, Loader2 } from 'lucide-react'
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
		if (lower.includes('gmail')) return <Mail className="h-5 w-5 text-red-500" />
		if (lower.includes('outlook')) return <Mail className="h-5 w-5 text-blue-500" />
		return <Mail className="h-5 w-5 text-slate-400" />
	}

	const getStatusBadge = () => {
		if (status === 'Syncing') {
			return (
				<span className="inline-flex items-center rounded-full bg-blue-500/10 px-2 py-0.5 text-xs font-medium text-blue-500 ring-1 ring-inset ring-blue-500/20">
					<Loader2 className="mr-1 h-3 w-3 animate-spin" />
					Syncing...
				</span>
			)
		}
		
		if (typeof status === 'object' && 'Error' in status) {
			return (
				<span className="inline-flex items-center rounded-full bg-red-500/10 px-2 py-0.5 text-xs font-medium text-red-500 ring-1 ring-inset ring-red-500/20">
					<AlertTriangle className="mr-1 h-3 w-3" />
					Error
				</span>
			)
		}

		return (
			<span className="inline-flex items-center rounded-full bg-emerald-500/10 px-2 py-0.5 text-xs font-medium text-emerald-500 ring-1 ring-inset ring-emerald-500/20">
				<CheckCircle2 className="mr-1 h-3 w-3" />
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
			transition={{ duration: 0.2 }}
		>
			<Card className="group relative overflow-hidden border-white/5 bg-white/5 backdrop-blur-md transition-all hover:bg-white/10 hover:shadow-lg hover:shadow-black/20 hover:border-white/10">
				<div className="absolute inset-0 bg-gradient-to-br from-white/5 to-transparent opacity-0 transition-opacity group-hover:opacity-100" />
				
				<div className="relative flex items-center justify-between p-5">
					<div className="flex items-center gap-4">
						<div className="flex h-12 w-12 items-center justify-center rounded-xl bg-slate-900/50 ring-1 ring-white/10 transition-transform group-hover:scale-110">
							{getProviderIcon(account.provider_type)}
						</div>
						
						<div className="flex flex-col">
							<h3 className="font-semibold text-slate-100">{account.name}</h3>
							<p className="text-sm text-slate-400 font-medium">{account.email}</p>
							<div className="flex items-center gap-2 mt-1">
								{getStatusBadge()}
								<span className="text-xs text-slate-600">
									{account.auth_type}
								</span>
							</div>
						</div>
					</div>

					<div className="flex items-center gap-2">
						<Button
							variant="ghost"
							size="icon"
							className={cn("h-8 w-8 text-slate-400 hover:text-slate-100 hover:bg-white/5", status === "Syncing" && "animate-spin")}
							onClick={() => onSync(account.id)}
							disabled={status === "Syncing"}
						>
							<RefreshCw className="h-4 w-4" />
						</Button>

						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<Button variant="ghost" size="icon" className="h-8 w-8 text-slate-400 hover:text-slate-100 hover:bg-white/5">
									<MoreVertical className="h-4 w-4" />
								</Button>
							</DropdownMenuTrigger>
							<DropdownMenuContent align="end" className="w-48 bg-slate-900 border-slate-800 text-slate-200">
								<DropdownMenuLabel>Actions</DropdownMenuLabel>
								<DropdownMenuSeparator className="bg-slate-800" />
								<DropdownMenuItem className="focus:bg-slate-800 focus:text-slate-100 cursor-pointer">
									Edit settings
								</DropdownMenuItem>
								<DropdownMenuItem 
									className="text-red-400 focus:text-red-300 focus:bg-red-500/10 cursor-pointer"
									onClick={() => onRemove(account.id)}
								>
									<Trash2 className="mr-2 h-4 w-4" />
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
