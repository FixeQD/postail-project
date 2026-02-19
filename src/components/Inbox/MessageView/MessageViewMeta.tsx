import { parseAddresses } from '@/lib/parseAddress'
import type { MailHeader } from '@/types/mail'
import { motion, type Variants } from 'framer-motion'


interface MessageViewMetaProps {
	header: MailHeader
	showDetails?: boolean
}

export const MessageViewMeta = ({
	header,
	showDetails = true, // Defaulting to true for now since toggle logic isn't fully there yet
}: MessageViewMetaProps) => {

	const from = parseAddresses(header.from)[0]
	const to = parseAddresses(header.to)
	// MailHeader doesn't have cc yet, placeholder

	// To format: "Feb 18, 2026 at 18:32"
	const dateStr = new Date(header.internal_date).toLocaleString('en-US', {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		hour12: false,
	})

	const container: Variants = {
		hidden: { opacity: 0 },
		show: {
			opacity: 1,
			transition: {
				staggerChildren: 0.05,
				delayChildren: 0.1,
			},
		},
	}

	const item: Variants = {
		hidden: { opacity: 0, x: -10 },
		show: { 
			opacity: 1, 
			x: 0, 
			transition: { duration: 0.3, ease: 'easeOut' } 
		},
	}

	return (
		<motion.div 
			className='flex flex-col gap-4 border-b border-white/[0.06] pb-6'
			initial={showDetails ? 'show' : 'hidden'}
			animate='show'
			variants={container}
		>
			{/* Subject */}
			<motion.h1 variants={item} className='text-lg font-semibold leading-relaxed text-white'>
				{header.subject || '(No Subject)'}
			</motion.h1>

			<div className='flex items-start gap-4'>
				{/* Avatar Placeholder */}
				<div className='mt-0.5 h-8 w-8 shrink-0 rounded-full bg-white/[0.03]' />

				<motion.div variants={container} className='flex flex-1 flex-col gap-2 text-sm'>
					{/* From */}
					<motion.div variants={item} className='flex items-baseline gap-2'>
						<span className='w-12 shrink-0 text-slate-400'>From:</span>
						<div className='flex flex-wrap items-baseline gap-1.5'>
							<span className='font-semibold text-slate-200'>
								{from.name}
							</span>
							<span className='text-xs text-slate-500'>
								&lt;{from.email}&gt;
							</span>
						</div>
					</motion.div>

					{/* Date */}
					<motion.div variants={item} className='flex items-baseline gap-2'>
						<span className='w-12 shrink-0 text-slate-400'>Date:</span>
						<span className='text-slate-300'>{dateStr}</span>
					</motion.div>

					{/* To */}
					<motion.div variants={item} className='flex items-baseline gap-2'>
						<span className='w-12 shrink-0 text-slate-400'>To:</span>
						<div className='flex flex-wrap gap-1'>
							{to.map((recipient, i) => (
								<span key={i} className='text-slate-300'>
									{recipient.name || recipient.email}
									{i < to.length - 1 && ','}
								</span>
							))}
						</div>
					</motion.div>
				</motion.div>
			</div>
		</motion.div>
	)
}
