import { memo, useState, useCallback, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import {
	X,
	AlertTriangle,
	Info,
	XCircle,
	Wand2,
	RefreshCw,
	ChevronDown,
	ChevronRight,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import type { SanitizeIssue, IssueSeverity } from '@/types/compose'

interface CompatibilityPanelProps {
	isOpen: boolean
	onClose: () => void
	width: number
	onWidthChange: (width: number) => void
	issues: SanitizeIssue[]
	isLoading: boolean
	onCheckAgain: () => void
	composeX: number
	composeY: number
	composeHeight: number
}

const severityConfig: Record<
	IssueSeverity,
	{ icon: typeof XCircle; color: string; bgColor: string; label: string }
> = {
	Error: {
		icon: XCircle,
		color: 'text-red-500',
		bgColor: 'bg-red-500/10',
		label: 'Error',
	},
	Warning: {
		icon: AlertTriangle,
		color: 'text-yellow-500',
		bgColor: 'bg-yellow-500/10',
		label: 'Warning',
	},
	Info: {
		icon: Info,
		color: 'text-blue-500',
		bgColor: 'bg-blue-500/10',
		label: 'Info',
	},
}

const IssueItem = memo(function IssueItem({
	issue,
	isLast,
}: {
	issue: SanitizeIssue
	isLast: boolean
}) {
	const { t } = useTranslation('validation')
	const config = severityConfig[issue.severity]
	const Icon = config.icon

	return (
		<div className={`flex gap-2 p-3 ${!isLast ? 'border-b border-zinc-800' : ''}`}>
			<div className={`mt-0.5 flex-shrink-0 ${config.color}`}>
				<Icon className='h-4 w-4' />
			</div>
			<div className='min-w-0 flex-1'>
				<div className='mb-1 flex items-center gap-2'>
					<Badge
						variant='outline'
						className={`px-1.5 py-0 text-[10px] ${config.bgColor} ${config.color} border-current`}>
						{t(`compatibilityPanel.severity.${issue.severity.toLowerCase()}`)}
					</Badge>
					<code className='font-mono text-[11px] text-zinc-500'>{issue.property}</code>
				</div>
				<p className='text-xs leading-relaxed text-zinc-400'>{issue.reason}</p>
			</div>
		</div>
	)
})

const IssueGroup = memo(function IssueGroup({
	severity,
	issues,
}: {
	severity: IssueSeverity
	issues: SanitizeIssue[]
}) {
	const { t } = useTranslation('validation')
	const [isExpanded, setIsExpanded] = useState(true)
	const config = severityConfig[severity]
	const Icon = config.icon

	if (issues.length === 0) return null

	return (
		<div className='mb-2'>
			<button
				type='button'
				onClick={() => setIsExpanded(!isExpanded)}
				className='flex w-full items-center gap-2 rounded-md px-3 py-2 text-left transition-colors hover:bg-zinc-800/50'>
				{isExpanded ? (
					<ChevronDown className='h-3.5 w-3.5 text-zinc-500' />
				) : (
					<ChevronRight className='h-3.5 w-3.5 text-zinc-500' />
				)}
				<Icon className={`h-4 w-4 ${config.color}`} />
				<span className={`text-xs font-medium ${config.color}`}>
					{t(`compatibilityPanel.severity.${severity.toLowerCase()}`)}s
				</span>
				<Badge
					variant='outline'
					className={`ml-auto px-1.5 py-0 text-[10px] ${config.bgColor} ${config.color} border-current`}>
					{issues.length}
				</Badge>
			</button>
			<AnimatePresence>
				{isExpanded && (
					<motion.div
						initial={{ height: 0, opacity: 0 }}
						animate={{ height: 'auto', opacity: 1 }}
						exit={{ height: 0, opacity: 0 }}
						transition={{ duration: 0.2 }}
						className='overflow-hidden'>
						<div className='mt-1 rounded-md border border-zinc-800/50 bg-zinc-900/50'>
							{issues.map((issue, index) => (
								<IssueItem
									key={`${issue.property}-${index}`}
									issue={issue}
									isLast={index === issues.length - 1}
								/>
							))}
						</div>
					</motion.div>
				)}
			</AnimatePresence>
		</div>
	)
})

export const CompatibilityPanel = memo(function CompatibilityPanel({
	isOpen,
	onClose,
	width,
	onWidthChange,
	issues,
	isLoading,
	onCheckAgain,
	composeX,
	composeY,
	composeHeight,
}: CompatibilityPanelProps) {
	const { t } = useTranslation('validation')
	const resizerRef = useRef<HTMLDivElement>(null)
	const [isResizing, setIsResizing] = useState(false)

	const groupedIssues = {
		Error: issues.filter((i) => i.severity === 'Error'),
		Warning: issues.filter((i) => i.severity === 'Warning'),
		Info: issues.filter((i) => i.severity === 'Info'),
	}

	const totalIssues = issues.length
	const hasIssues = totalIssues > 0

	const handleResizeStart = useCallback(() => {
		setIsResizing(true)
	}, [])

	const handleResizeMove = useCallback(
		(e: MouseEvent) => {
			if (!isResizing) return
			const newWidth = Math.max(200, Math.min(500, e.clientX))
			onWidthChange(newWidth)
		},
		[isResizing, onWidthChange]
	)

	const handleResizeEnd = useCallback(() => {
		setIsResizing(false)
	}, [])

	useState(() => {
		if (isResizing) {
			document.addEventListener('mousemove', handleResizeMove)
			document.addEventListener('mouseup', handleResizeEnd)
			return () => {
				document.removeEventListener('mousemove', handleResizeMove)
				document.removeEventListener('mouseup', handleResizeEnd)
			}
		}
	})

	return (
		<AnimatePresence>
			{isOpen && (
				<motion.div
					initial={{ x: -width, opacity: 0 }}
					animate={{ x: 0, opacity: 1 }}
					exit={{ x: -width, opacity: 0 }}
					transition={{ type: 'spring', damping: 25, stiffness: 200 }}
					className='fixed z-[60] flex flex-col border-r border-zinc-800 bg-zinc-950 shadow-2xl'
					style={{
						width,
						left: composeX - width,
						top: composeY,
						height: composeHeight,
					}}>
					{/* Resizer handle */}
					<div
						ref={resizerRef}
						role='separator'
						aria-orientation='vertical'
						aria-label='Resize panel'
						onMouseDown={handleResizeStart}
						className={`absolute top-0 right-0 bottom-0 z-10 w-1 cursor-col-resize transition-colors ${
							isResizing ? 'bg-blue-500' : 'hover:bg-blue-500/50'
						}`}
					/>

					<Card className='flex flex-1 flex-col overflow-hidden rounded-none border-0 bg-transparent'>
						<CardHeader className='flex-shrink-0 border-b border-zinc-800/50 px-4 py-3 pb-3'>
							<div className='flex items-center justify-between'>
								<CardTitle className='flex items-center gap-2 text-sm font-semibold text-zinc-200'>
									{t('compatibilityPanel.title')}
									{isLoading && (
										<span className='h-2 w-2 animate-pulse rounded-full bg-blue-500' />
									)}
								</CardTitle>
								<Button
									variant='ghost'
									size='icon'
									className='h-7 w-7 text-zinc-500 hover:bg-zinc-800 hover:text-zinc-300'
									onClick={onClose}>
									<X className='h-4 w-4' />
								</Button>
							</div>

							{/* Summary badges */}
							<div className='mt-2 flex items-center gap-2'>
								{groupedIssues.Error.length > 0 && (
									<Badge
										variant='outline'
										className='border-red-500/30 bg-red-500/10 text-[10px] text-red-500'>
										{t('compatibilityPanel.issues.error', {
											count: groupedIssues.Error.length,
										})}
									</Badge>
								)}
								{groupedIssues.Warning.length > 0 && (
									<Badge
										variant='outline'
										className='border-yellow-500/30 bg-yellow-500/10 text-[10px] text-yellow-500'>
										{t('compatibilityPanel.issues.warning', {
											count: groupedIssues.Warning.length,
										})}
									</Badge>
								)}
								{groupedIssues.Info.length > 0 && (
									<Badge
										variant='outline'
										className='border-blue-500/30 bg-blue-500/10 text-[10px] text-blue-500'>
										{t('compatibilityPanel.issues.info', {
											count: groupedIssues.Info.length,
										})}
									</Badge>
								)}
								{!hasIssues && !isLoading && (
									<span className='flex items-center gap-1 text-[10px] text-green-500'>
										<Info className='h-3 w-3' />
										{t('compatibilityPanel.issues.none')}
									</span>
								)}
							</div>
						</CardHeader>

						<CardContent className='custom-scrollbar flex-1 space-y-1 overflow-y-auto p-3'>
							{hasIssues ? (
								<>
									<IssueGroup severity='Error' issues={groupedIssues.Error} />
									<IssueGroup severity='Warning' issues={groupedIssues.Warning} />
									<IssueGroup severity='Info' issues={groupedIssues.Info} />
								</>
							) : (
								<div className='flex h-32 flex-col items-center justify-center text-center'>
									<div className='mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-green-500/10'>
										<Info className='h-5 w-5 text-green-500' />
									</div>
									<p className='text-sm text-zinc-400'>
										{t('compatibilityPanel.issues.none')}
									</p>
									<p className='mt-1 text-xs text-zinc-600'>
										Your email looks good!
									</p>
								</div>
							)}
						</CardContent>

						{/* Action buttons */}
						<div className='flex-shrink-0 space-y-2 border-t border-zinc-800/50 p-3'>
							<Button
								variant='outline'
								size='sm'
								className='h-8 w-full border-zinc-700 bg-zinc-900 text-xs text-zinc-300 hover:bg-zinc-800 hover:text-zinc-200'
								onClick={onCheckAgain}
								disabled={isLoading}>
								<RefreshCw
									className={`mr-1.5 h-3.5 w-3.5 ${isLoading ? 'animate-spin' : ''}`}
								/>
								{t('compatibilityPanel.actions.checkAgain')}
							</Button>

							{/* Auto-fix placeholder */}
							<Button
								variant='outline'
								size='sm'
								className='h-8 w-full cursor-not-allowed border-zinc-700 bg-zinc-900 text-xs text-zinc-500 opacity-50 hover:bg-zinc-800 hover:text-zinc-400'
								disabled>
								<Wand2 className='mr-1.5 h-3.5 w-3.5' />
								{t('compatibilityPanel.actions.autoFix')}
								<span className='ml-1.5 text-[10px] opacity-60'>(soon)</span>
							</Button>
						</div>
					</Card>
				</motion.div>
			)}
		</AnimatePresence>
	)
})
