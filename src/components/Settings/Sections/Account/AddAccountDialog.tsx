import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import * as opener from '@tauri-apps/plugin-opener'
import { useAccountsTranslation } from '@/hooks/useTypedTranslation'
import { Plus, Mail, Loader2, ArrowRight, Settings } from 'lucide-react'
import { MailboxRoleStep } from './MailboxRoleStep'
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
import { useAccountStore } from '@/stores/accountStore'
import { useShellTransition } from '@/hooks/useShellTransition'
import type { ComponentType } from 'react'
import type { AddAccountDialogProps } from '@/types/components/shared'

type DialogView = 'providers' | 'manual' | 'mailbox-roles'

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
			'group relative flex w-full items-center justify-between overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-4 transition-all hover:border-[var(--text-tertiary)] hover:bg-[var(--surface-hover)]',
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
					'flex h-10 w-10 items-center justify-center rounded-lg bg-[var(--surface-hover)] ring-1 ring-[var(--border-subtle)]',
					brandColor.replace('from-', 'text-')
				)}>
				<Icon className='h-5 w-5' />
			</div>
			<h3 className='font-medium text-[var(--text-primary)]'>{title}</h3>
		</div>
		{isLoading ? (
			<Loader2 className='h-5 w-5 animate-spin text-[var(--text-secondary)]' />
		) : (
			<ArrowRight className='h-5 w-5 -translate-x-2 text-[var(--text-secondary)] opacity-0 transition-all group-hover:translate-x-0 group-hover:opacity-100 ' />
		)}
	</button>
)

export function AddAccountDialog({ children, onAccountAdded }: AddAccountDialogProps) {
	const { t } = useAccountsTranslation()
	const fetchAccounts = useAccountStore((s) => s.fetchAccounts)
	const [loading, setLoading] = useState<string | null>(null)
	const [open, setOpen] = useState(false)
	const [view, setView] = useState<DialogView>('providers')
	const [newAccountId, setNewAccountId] = useState<string | null>(null)
	const [availableProviders, setAvailableProviders] = useState<string[]>([])
	const { shellScope, contentScope, transition, reset } = useShellTransition()

	useEffect(() => {
		if (open) {
			invoke<{ providers: string[] }>('get_available_providers')
				.then((res) => setAvailableProviders(res.providers))
				.catch(console.error)
		}
	}, [open])

	const switchTo = (next: DialogView) => transition(() => setView(next))

	const handleProviderClick = async (provider: 'gmail' | 'outlook') => {
		setLoading(provider)
		try {
			const { url } = await invoke<{ url: string; port: number }>('start_oauth_flow', {
				provider,
			})
			await opener.openUrl(url)
		} catch (e) {
			console.error(e)
			setLoading(null)
		}
	}

	const handleManualSuccess = (accountId: string) => {
		setNewAccountId(accountId)
		transition(() => setView('mailbox-roles'))
	}

	const handleMailboxRolesDone = () => {
		setOpen(false)
		onAccountAdded ? onAccountAdded(newAccountId ?? undefined) : fetchAccounts()
	}

	const handleOpenChange = (newOpen: boolean) => {
		if (!newOpen) {
			reset()
		}
		setOpen(newOpen)
	}

	const handleAnimationEnd = () => {
		if (!open) {
			setView('providers')
			setNewAccountId(null)
			setLoading(null)
		}
	}

	return (
		<Dialog open={open} onOpenChange={handleOpenChange}>
			<DialogTrigger asChild>
				{children || (
					<Button className='gap-2 bg-status-info text-white shadow-lg shadow-blue-500/20 hover:bg-status-info'>
						<Plus className='h-4 w-4' />
						{t('settings:accounts.list.add')}
					</Button>
				)}
			</DialogTrigger>

			<DialogContent
				onAnimationEnd={handleAnimationEnd}
				className='glass overflow-hidden border-[var(--border-subtle)] bg-[var(--surface-glass)] p-0 text-[var(--text-primary)]'>
				{/* Shell: we pin its height in px during the resize animation */}
				<div ref={shellScope} className='w-full'>
					{/* Content wrapper: we fade this independently */}
					<div ref={contentScope} className='p-6'>
						{view === 'mailbox-roles' && newAccountId ? (
							<MailboxRoleStep
								accountId={newAccountId}
								onDone={handleMailboxRolesDone}
							/>
						) : (
							<>
								<DialogHeader className='mb-2'>
									<DialogTitle className='text-xl font-bold'>
										{t('settings:accounts.title')}
									</DialogTitle>
									<DialogDescription className='text-[var(--text-secondary)]'>
										{t('settings:accounts.subtitle')}
									</DialogDescription>
								</DialogHeader>

								{view === 'providers' ? (
									<div className='grid gap-3 py-4'>
										<ProviderOption
											title={t('settings:accounts.providers.gmail.title')}
											icon={Mail}
											brandColor={
												availableProviders.includes('gmail')
													? 'from-red-500 to-orange-500'
													: 'from-slate-500 to-slate-400'
											}
											onClick={() => handleProviderClick('gmail')}
											isLoading={loading === 'gmail'}
											disabled={
												!availableProviders.includes('gmail') ||
												loading !== null
											}
										/>
										<ProviderOption
											title={t('settings:accounts.providers.outlook.title')}
											icon={Mail}
											brandColor={
												availableProviders.includes('outlook')
													? 'from-blue-500 to-cyan-500'
													: 'from-slate-500 to-slate-400'
											}
											onClick={() => handleProviderClick('outlook')}
											isLoading={loading === 'outlook'}
											disabled={
												!availableProviders.includes('outlook') ||
												loading !== null
											}
										/>
										<ProviderOption
											title={t('settings:accounts.providers.imap.title')}
											icon={Settings}
											brandColor='from-slate-500 to-slate-400'
											onClick={() => switchTo('manual')}
											isLoading={false}
											disabled={loading !== null}
										/>
									</div>
								) : (
									<ManualAccountForm
										onSuccess={handleManualSuccess}
										onCancel={() => switchTo('providers')}
									/>
								)}
							</>
						)}
					</div>
				</div>
			</DialogContent>
		</Dialog>
	)
}
