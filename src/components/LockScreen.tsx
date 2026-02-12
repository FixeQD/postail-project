import { useState } from 'react'
import { motion } from 'framer-motion'
import { Lock } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { toast } from './ui/custom/Toaster'

interface LockScreenProps {
	isLocked: boolean
	onUnlock: () => void
	useEncryptionPassword: boolean
}

export const LockScreen = ({ isLocked, onUnlock, useEncryptionPassword }: LockScreenProps) => {
	const [password, setPassword] = useState('')
	const [isUnlocking, setIsUnlocking] = useState(false)

	if (!isLocked) return null

	const handleUnlock = async () => {
		if (!password) return

		setIsUnlocking(true)
		try {
			await invoke('unlock_app', { password })
			onUnlock()
			setPassword('')
		} catch (error) {
			toast.error('Invalid password', {
				description: String(error),
			})
		} finally {
			setIsUnlocking(false)
		}
	}

	const handleKeyDown = (e: React.KeyboardEvent) => {
		if (e.key === 'Enter') {
			handleUnlock()
		}
	}

	return (
		<motion.div
			initial={{ opacity: 0 }}
			animate={{ opacity: 1 }}
			exit={{ opacity: 0 }}
			className='fixed inset-0 z-[100] flex items-center justify-center'
			style={{
				backdropFilter: 'blur(100px) saturate(0%) brightness(0.02)',
				WebkitBackdropFilter: 'blur(100px) saturate(0%) brightness(0.02)',
				backgroundColor: 'rgba(2, 6, 23, 1)',
			}}>
			<div
				className='pointer-events-none absolute inset-0'
				style={{
					backdropFilter: 'blur(50px)',
					WebkitBackdropFilter: 'blur(50px)',
					backgroundColor: 'rgba(2, 6, 23, 0.8)',
				}}
			/>
			<div
				className='pointer-events-none absolute inset-0'
				style={{
					backdropFilter: 'blur(50px)',
					WebkitBackdropFilter: 'blur(50px)',
					backgroundColor: 'rgba(2, 6, 23, 0.8)',
				}}
			/>
			<div className='relative z-10 flex flex-col items-center gap-6 p-8'>
				<div className='flex h-20 w-20 items-center justify-center rounded-2xl bg-slate-800/50 ring-1 ring-white/10'>
					<Lock className='h-10 w-10 text-slate-400' />
				</div>

				<div className='text-center'>
					<h2 className='text-2xl font-bold text-white'>App Locked</h2>
					<p className='mt-2 text-sm text-slate-400'>
						{useEncryptionPassword
							? 'Enter your database encryption password'
							: 'Enter your PIN to unlock'}
					</p>
				</div>

				<div className='w-72'>
					<input
						type='password'
						value={password}
						onChange={(e) => setPassword(e.target.value)}
						onKeyDown={handleKeyDown}
						placeholder={useEncryptionPassword ? 'Password' : 'PIN'}
						className='w-full rounded-xl border border-slate-700 bg-slate-900/50 px-4 py-3 text-center text-white placeholder-slate-500 ring-0 transition-all outline-none focus:border-slate-500 focus:bg-slate-800/50'
					/>
				</div>

				<button
					type='button'
					onClick={handleUnlock}
					disabled={!password || isUnlocking}
					className='w-72 rounded-xl bg-slate-700 px-4 py-3 font-medium text-white transition-colors hover:bg-slate-600 disabled:cursor-not-allowed disabled:opacity-50'>
					{isUnlocking ? 'Unlocking...' : 'Unlock'}
				</button>
			</div>
		</motion.div>
	)
}
