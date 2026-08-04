import { memo, useState, useCallback, useRef, useEffect, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
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
	onAutoFix?: () => void
	hasIssues: boolean
	composeX: number
	composeY: number
	composeHeight: number
}

const severityConfig: Record<
	IssueSeverity,
	{
		icon: typeof XCircle
		color: string
		bgColor: string
		borderColor: string
		label: string
	}
> = {
	Error: {
		icon: XCircle,
		color: 'text-destructive',
		bgColor: 'bg-destructive/15',
		borderColor: 'border-l-red-500',
		label: 'Error',
	},
	Warning: {
		icon: AlertTriangle,
		color: 'text-status-warning',
		bgColor: 'bg-status-warning/15',
		borderColor: 'border-l-yellow-500',
		label: 'Warning',
	},
	Info: {
		icon: Info,
		color: 'text-status-info',
		bgColor: 'bg-status-info/15',
		borderColor: 'border-l-blue-500',
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
		<div
			className={`flex gap-3 border-l-2 bg-[var(--compose-context-bg)] p-3.5 ${config.borderColor} ${!isLast ? 'border-b border-[var(--compose-input-border)]' : ''} transition-colors hover:bg-[var(--compose-hover)]`}>
			<div className={`mt-0.5 flex-shrink-0 ${config.color}`}>
				<Icon className='h-4 w-4' />
			</div>
			<div className='min-w-0 flex-1'>
				<div className='mb-1.5 flex items-center gap-2'>
					<code className='font-mono text-xs font-bold text-[var(--compose-text)]'>
						{issue.property}
					</code>
					{issue.count > 1 && (
						<Badge
							variant='secondary'
							className='scale-90 bg-[var(--compose-active)] px-1.5 py-0 text-[10px] text-[var(--compose-text-muted)]'>
							x{issue.count}
						</Badge>
					)}
					<Badge
						variant='outline'
						className={`ml-auto px-1.5 py-0 text-[10px] ${config.bgColor} ${config.color} border-current opacity-80`}>
						{t(`compatibilityPanel.severity.${issue.severity.toLowerCase()}`)}
					</Badge>
				</div>
				<p className='text-xs leading-relaxed text-[var(--compose-text-muted)]'>
					{issue.reason}
				</p>
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
				className='flex w-full items-center gap-2 rounded-md px-3 py-2 text-left transition-colors hover:bg-[var(--compose-hover)]'>
				{isExpanded ? (
					<ChevronDown className='h-3.5 w-3.5 text-[var(--compose-text-muted)]' />
				) : (
					<ChevronRight className='h-3.5 w-3.5 text-[var(--compose-text-muted)]' />
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
			<div
				className={`overflow-hidden transition-all duration-200 ${isExpanded ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0'}`}>
				<div className='mt-1 rounded-md border border-[var(--compose-ring)] bg-[var(--compose-context-bg)]'>
					{issues.map((issue, index) => (
						<IssueItem
							key={`${issue.property}-${index}`}
							issue={issue}
							isLast={index === issues.length - 1}
						/>
					))}
				</div>
			</div>
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
	onAutoFix,
	hasIssues: hasIssuesProp,
	composeX,
	composeY,
	composeHeight,
}: CompatibilityPanelProps) {
	const { t } = useTranslation('validation')
	const resizerRef = useRef<HTMLDivElement>(null)
	const [isResizing, setIsResizing] = useState(false)

	const groupedIssues = useMemo(
		() => ({
			Error: issues.filter((i) => i.severity === 'Error'),
			Warning: issues.filter((i) => i.severity === 'Warning'),
			Info: issues.filter((i) => i.severity === 'Info'),
		}),
		[issues]
	)

	const totalIssues = issues.length
	const hasIssues = hasIssuesProp || totalIssues > 0

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

	const handleAutoFix = useCallback(async () => {
		if (!onAutoFix) return
		await onAutoFix()
	}, [onAutoFix])

	useEffect(() => {
		if (isResizing) {
			document.addEventListener('mousemove', handleResizeMove)
			document.addEventListener('mouseup', handleResizeEnd)
			return () => {
				document.removeEventListener('mousemove', handleResizeMove)
				document.removeEventListener('mouseup', handleResizeEnd)
			}
		}
	}, [isResizing, handleResizeMove, handleResizeEnd])

	const panelStyle = useMemo(
		(): React.CSSProperties => ({
			width,
			left: composeX - width,
			top: composeY,
			height: composeHeight,
			transform: isOpen ? 'translateX(0)' : `translateX(-${width}px)`,
			opacity: isOpen ? 1 : 0,
			transition: 'transform 200ms ease-out, opacity 200ms ease-out',
			pointerEvents: isOpen ? 'auto' : 'none',
		}),
		[width, composeX, composeY, composeHeight, isOpen]
	)

	return (
		<div
			className='fixed z-[60] flex flex-col border-r border-[var(--compose-ring)] bg-[var(--compose-bg)] shadow-2xl'
			style={panelStyle}>
			{/* Resizer handle */}
			<div
				ref={resizerRef}
				role='separator'
				aria-orientation='vertical'
				aria-label={t('aria.resizePanel')}
				aria-valuenow={width}
				tabIndex={0}
				onMouseDown={handleResizeStart}
				className={`absolute top-0 right-0 bottom-0 z-10 w-1 cursor-col-resize transition-colors ${
					isResizing ? 'bg-status-info' : 'hover:bg-status-info/15'
				}`}
			/>

			<Card className='relative flex flex-1 flex-col overflow-hidden rounded-none border-0 bg-transparent'>
				<CardHeader className='flex-shrink-0 border-b border-[var(--compose-input-border)] px-4 py-3 pb-3'>
					<div className='flex items-center justify-between'>
						<CardTitle className='flex items-center gap-2 text-sm font-semibold text-[var(--compose-text)]'>
							{t('compatibilityPanel.title')}
							{isLoading && (
								<span className='h-2 w-2 animate-pulse rounded-full bg-status-info' />
							)}
						</CardTitle>
						<Button
							variant='ghost'
							size='icon'
							className='h-7 w-7 text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)] hover:text-[var(--compose-text)]'
							onClick={onClose}>
							<X className='h-4 w-4' />
						</Button>
					</div>

					{/* Summary badges */}
					<div className='mt-2 flex items-center gap-2'>
						{groupedIssues.Error.length > 0 && (
							<Badge
								variant='outline'
								className='border-destructive/30 bg-destructive/15 text-[10px] text-destructive'>
								{t('compatibilityPanel.issues.error', {
									count: groupedIssues.Error.length,
								})}
							</Badge>
						)}
						{groupedIssues.Warning.length > 0 && (
							<Badge
								variant='outline'
								className='border-status-warning/30 bg-status-warning/15 text-[10px] text-status-warning'>
								{t('compatibilityPanel.issues.warning', {
									count: groupedIssues.Warning.length,
								})}
							</Badge>
						)}
						{groupedIssues.Info.length > 0 && (
							<Badge
								variant='outline'
								className='border-status-info/30 bg-status-info/15 text-[10px] text-status-info'>
								{t('compatibilityPanel.issues.info', {
									count: groupedIssues.Info.length,
								})}
							</Badge>
						)}
						{!hasIssues && !isLoading && (
							<span className='flex items-center gap-1 text-[10px] text-status-success'>
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
							<div className='mb-3 flex h-10 w-10 items-center justify-center rounded-full bg-status-success/15'>
								<Info className='h-5 w-5 text-status-success' />
							</div>
							<p className='text-sm text-[var(--compose-text-muted)]'>
								{t('compatibilityPanel.issues.none')}
							</p>
							<p className='mt-1 text-xs text-[var(--compose-placeholder)]'>
								Your email looks good!
							</p>
						</div>
					)}
				</CardContent>

				{/* Action buttons */}
				<div className='flex-shrink-0 space-y-2 border-t border-[var(--compose-input-border)] p-3'>
					<Button
						variant='outline'
						size='sm'
						className='h-8 w-full border-[var(--compose-ring)] bg-[var(--compose-context-bg)] text-xs text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)] hover:text-[var(--compose-text)]'
						onClick={onCheckAgain}
						disabled={isLoading}>
						<RefreshCw
							className={`mr-1.5 h-3.5 w-3.5 ${isLoading ? 'animate-spin' : ''}`}
						/>
						{t('compatibilityPanel.actions.checkAgain')}
					</Button>

					{/* Auto-fix button */}
					<Button
						variant='outline'
						size='sm'
						className={`h-8 w-full border-[var(--compose-ring)] bg-[var(--compose-context-bg)] text-xs hover:bg-[var(--compose-hover)] ${
							hasIssues
								? 'text-status-warning hover:text-status-warning'
								: 'cursor-not-allowed text-[var(--compose-placeholder)] opacity-50'
						}`}
						onClick={handleAutoFix}
						disabled={!hasIssues || isLoading}>
						<Wand2 className='mr-1.5 h-3.5 w-3.5' />
						{t('compatibilityPanel.actions.autoFix')}
					</Button>
				</div>
			</Card>
		</div>
	)
})
