import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion } from 'framer-motion'
import { useTranslation } from 'react-i18next'
import { Copy, Download, Check, ShieldAlert, ArrowRight } from 'lucide-react'
import DecryptedText from '../../DecryptedText'
import { useThemeStore } from '@/stores/themeStore'
import { RecoveryVerifyDialog } from './RecoveryVerifyDialog'
import type { RecoveryStepProps } from '@/types/components/welcome'

export const RecoveryStep = ({ onNext, variant = 'page', encryptionMethod }: RecoveryStepProps) => {
	const { t } = useTranslation('welcome')
	const accentColor = useThemeStore((s) => s.accentColor)
	const [phrase, setPhrase] = useState<string>('')
	const [words, setWords] = useState<string[]>([])
	const [copied, setCopied] = useState(false)
	const [saved, setSaved] = useState(false)
	const [visibleIndices, setVisibleIndices] = useState<Set<number>>(new Set())
	const [loading, setLoading] = useState(true)
	const [verifyOpen, setVerifyOpen] = useState(false)

	const handleAnimationComplete = (index: number) => {
		setVisibleIndices((prev) => {
			const next = new Set(prev)
			next.add(index)
			return next
		})
	}

	useEffect(() => {
		const generate = async () => {
			try {
				const p = await invoke<string>('generate_recovery_phrase')
				setPhrase(p)
				setWords(p.split(' '))
			} catch (err) {
				console.error('Failed to generate phrase:', err)
			} finally {
				setLoading(false)
			}
		}
		generate()
	}, [])

	const handleCopy = async () => {
		await navigator.clipboard.writeText(phrase)
		setCopied(true)
		setTimeout(() => setCopied(false), 2000)
	}

	const handleSave = async () => {
		const blob = new Blob([phrase], { type: 'text/plain' })
		const url = URL.createObjectURL(blob)
		const a = document.createElement('a')
		a.href = url
		a.download = 'postail-recovery-phrase.txt'
		document.body.appendChild(a)
		a.click()
		document.body.removeChild(a)
		URL.revokeObjectURL(url)
		setSaved(true)
		setTimeout(() => setSaved(false), 2000)
	}

	const handleNext = () => {
		setVerifyOpen(true)
	}

	const cardContent = (
		<>
			{variant === 'embedded' && (
				<h2 className='text-lg font-semibold text-[var(--text-primary)]'>
					{t('recovery.title')}
				</h2>
			)}
			<div className='rounded-xl border border-amber-500/20 bg-amber-500/10 p-4'>
				<div className='flex gap-3'>
					<ShieldAlert className='h-5 w-5 shrink-0 text-amber-500' />
					<div>
						<h3 className='font-semibold text-amber-500'>
							{t('recovery.warning.title')}
						</h3>
						<p className='mt-1 text-sm text-amber-700 dark:text-amber-200/80'>
							{encryptionMethod === 'tpm'
								? t('recovery.warning.descriptionTPM')
								: t('recovery.warning.descriptionArgon2')}
						</p>
					</div>
				</div>
			</div>

			<div className='grid grid-cols-3 gap-3 sm:grid-cols-4'>
				{loading
					? Array.from({ length: 12 }).map((_, i) => (
							<div
								key={i}
								className='flex h-12 w-full animate-pulse rounded-lg bg-[var(--surface-active)]'
							/>
						))
					: words.map((word, i) => (
							<motion.div
								key={i}
								initial={{ opacity: 0, y: 10 }}
								animate={{ opacity: 1, y: 0 }}
								transition={{ delay: i * 0.05 + 0.2 }}
								onAnimationComplete={() => handleAnimationComplete(i)}
								className='group relative flex items-center gap-3 overflow-hidden rounded-lg bg-[var(--surface-panel)] px-3 py-2.5 ring-1 ring-[var(--border-subtle)] transition-all hover:bg-[var(--surface-hover)]'>
								<span className='w-6 shrink-0 text-xs font-medium text-[var(--text-secondary)]'>
									{i + 1}.
								</span>
								<span className='font-mono text-sm font-medium text-[var(--text-primary)]'>
									{visibleIndices.has(i) ? (
										<DecryptedText
											text={word}
											animateOn='view'
											revealDirection='start'
											speed={50}
											maxIterations={20}
											characters='abcdefghijklmnopqrstuvwxyz'
											forceScramble={verifyOpen}
										/>
									) : (
										<span className='invisible'>{word}</span>
									)}
								</span>
							</motion.div>
						))}
			</div>

			<div className='flex gap-3'>
				<button
					onClick={handleCopy}
					className='flex flex-1 items-center justify-center gap-2 rounded-xl bg-[var(--surface-panel)] px-4 py-3 text-sm font-medium text-[var(--text-primary)] ring-1 ring-[var(--border-subtle)] transition-all hover:bg-[var(--surface-hover)] active:scale-[0.98]'>
					{copied ? (
						<>
							<Check className='h-4 w-4 text-green-400' />
							<span className='text-green-400'>{t('recovery.action.copied')}</span>
						</>
					) : (
						<>
							<Copy className='h-4 w-4' />
							{t('recovery.action.copy')}
						</>
					)}
				</button>
				<button
					onClick={handleSave}
					className='flex flex-1 items-center justify-center gap-2 rounded-xl bg-[var(--surface-panel)] px-4 py-3 text-sm font-medium text-[var(--text-primary)] ring-1 ring-[var(--border-subtle)] transition-all hover:bg-[var(--surface-hover)] active:scale-[0.98]'>
					{saved ? (
						<>
							<Check className='h-4 w-4 text-green-400' />
							<span className='text-green-400'>{t('recovery.action.saved')}</span>
						</>
					) : (
						<>
							<Download className='h-4 w-4' />
							{t('recovery.action.save')}
						</>
					)}
				</button>
			</div>

			<div className='pt-4'>
				<motion.button
					onClick={handleNext}
					whileHover={{ scale: 1.01 }}
					whileTap={{ scale: 0.99 }}
					className='text-accent-contrast flex w-full items-center justify-center gap-2.5 rounded-xl px-6 py-4 text-base font-bold shadow-lg transition-all hover:shadow-xl'
					style={{
						background: `linear-gradient(to right, var(--accent-dark), var(--accent-color))`,
						boxShadow: `0 8px 24px -4px rgba(var(--accent-rgb), 0.2)`,
					}}>
					<span>{t('recovery.action.continue')}</span>
					<ArrowRight className='h-5 w-5' />
				</motion.button>
			</div>

			<RecoveryVerifyDialog
				open={verifyOpen}
				onClose={() => setVerifyOpen(false)}
				onVerified={async () => {
					await onNext(phrase)
					setVerifyOpen(false)
				}}
			/>
		</>
	)

	if (variant === 'embedded') {
		return <div className='mx-auto w-full max-w-2xl space-y-6 pt-2'>{cardContent}</div>
	}

	return (
		<div className='noise-overlay relative flex h-full flex-col'>
			<motion.div
				initial={{ opacity: 0, y: -20, filter: 'blur(8px)' }}
				animate={{ opacity: 1, y: 0, filter: 'blur(0px)' }}
				transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
				className='relative border-b border-black/5 bg-white/10 px-4 py-6 shadow-sm backdrop-blur-[32px] dark:border-white/5 dark:bg-black/20'>
				<div className='pointer-events-none absolute inset-x-0 bottom-0 h-px bg-gradient-to-r from-transparent via-[var(--accent-color)] to-transparent opacity-20' />
				<div className='container mx-auto'>
					<div className='flex items-center gap-3'>
						<div
							className='flex h-10 w-10 items-center justify-center rounded-xl ring-1'
							style={{
								backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
								boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
							}}>
							<ShieldAlert className='h-5 w-5' style={{ color: accentColor }} />
						</div>
						<div>
							<h1 className='text-3xl font-bold tracking-tight text-[var(--text-primary)]'>
								{t('recovery.title')}
							</h1>
							<p className='mt-1 text-sm text-[var(--text-secondary)]'>
								{t('recovery.subtitle')}
							</p>
						</div>
					</div>
				</div>
			</motion.div>

			<div className='container mx-auto flex flex-1 flex-col items-center justify-center px-4 py-8'>
				<motion.div
					initial={{ opacity: 0, scale: 0.95 }}
					animate={{ opacity: 1, scale: 1 }}
					transition={{ duration: 0.5, delay: 0.1 }}
					className='w-full max-w-2xl space-y-8'>
					{cardContent}
				</motion.div>
			</div>
		</div>
	)
}
