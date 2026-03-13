import { motion } from 'framer-motion'
import { useThemeStore } from '@/stores/themeStore'

export const MessageViewSkeleton = () => {
	const accentColor = useThemeStore((s) => s.accentColor)

	const SkeletonBlock = ({ className }: { className?: string }) => (
		<div className={`relative overflow-hidden bg-[var(--surface-active)] ${className}`}>
			<motion.div
				className='absolute inset-0'
				style={{
					background: `linear-gradient(90deg, transparent, ${accentColor}22, transparent)`,
				}}
				animate={{ x: ['-100%', '100%'] }}
				transition={{
					duration: 1.5,
					repeat: Infinity,
					ease: 'easeInOut',
				}}
			/>
		</div>
	)

	return (
		<div className='flex h-full flex-col bg-[var(--surface-panel)]'>
			{/* Header imitation */}
			<div className='flex h-14 items-center gap-2 border-b border-[var(--border-faint)] bg-[var(--surface-panel)] px-4'>
				<SkeletonBlock className='h-8 w-8 rounded-lg' />
				<div className='mx-2 h-6 w-px bg-[var(--border-subtle)]' />
				<SkeletonBlock className='h-8 w-8 rounded-lg' />
				<SkeletonBlock className='h-8 w-8 rounded-lg' />
				<SkeletonBlock className='h-8 w-8 rounded-lg' />
				<SkeletonBlock className='ml-auto h-8 w-24 rounded-lg' />
			</div>

			<div className='flex-1 overflow-y-auto px-6 py-4'>
				{/* Meta imitation */}
				<div className='space-y-4'>
					<SkeletonBlock className='h-7 w-3/4 rounded' />
					<div className='flex items-center gap-3'>
						<SkeletonBlock className='h-5 w-1/3 rounded' />
						<SkeletonBlock className='h-5 w-1/4 rounded' />
					</div>
					<SkeletonBlock className='h-4 w-48 rounded' />
				</div>

				{/* Body imitation */}
				<div className='mt-8 space-y-3'>
					<SkeletonBlock className='h-4 w-full rounded' />
					<SkeletonBlock className='h-4 w-[98%] rounded' />
					<SkeletonBlock className='h-4 w-[95%] rounded' />
					<SkeletonBlock className='h-4 w-[90%] rounded' />
					<SkeletonBlock className='h-4 w-[92%] rounded' />
					<SkeletonBlock className='h-4 w-[60%] rounded' />
				</div>

				<div className='mt-6 space-y-3'>
					<SkeletonBlock className='h-4 w-full rounded' />
					<SkeletonBlock className='h-4 w-[96%] rounded' />
					<SkeletonBlock className='h-4 w-[93%] rounded' />
					<SkeletonBlock className='h-4 w-[40%] rounded' />
				</div>
			</div>
		</div>
	)
}
