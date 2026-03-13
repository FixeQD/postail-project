import { motion } from 'framer-motion'
import { useWelcomeTranslation } from '@/hooks/useTypedTranslation'
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
		<div className='ambient-glow noise-overlay relative flex h-full flex-col items-center justify-center overflow-hidden p-8 text-center'>
			{/* Background accent orbs */}
			<div
				className='pointer-events-none absolute top-1/4 left-1/4 h-96 w-96 rounded-full blur-[120px]'
				style={{ backgroundColor: `rgba(var(--accent-rgb), 0.05)` }}
			/>
			<div
				className='pointer-events-none absolute right-1/4 bottom-1/4 h-80 w-80 rounded-full blur-[100px]'
				style={{ backgroundColor: `rgba(var(--accent-rgb), 0.03)` }}
			/>

			{/* Main Glass Card */}
			<motion.div
				initial={{ opacity: 0, y: 24, scale: 0.96, filter: 'blur(8px)' }}
				animate={{ opacity: 1, y: 0, scale: 1, filter: 'blur(0px)' }}
				transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
				className='relative z-10 flex flex-col items-center rounded-[2.5rem] border border-[var(--border-subtle)] p-12 shadow-2xl backdrop-blur-[64px]'
				style={{
					backgroundColor: `var(--surface-glass)`,
					backgroundImage: `linear-gradient(135deg, rgba(255,255,255,0.03) 0%, transparent 100%)`,
					boxShadow:
						'0 32px 80px -16px rgba(0,0,0,0.5), inset 0 1px 1px rgba(255,255,255,0.12), inset 0 -1px 1px rgba(0,0,0,0.1)',
				}}>
				{/* Logo */}
				<motion.div
					initial={{ opacity: 0, scale: 0.8 }}
					animate={{ opacity: 1, scale: 1 }}
					transition={{ duration: 0.6, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
					className='relative z-10 mb-8'>
					<div className='animate-subtle-float'>
						<div className='relative flex h-24 w-24 items-center justify-center rounded-3xl bg-[var(--surface-panel)] shadow-xl ring-1 ring-[var(--border-subtle)]'>
							<img src={icon} alt='Postail' className='h-16 w-16' />
							{/* Glow behind logo */}
							<div
								className='animate-glow-breathe absolute -inset-3 -z-10 rounded-3xl blur-2xl'
								style={{ backgroundColor: `rgba(var(--accent-rgb), 0.15)` }}
							/>
						</div>
					</div>
				</motion.div>

				{/* Title */}
				<motion.h1
					initial={{ opacity: 0, y: 16 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.5, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
					className='gradient-text relative z-10 mb-3 text-5xl font-bold tracking-tight'>
					{t('welcome:title')}
				</motion.h1>

				{/* Subtitle */}
				<motion.p
					initial={{ opacity: 0, y: 12 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.5, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
					className='text-muted-foreground relative z-10 text-lg'>
					{t('welcome:subtitle')}
				</motion.p>

				{/* Description */}
				<motion.div
					initial={{ opacity: 0, y: 12 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.5, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
					className='relative z-10 mt-4 mb-10 max-w-md'>
					<p className='text-muted-foreground/70 leading-relaxed'>
						{t('welcome:description')}
					</p>
				</motion.div>

				{/* CTA Button */}
				<motion.button
					type='button'
					onClick={onGetStarted}
					initial={{ opacity: 0, y: 16, scale: 0.95 }}
					animate={{ opacity: 1, y: 0, scale: 1 }}
					transition={{ duration: 0.5, delay: 0.5, ease: [0.16, 1, 0.3, 1] }}
					whileHover={{ scale: 1.05, y: -2 }}
					whileTap={{ scale: 0.95 }}
					className='text-accent-contrast relative z-10 rounded-2xl px-12 py-4 text-sm font-semibold shadow-xl transition-all focus:ring-2 focus:ring-offset-2 focus:ring-offset-[var(--app-bg)] focus:outline-none'
					style={{
						background: `linear-gradient(135deg, var(--accent-dark), var(--accent-color))`,
						boxShadow: `0 12px 32px -8px rgba(var(--accent-rgb), 0.4)`,
					}}
					title={t('welcome:getStarted')}>
					{t('welcome:getStarted')}
				</motion.button>

				{/* Existing data link */}
				<motion.button
					type='button'
					onClick={onExistingData}
					initial={{ opacity: 0 }}
					animate={{ opacity: 1 }}
					transition={{ duration: 0.4, delay: 0.65, ease: 'easeOut' }}
					className='text-tertiary hover:text-muted-foreground relative z-10 mt-6 text-sm transition-colors focus:outline-none'>
					{t('welcome:existingData')}
				</motion.button>
			</motion.div>

			{/* Decorative bottom gradient line */}
			<motion.div
				initial={{ scaleX: 0, opacity: 0 }}
				animate={{ scaleX: 1, opacity: 1 }}
				transition={{ duration: 0.8, delay: 0.7, ease: [0.16, 1, 0.3, 1] }}
				className='absolute bottom-0 left-0 h-px w-full origin-center'
				style={{
					background: `linear-gradient(to right, transparent, rgba(var(--accent-rgb), 0.2), transparent)`,
				}}
			/>
		</div>
	)
}
