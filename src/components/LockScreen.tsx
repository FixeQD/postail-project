import { useState } from 'react'
import { motion } from 'framer-motion'
import { Lock } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { toast } from './ui/custom/Toaster'
import type { LockScreenProps } from '@/types/components/shared'

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
				backgroundColor: 'var(--app-bg, #020617)',
			}}>
			<div
				className='pointer-events-none absolute inset-0'
				style={{
					backdropFilter: 'blur(50px)',
					WebkitBackdropFilter: 'blur(50px)',
					backgroundColor: `color-mix(in srgb, var(--app-bg, #020617) 80%, transparent)`,
				}}
			/>
			<div
				className='pointer-events-none absolute inset-0'
				style={{
					backdropFilter: 'blur(50px)',
					WebkitBackdropFilter: 'blur(50px)',
					backgroundColor: `color-mix(in srgb, var(--app-bg, #020617) 80%, transparent)`,
				}}
			/>
			<div className='relative z-10 flex flex-col items-center gap-6 p-8'>
				<div className='flex h-20 w-20 items-center justify-center rounded-2xl bg-[var(--surface-active)] ring-1 ring-[var(--border-subtle)]'>
					<Lock className='text-muted-foreground h-10 w-10' />
				</div>

				<div className='text-center'>
					<h2 className='text-foreground text-2xl font-bold'>App Locked</h2>
					<p className='text-muted-foreground mt-2 text-sm'>
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
						className='text-foreground placeholder:text-muted-foreground w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] px-4 py-3 text-center ring-0 transition-all outline-none focus:border-[var(--accent-color)] focus:bg-[var(--surface-hover)]'
					/>
				</div>

				<button
					type='button'
					onClick={handleUnlock}
					disabled={!password || isUnlocking}
					className='text-accent-contrast w-72 rounded-xl px-4 py-3 font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50'
					style={{
						background: `linear-gradient(to right, var(--accent-dark), var(--accent-color))`,
					}}>
					{isUnlocking ? 'Unlocking...' : 'Unlock'}
				</button>
			</div>
		</motion.div>
	)
}
