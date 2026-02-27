import { useEffect, useState, useMemo, memo, useRef } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import {
	MoreVertical,
	RefreshCw,
	CheckCircle2,
	Mail,
	Trash2,
	AlertTriangle,
	Loader2,
	Settings2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { cn } from '@/lib/utils'
import { invoke } from '@tauri-apps/api/core'
import { EditAccountDialog } from './EditAccountDialog'
import type { AccountCardProps } from '@/types/components/shared'

type SyncStatus = 'Idle' | 'Syncing' | { Error: string }

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

export const AccountCard = memo(({ account, onRemove, onSync }: AccountCardProps) => {
	const animationsEnabled = useAnimationsEnabled()
	const [status, setStatus] = useState<SyncStatus>('Idle')
	const [isEditDialogOpen, setIsEditDialogOpen] = useState(false)
	const [menuOpen, setMenuOpen] = useState(false)
	const menuRef = useRef<HTMLDivElement>(null)

	// Close menu on outside click
	useEffect(() => {
		if (!menuOpen) return
		const handler = (e: MouseEvent) => {
			if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
				setMenuOpen(false)
			}
		}
		document.addEventListener('mousedown', handler)
		return () => document.removeEventListener('mousedown', handler)
	}, [menuOpen])

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

	const statusBadge = useMemo(() => {
		const motionFade = animationsEnabled
			? { initial: { opacity: 0, scale: 0.9 }, animate: { opacity: 1, scale: 1 } }
			: {}

		if (status === 'Syncing') {
			return (
				<motion.span
					{...motionFade}
					className='inline-flex items-center rounded-full bg-blue-500/10 px-2.5 py-0.5 text-[11px] font-semibold text-blue-400 ring-1 ring-blue-500/20 ring-inset'>
					<Loader2 className='mr-1 h-3 w-3 animate-spin' />
					Syncing...
				</motion.span>
			)
		}

		if (typeof status === 'object' && 'Error' in status) {
			return (
				<motion.span
					{...motionFade}
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
	}, [status, animationsEnabled])

	return (
		<motion.div
			{...(animationsEnabled
				? {
						layout: true,
						initial: { opacity: 0, y: 20 },
						animate: { opacity: 1, y: 0 },
						exit: { opacity: 0, scale: 0.95 },
						transition: { duration: 0.3, ease: [0.16, 1, 0.3, 1] },
					}
				: {})}>
			<Card className='group relative overflow-visible border-white/[0.06] bg-white/[0.03] backdrop-blur-md transition-all duration-300 hover:border-white/[0.1] hover:bg-white/[0.06] hover:shadow-xl hover:shadow-black/30'>
				{/* Hover gradient overlay */}
				<div className='pointer-events-none absolute inset-0 rounded-[inherit] bg-gradient-to-br from-white/[0.04] via-transparent to-transparent opacity-0 transition-opacity duration-500 group-hover:opacity-100' />

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
								{animationsEnabled ? (
									<AnimatePresence mode='wait'>
										<motion.div
											key={typeof status === 'string' ? status : 'error'}>
											{statusBadge}
										</motion.div>
									</AnimatePresence>
								) : (
									<div>{statusBadge}</div>
								)}
								<span className='text-[11px] text-slate-600'>
									{account.auth_type}
								</span>
							</div>
						</div>
					</div>

					<div className='flex items-center gap-1'>
						<motion.div
							{...(animationsEnabled
								? { whileHover: { scale: 1.1 }, whileTap: { scale: 0.85 } }
								: {})}>
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

						{/* Inline actions menu */}
						<div ref={menuRef} className='relative'>
							<Button
								variant='ghost'
								size='icon'
								className='h-8 w-8 text-slate-500 hover:bg-white/[0.06] hover:text-slate-200'
								onClick={() => setMenuOpen((v) => !v)}>
								<MoreVertical className='h-4 w-4' />
							</Button>

							<AnimatePresence>
								{menuOpen && (
									<motion.div
										initial={{ opacity: 0, scaleY: 0.8, y: -4 }}
										animate={{ opacity: 1, scaleY: 1, y: 0 }}
										exit={{ opacity: 0, scaleY: 0.8, y: -4 }}
										transition={{ duration: 0.15, ease: [0.16, 1, 0.3, 1] }}
										style={{ transformOrigin: 'top right' }}
										className='absolute top-full right-0 z-50 mt-1 w-48 overflow-hidden rounded-lg border border-white/[0.06] bg-slate-900/95 py-1 shadow-xl shadow-black/40 backdrop-blur-xl'>
										<p className='px-3 py-1.5 text-xs font-medium text-slate-500'>
											Actions
										</p>
										<div className='mx-1 my-1 h-px bg-white/[0.06]' />
										<button
											className='flex w-full items-center gap-2.5 px-3 py-2 text-sm text-slate-200 transition-colors hover:bg-white/[0.06] hover:text-slate-100'
											onClick={() => {
												setIsEditDialogOpen(true)
												setMenuOpen(false)
											}}>
											<Settings2 className='h-4 w-4 text-slate-400' />
											Edit settings
										</button>
										<button
											className='flex w-full items-center gap-2.5 px-3 py-2 text-sm text-red-400 transition-colors hover:bg-red-500/10 hover:text-red-300'
											onClick={() => {
												onRemove(account.id)
												setMenuOpen(false)
											}}>
											<Trash2 className='h-4 w-4' />
											Remove account
										</button>
									</motion.div>
								)}
							</AnimatePresence>
						</div>
					</div>
				</div>
			</Card>

			<EditAccountDialog
				account={account}
				open={isEditDialogOpen}
				onOpenChange={setIsEditDialogOpen}
			/>
		</motion.div>
	)
})
