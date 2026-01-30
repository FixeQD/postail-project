import { memo, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { AlertTriangle, CheckCircle2, Info, Shield } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import type { SanitizeIssue } from '@/types/compose'

interface CompatibilityButtonProps {
	isOpen: boolean
	onClick: () => void
	issues: SanitizeIssue[]
	isLoading?: boolean
}

export const CompatibilityButton = memo(function CompatibilityButton({
	isOpen,
	onClick,
	issues,
	isLoading = false,
}: CompatibilityButtonProps) {
	const { t } = useTranslation('validation')

	const issueCounts = useMemo(() => {
		return {
			error: issues.filter((i) => i.severity === 'Error').length,
			warning: issues.filter((i) => i.severity === 'Warning').length,
			info: issues.filter((i) => i.severity === 'Info').length,
		}
	}, [issues])

	const totalIssues = issueCounts.error + issueCounts.warning + issueCounts.info

	const getIconAndColor = () => {
		if (totalIssues === 0) {
			return {
				icon: <CheckCircle2 className='h-4 w-4' />,
				colorClass: 'text-green-500',
				badgeClass: 'bg-green-500/20 text-green-500',
			}
		}
		if (issueCounts.error > 0) {
			return {
				icon: <AlertTriangle className='h-4 w-4' />,
				colorClass: 'text-red-500',
				badgeClass: 'bg-red-500/20 text-red-500',
			}
		}
		if (issueCounts.warning > 0) {
			return {
				icon: <AlertTriangle className='h-4 w-4' />,
				colorClass: 'text-yellow-500',
				badgeClass: 'bg-yellow-500/20 text-yellow-500',
			}
		}
		return {
			icon: <Info className='h-4 w-4' />,
			colorClass: 'text-blue-500',
			badgeClass: 'bg-blue-500/20 text-blue-500',
		}
	}

	const { icon, colorClass, badgeClass } = getIconAndColor()

	return (
		<TooltipProvider delayDuration={200}>
			<Tooltip>
				<TooltipTrigger asChild>
					<Button
						variant='ghost'
						size='icon'
						onClick={onClick}
						className={`relative h-9 w-9 transition-colors ${
							isOpen
								? 'bg-zinc-800 text-zinc-100'
								: `text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100 ${colorClass}`
						}`}>
						{isLoading ? (
							<span className='h-4 w-4 animate-pulse rounded-full bg-current' />
						) : (
							<>
								{totalIssues > 0 ? icon : <Shield className='h-4 w-4' />}
								{totalIssues > 0 && (
									<span
										className={`absolute -top-1 -right-1 flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[10px] font-medium ${badgeClass}`}>
										{totalIssues > 9 ? '9+' : totalIssues}
									</span>
								)}
							</>
						)}
					</Button>
				</TooltipTrigger>
				<TooltipContent side='bottom'>
					<p>{t('compatibilityPanel.toggleTooltip')}</p>
				</TooltipContent>
			</Tooltip>
		</TooltipProvider>
	)
})
