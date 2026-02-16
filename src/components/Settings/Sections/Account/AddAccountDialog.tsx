import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import * as opener from '@tauri-apps/plugin-opener'
import { useAccountsTranslation } from '../../../../hooks/useTypedTranslation'
import { Plus, Mail, Loader2, ArrowRight, Settings } from 'lucide-react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { ManualAccountForm } from './ManualAccountForm'

interface AddAccountDialogProps {
	onAccountAdded?: () => void
	children?: React.ReactNode
}

import { ComponentType } from 'react'

const ProviderOption = ({
	title,
	icon: Icon,
	onClick,
	isLoading,
	disabled,
	brandColor,
}: {
	title: string
	icon: ComponentType<{ className?: string }>
	onClick: () => void
	isLoading: boolean
	disabled: boolean
	brandColor: string
}) => (
	<button
		type='button'
		onClick={onClick}
		disabled={disabled}
		className={cn(
			'group relative flex w-full items-center justify-between overflow-hidden rounded-xl border border-white/5 bg-white/5 p-4 transition-all hover:border-white/10 hover:bg-white/10',
			disabled && 'cursor-not-allowed opacity-50'
		)}>
		<div
			className={cn(
				'absolute inset-0 bg-gradient-to-r opacity-0 transition-opacity group-hover:opacity-5',
				brandColor
			)}
		/>

		<div className='flex items-center gap-4'>
			<div
				className={cn(
					'flex h-10 w-10 items-center justify-center rounded-lg bg-slate-950/50 ring-1 ring-white/10',
					brandColor.replace('from-', 'text-')
				)}>
				<Icon className='h-5 w-5' />
			</div>
			<div className='text-left'>
				<h3 className='font-medium text-slate-100'>{title}</h3>
			</div>
		</div>

		{isLoading ? (
			<Loader2 className='h-5 w-5 animate-spin text-slate-400' />
		) : (
			<ArrowRight className='h-5 w-5 -translate-x-2 text-slate-500 opacity-0 transition-all group-hover:translate-x-0 group-hover:opacity-100' />
		)}
	</button>
)

export function AddAccountDialog({ children, onAccountAdded }: AddAccountDialogProps) {
	const { t } = useAccountsTranslation()
	const [loading, setLoading] = useState<string | null>(null)
	const [open, setOpen] = useState(false)
	const [showManualForm, setShowManualForm] = useState(false)

	const handleProviderClick = async (provider: 'gmail' | 'outlook') => {
		setLoading(provider)
		try {
			const { url } = await invoke<{ url: string; port: number }>('start_oauth_flow', {
				provider,
			})
			await opener.openUrl(url)
		} catch (error) {
			console.error(`Failed to start ${provider} OAuth:`, error)
			setLoading(null)
		}
	}

	const handleManualSuccess = () => {
		setShowManualForm(false)
		setOpen(false)
		onAccountAdded?.()
	}

	const handleManualCancel = () => {
		setShowManualForm(false)
	}

	const handleOpenChange = (newOpen: boolean) => {
		setOpen(newOpen)
		if (!newOpen) {
			setShowManualForm(false)
			setLoading(null)
		}
	}

	return (
		<Dialog open={open} onOpenChange={handleOpenChange}>
			<DialogTrigger asChild>
				{children || (
					<Button className='gap-2 bg-blue-600 text-white shadow-lg shadow-blue-500/20 hover:bg-blue-500'>
						<Plus className='h-4 w-4' />
						{t('settings:accounts.list.add')}
					</Button>
				)}
			</DialogTrigger>
			<DialogContent className='border-slate-800 bg-slate-900/95 text-slate-100 backdrop-blur-xl sm:max-w-md'>
				<DialogHeader>
					<DialogTitle className='text-xl font-bold'>
						{t('settings:accounts.title')}
					</DialogTitle>
					<DialogDescription className='text-slate-400'>
						{t('settings:accounts.subtitle')}
					</DialogDescription>
				</DialogHeader>

				{showManualForm ? (
					<ManualAccountForm
						onSuccess={handleManualSuccess}
						onCancel={handleManualCancel}
					/>
				) : (
					<div className='grid gap-3 py-4'>
						<ProviderOption
							title={t('settings:accounts.providers.gmail.title')}
							icon={Mail}
							brandColor='from-red-500 to-orange-500'
							onClick={() => handleProviderClick('gmail')}
							isLoading={loading === 'gmail'}
							disabled={loading !== null}
						/>
						<ProviderOption
							title={t('settings:accounts.providers.outlook.title')}
							icon={Mail}
							brandColor='from-blue-500 to-cyan-500'
							onClick={() => handleProviderClick('outlook')}
							isLoading={loading === 'outlook'}
							disabled={loading !== null}
						/>
						<ProviderOption
							title={t('settings:accounts.providers.imap.title')}
							icon={Settings}
							brandColor='from-slate-500 to-slate-400'
							onClick={() => setShowManualForm(true)}
							isLoading={false}
							disabled={loading !== null}
						/>
					</div>
				)}
			</DialogContent>
		</Dialog>
	)
}
