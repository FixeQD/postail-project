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
				className='pointer-events-none absolute top-1/4 left-1/3 h-64 w-64 rounded-full blur-[100px]'
				style={{ backgroundColor: `rgba(var(--accent-rgb), 0.04)` }}
			/>
			<div className='pointer-events-none absolute right-1/3 bottom-1/3 h-48 w-48 rounded-full bg-indigo-500/[0.04] blur-[80px]' />

			{/* Logo */}
			<motion.div
				initial={{ opacity: 0, scale: 0.8 }}
				animate={{ opacity: 1, scale: 1 }}
				transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
				className='relative z-10 mb-8'>
				<div className='animate-subtle-float'>
					<div className='relative flex h-24 w-24 items-center justify-center rounded-2xl bg-[var(--surface-active)] shadow-xl ring-1 ring-[var(--border-subtle)]'>
						<img src={icon} alt='Postail' className='h-20 w-20' />
						{/* Glow behind logo */}
						<div
							className='animate-glow-breathe absolute -inset-3 -z-10 rounded-3xl blur-xl'
							style={{ backgroundColor: `rgba(var(--accent-rgb), 0.1)` }}
						/>
					</div>
				</div>
			</motion.div>

			{/* Title */}
			<motion.h1
				initial={{ opacity: 0, y: 16 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.5, delay: 0.15, ease: [0.16, 1, 0.3, 1] }}
				className='gradient-text relative z-10 mb-3 text-5xl font-bold tracking-tight'>
				{t('welcome:title')}
			</motion.h1>

			{/* Subtitle */}
			<motion.p
				initial={{ opacity: 0, y: 12 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.5, delay: 0.25, ease: [0.16, 1, 0.3, 1] }}
				className='text-muted-foreground relative z-10 text-lg'>
				{t('welcome:subtitle')}
			</motion.p>

			{/* Description */}
			<motion.div
				initial={{ opacity: 0, y: 12 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.5, delay: 0.35, ease: [0.16, 1, 0.3, 1] }}
				className='relative z-10 mt-4 mb-12 max-w-md'>
				<p className='text-muted-foreground/70 leading-relaxed'>
					{t('welcome:description')}
				</p>
			</motion.div>

			{/* CTA Button */}
			<motion.button
				type='button'
				onClick={onGetStarted}
				initial={{ opacity: 0, y: 16 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.5, delay: 0.5, ease: [0.16, 1, 0.3, 1] }}
				whileHover={{ scale: 1.03, y: -1 }}
				whileTap={{ scale: 0.97 }}
				className='text-accent-contrast press-down relative z-10 rounded-xl px-10 py-3.5 text-sm font-semibold shadow-lg transition-shadow hover:shadow-xl focus:ring-2 focus:ring-offset-2 focus:ring-offset-[var(--app-bg)] focus:outline-none'
				style={{
					background: `linear-gradient(to right, var(--accent-dark), var(--accent-color))`,
					boxShadow: `0 8px 24px -4px rgba(var(--accent-rgb), 0.2)`,
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
				className='text-tertiary hover:text-muted-foreground relative z-10 mt-4 text-sm transition-colors focus:outline-none'>
				{t('welcome:existingData')}
			</motion.button>

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
