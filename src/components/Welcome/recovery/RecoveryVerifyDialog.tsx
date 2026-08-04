import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogDescription,
} from '@/components/ui/dialog'
import { ShieldCheck, Loader2, AlertTriangle, ArrowRight } from 'lucide-react'
import { motion } from 'framer-motion'
import { useThemeStore } from '@/stores/themeStore'
import type { RecoveryVerifyDialogProps } from '@/types/components/shared'

export function RecoveryVerifyDialog({ open, onClose, onVerified }: RecoveryVerifyDialogProps) {
	const { t } = useTranslation(['welcome', 'common'])
	const accentColor = useThemeStore((s) => s.accentColor)
	const [indices, setIndices] = useState<number[]>([])
	const [inputs, setInputs] = useState<string[]>(['', '', ''])
	const [loading, setLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)

	useEffect(() => {
		if (open) {
			// Generate 3 unique random indices between 0 and 11
			const newIndices = new Set<number>()
			while (newIndices.size < 3) {
				newIndices.add(Math.floor(Math.random() * 12))
			}
			setIndices(Array.from(newIndices).sort((a, b) => a - b))
			setInputs(['', '', ''])
			setError(null)
			setLoading(false)
		}
	}, [open])

	const handleVerify = async () => {
		setLoading(true)
		setError(null)

		try {
			const isValid = await invoke<boolean>('verify_recovery_words', {
				indices,
				words: inputs,
			})

			if (isValid) {
				await onVerified()
			} else {
				setError(t('recovery.verify.error'))
			}
		} catch (err) {
			console.error('Verification failed:', err)
			setError(t('recovery.verify.error'))
		} finally {
			setLoading(false)
		}
	}

	const handleInputChange = (index: number, value: string) => {
		const newInputs = [...inputs]
		newInputs[index] = value
		setInputs(newInputs)
		if (error) setError(null)
	}

	const isComplete = inputs.every((input) => input.trim().length > 0)

	return (
		<Dialog open={open} onOpenChange={(o) => !o && onClose()}>
			<DialogContent className='border-[var(--border-subtle)] bg-[var(--surface-glass)] text-[var(--text-primary)] backdrop-blur-xl sm:max-w-md'>
				<DialogHeader>
					<DialogTitle className='flex items-center gap-2'>
						<ShieldCheck className='h-5 w-5 text-status-success' />
						{t('recovery.verify.title')}
					</DialogTitle>
					<DialogDescription className='text-[var(--text-secondary)]'>
						{t('recovery.verify.description')}
					</DialogDescription>
				</DialogHeader>

				<div className='py-4'>
					<div className='grid gap-4'>
						{indices.map((idx, i) => (
							<div key={idx} className='space-y-2'>
								<label className='text-sm font-medium text-[var(--text-primary)]'>
									{t('recovery.verify.wordLabel', { number: idx + 1 })}
								</label>
								<input
									type='text'
									value={inputs[i]}
									onChange={(e) => handleInputChange(i, e.target.value)}
									className='w-full rounded-lg bg-[var(--surface-hover)] px-3 py-2 text-[var(--text-primary)] ring-1 ring-[var(--border-subtle)] placeholder:text-[var(--text-tertiary)] focus:ring-2 focus:ring-[var(--accent-color)] focus:outline-none'
									placeholder={t('recovery.verify.placeholder', {
										number: idx + 1,
									})}
									style={
										{
											'--accent-color': accentColor,
										} as React.CSSProperties
									}
									autoComplete='off'
									disabled={loading}
								/>
							</div>
						))}
					</div>

					{error && (
						<motion.div
							initial={{ opacity: 0, y: -5 }}
							animate={{ opacity: 1, y: 0 }}
							className='mt-4 flex items-center gap-2 rounded-lg bg-destructive/15 p-3 text-sm text-destructive ring-1 ring-destructive/30'>
							<AlertTriangle className='h-4 w-4 shrink-0' />
							{error}
						</motion.div>
					)}

					<div className='mt-6 flex justify-end gap-3'>
						<button
							onClick={onClose}
							className='rounded-lg px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
							disabled={loading}>
							{t('common:actions.cancel') || 'Cancel'}
						</button>
						<button
							onClick={handleVerify}
							disabled={!isComplete || loading}
							className='flex items-center gap-2 rounded-lg bg-[var(--accent-color)] px-4 py-2 text-sm font-medium text-white shadow-lg transition-all hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-50 disabled:shadow-none'
							style={
								{
									'--accent-color': accentColor,
								} as React.CSSProperties
							}>
							{loading ? (
								<Loader2 className='h-4 w-4 animate-spin' />
							) : (
								<ArrowRight className='h-4 w-4' />
							)}
							{t('recovery.verify.verify')}
						</button>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	)
}
