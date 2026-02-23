import { useState } from 'react'
import { ChevronDown } from 'lucide-react'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { parseAddresses } from '@/lib/parseAddress'
import type { MailHeader } from '@/types/mail'
import { motion, AnimatePresence } from 'framer-motion'
import i18n from '@/i18n'

interface MessageViewMetaProps {
	header: MailHeader
}

// Generate initials + deterministic hue from email string
const senderAvatar = (name: string, email: string) => {
	const initials = name
		? name
				.split(' ')
				.slice(0, 2)
				.map((w) => w[0])
				.join('')
				.toUpperCase()
		: email.slice(0, 2).toUpperCase()

	let hash = 0
	for (let i = 0; i < email.length; i++) hash = email.charCodeAt(i) + ((hash << 5) - hash)
	const hue = Math.abs(hash) % 360

	return { initials, hue }
}

const formatDate = (iso: string) =>
	new Date(iso).toLocaleString(i18n.t('app.languageCode'), {
		weekday: 'short',
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		hour12: false,
	})

export const MessageViewMeta = ({ header }: MessageViewMetaProps) => {
	const animationsEnabled = useAnimationsEnabled()
	const [expanded, setExpanded] = useState(false)

	const from = parseAddresses(header.from)[0]
	const to = parseAddresses(header.to)
	const cc = header.cc?.length ? parseAddresses(header.cc) : []
	const dateStr = formatDate(header.internal_date)
	const { initials, hue } = senderAvatar(from?.name || '', from?.email || '')

	const toStr = to.map((r) => r.name || r.email).join(', ')
	const recipientSummary = cc.length > 0 ? `${toStr} +${cc.length} more` : toStr

	return (
		<div className='flex items-start gap-3 px-5 py-4'>
			{/* Avatar */}
			<div
				className='mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-xs font-bold'
				style={{
					background: `hsl(${hue} 55% 22%)`,
					color: `hsl(${hue} 80% 75%)`,
					boxShadow: `0 0 0 1px hsl(${hue} 55% 30% / 0.5)`,
				}}>
				{initials}
			</div>

			{/* Sender + meta */}
			<div className='min-w-0 flex-1'>
				{/* Row 1: name + date */}
				<div className='flex items-baseline justify-between gap-3'>
					<div className='flex min-w-0 items-baseline gap-2'>
						<span className='truncate text-sm font-semibold text-slate-100'>
							{from?.name || from?.email}
						</span>
						{from?.name && (
							<span className='shrink-0 text-xs text-slate-500'>{from.email}</span>
						)}
					</div>
					<span className='shrink-0 text-xs text-slate-500'>{dateStr}</span>
				</div>

				{/* Row 2: to summary + expand toggle */}
				<button
					type='button'
					onClick={() => setExpanded((e) => !e)}
					className='mt-0.5 flex items-center gap-1 text-xs text-slate-500 transition-colors hover:text-slate-400'>
					<span>to {recipientSummary}</span>
					<motion.div
						animate={{ rotate: expanded ? 180 : 0 }}
						transition={{ duration: 0.2 }}>
						<ChevronDown className='h-3 w-3' />
					</motion.div>
				</button>

				{/* Expanded details */}
				<AnimatePresence>
					{expanded && (
						<motion.div
							initial={{ opacity: 0, height: 0 }}
							animate={{ opacity: 1, height: 'auto' }}
							exit={{ opacity: 0, height: 0 }}
							transition={{ duration: animationsEnabled ? 0.2 : 0 }}
							className='overflow-hidden'>
							<div className='mt-2.5 flex flex-col gap-1.5 rounded-lg border border-white/[0.05] bg-slate-900/40 px-3 py-2.5 text-xs'>
								<MetaRow label='From'>
									<span className='text-slate-200'>
										{from?.name}{' '}
										<span className='text-slate-400'>
											&lt;{from?.email}&gt;
										</span>
									</span>
								</MetaRow>
								<MetaRow label='To'>
									<span className='text-slate-200'>
										{to.map((r, i) => (
											<span key={i}>
												{r.name ? (
													<>
														{r.name}{' '}
														<span className='text-slate-400'>
															&lt;{r.email}&gt;
														</span>
													</>
												) : (
													r.email
												)}
												{i < to.length - 1 && ', '}
											</span>
										))}
									</span>
								</MetaRow>
								{cc.length > 0 && (
									<MetaRow label='Cc'>
										<span className='text-slate-200'>
											{cc.map((r, i) => (
												<span key={i}>
													{r.name || r.email}
													{i < cc.length - 1 && ', '}
												</span>
											))}
										</span>
									</MetaRow>
								)}
								<MetaRow label='Date'>
									<span className='text-slate-200'>{dateStr}</span>
								</MetaRow>
							</div>
						</motion.div>
					)}
				</AnimatePresence>
			</div>
		</div>
	)
}

const MetaRow = ({ label, children }: { label: string; children: React.ReactNode }) => (
	<div className='flex items-baseline gap-2'>
		<span className='w-8 shrink-0 text-right text-slate-500'>{label}</span>
		<div className='min-w-0 flex-1'>{children}</div>
	</div>
)
