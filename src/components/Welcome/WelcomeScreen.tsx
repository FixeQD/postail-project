import { useWelcomeTranslation } from '@/hooks/useTypedTranslation'
import { motion } from 'framer-motion'
import icon from '@/assets/icon.png'

export const WelcomeScreen = ({
	onGetStarted,
	onExistingData,
}: {
	onGetStarted: () => void
	onExistingData: () => void
}) => {
	const { t } = useWelcomeTranslation()

	return (
		<div className="relative flex min-h-full flex-col items-center justify-center p-6 bg-[var(--app-bg)] overflow-hidden">
			{/* Subtle background glow - simple CSS gradient, no blur */}
			<div 
				className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] pointer-events-none opacity-30"
				style={{
					background: 'radial-gradient(ellipse at center, rgba(var(--accent-rgb), 0.12) 0%, transparent 60%)',
				}}
			/>

			{/* Soft grid background */}
			<div 
				className="absolute inset-0 pointer-events-none opacity-[0.15]"
				style={{
					backgroundImage: 'radial-gradient(circle, var(--border-subtle) 1px, transparent 1px)',
					backgroundSize: '32px 32px',
					maskImage: 'radial-gradient(ellipse 60% 60% at 50% 50%, black 20%, transparent 100%)',
					WebkitMaskImage: 'radial-gradient(ellipse 60% 60% at 50% 50%, black 20%, transparent 100%)',
				}}
			/>

			<motion.div
				initial={{ opacity: 0, y: 16 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
				className="relative z-10 flex w-full max-w-[420px] flex-col items-center text-center"
			>
				{/* Large, elegant app icon */}
				<div className="mb-8 flex h-28 w-28 items-center justify-center rounded-[2rem] bg-[var(--surface-panel)] border border-[var(--border-subtle)] shadow-2xl relative">
					<div className="absolute inset-0 rounded-[2rem] border border-white/10 dark:border-white/5" />
					<img src={icon} alt="Postail" className="h-16 w-16 drop-shadow-lg" />
				</div>

				{/* Clean typography */}
				<h1 className="mb-3 text-[2.5rem] font-bold tracking-tight text-[var(--text-primary)] leading-none">
					{t('welcome:title')}
				</h1>

				<p className="mb-10 text-[1.05rem] leading-relaxed text-[var(--text-secondary)]">
					{t('welcome:subtitle')}
				</p>

				{/* Simple, prominent actions */}
				<div className="flex w-full flex-col gap-3 px-2">
					<button
						type="button"
						onClick={onGetStarted}
						className="group relative flex w-full items-center justify-center rounded-xl px-6 py-4 text-[0.95rem] font-semibold text-white transition-all hover:-translate-y-0.5 active:translate-y-0 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[var(--accent-color)] focus:ring-offset-[var(--app-bg)]"
						style={{
							background: 'linear-gradient(180deg, var(--accent-light) 0%, var(--accent-color) 100%)',
							boxShadow: '0 8px 24px -8px rgba(var(--accent-rgb), 0.6), inset 0 1px 0 rgba(255, 255, 255, 0.25)'
						}}
					>
						{t('welcome:getStarted')}
					</button>

					<button
						type="button"
						onClick={onExistingData}
						className="rounded-xl px-6 py-3.5 text-[0.9rem] font-medium text-[var(--text-tertiary)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--border-subtle)]"
					>
						{t('welcome:existingData')}
					</button>
				</div>
			</motion.div>
		</div>
	)
}
