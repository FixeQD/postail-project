import { parseAddresses } from '@/lib/parseAddress'
import type { MailHeader } from '@/types/mail'
import { useState } from 'react'

interface MessageViewMetaProps {
	header: MailHeader
	showDetails?: boolean
}

export const MessageViewMeta = ({
	header,
	showDetails = false,
}: MessageViewMetaProps) => {
	const [expanded, setExpanded] = useState(showDetails)

	const from = parseAddresses(header.from)[0]
	const to = parseAddresses(header.to)
	const cc = header.flags.includes('cc') ? [] : [] // MailHeader doesn't have cc yet, placeholder
	// To format: "Feb 18, 2026 at 18:32"
	const dateStr = new Date(header.internal_date).toLocaleString('en-US', {
		month: 'short',
		day: 'numeric',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		hour12: false,
	})

	return (
		<div className='flex flex-col gap-4 border-b border-white/[0.06] pb-6'>
			{/* Subject */}
			<h1 className='text-lg font-semibold leading-relaxed text-white'>
				{header.subject || '(No Subject)'}
			</h1>

			<div className='flex flex-col gap-2 text-sm'>
				{/* From */}
				<div className='flex items-baseline gap-2'>
					<span className='w-12 shrink-0 text-slate-400'>From:</span>
					<div className='flex flex-wrap items-baseline gap-1.5'>
						<span className='font-semibold text-slate-200'>
							{from.name}
						</span>
						<span className='text-xs text-slate-500'>
							&lt;{from.email}&gt;
						</span>
					</div>
				</div>

				{/* Date */}
				<div className='flex items-baseline gap-2'>
					<span className='w-12 shrink-0 text-slate-400'>Date:</span>
					<span className='text-slate-300'>{dateStr}</span>
				</div>

				{/* To */}
				<div className='flex items-baseline gap-2'>
					<span className='w-12 shrink-0 text-slate-400'>To:</span>
					<div className='flex flex-wrap gap-1'>
						{to.map((recipient, i) => (
							<span key={i} className='text-slate-300'>
								{recipient.name || recipient.email}
								{i < to.length - 1 && ','}
							</span>
						))}
					</div>
				</div>

				{/* CC Placeholder - render conditionally if we had CC data */}
				{/* {cc.length > 0 && (
          <div className="flex items-baseline gap-2">
            <span className="w-12 shrink-0 text-slate-400">Cc:</span>
             ...
          </div>
        )} */}
			</div>
		</div>
	)
}
