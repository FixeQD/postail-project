import { useState, useEffect, useRef } from 'react'
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
import { useAccountStore } from '@/stores/accountStore'
import { useAnimate } from 'framer-motion'
import { ComponentType } from 'react'

interface AddAccountDialogProps {
	onAccountAdded?: () => void
	children?: React.ReactNode
}

const ProviderOption = ({
	title, icon: Icon, onClick, isLoading, disabled, brandColor,
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
		<div className={cn('absolute inset-0 bg-gradient-to-r opacity-0 transition-opacity group-hover:opacity-5', brandColor)} />
		<div className='flex items-center gap-4'>
			<div className={cn('flex h-10 w-10 items-center justify-center rounded-lg bg-slate-950/50 ring-1 ring-white/10', brandColor.replace('from-', 'text-'))}>
				<Icon className='h-5 w-5' />
			</div>
			<h3 className='font-medium text-slate-100'>{title}</h3>
		</div>
		{isLoading
			? <Loader2 className='h-5 w-5 animate-spin text-slate-400' />
			: <ArrowRight className='h-5 w-5 -translate-x-2 text-slate-500 opacity-0 transition-all group-hover:translate-x-0 group-hover:opacity-100' />}
	</button>
)

export function AddAccountDialog({ children, onAccountAdded }: AddAccountDialogProps) {
	const { t } = useAccountsTranslation()
	const fetchAccounts = useAccountStore((s) => s.fetchAccounts)
	const [loading, setLoading] = useState<string | null>(null)
	const [open, setOpen] = useState(false)
	const [showManualForm, setShowManualForm] = useState(false)
	const [availableProviders, setAvailableProviders] = useState<string[]>([])
	const transitioning = useRef(false)

	// shellScope  → the outer div we animate height on
	// contentScope → the inner div we fade
	const [shellScope, animateShell] = useAnimate()
	const [contentScope, animateContent] = useAnimate()

	useEffect(() => {
		if (open) {
			invoke<{ providers: string[] }>('get_available_providers')
				.then((res) => setAvailableProviders(res.providers))
				.catch(console.error)
		}
	}, [open])

	const switchTo = async (next: boolean) => {
		if (transitioning.current) return
		transitioning.current = true

		const shell   = shellScope.current   as HTMLDivElement | null
		const content = contentScope.current as HTMLDivElement | null
		if (!shell || !content) { transitioning.current = false; return }

		// 1. Fade out current content
		await animateContent(content, { opacity: 0 }, { duration: 0.15, ease: 'easeInOut' })

		// 2. Lock shell at current pixel height so it doesn't jump
		shell.style.height = shell.offsetHeight + 'px'
		shell.style.overflow = 'hidden'

		// 3. Swap content
		setShowManualForm(next)

		// 4. Wait two frames so the browser has painted the new content
		await new Promise<void>(r => requestAnimationFrame(() => requestAnimationFrame(() => r())))

		// 5. Read the new natural height
		const newH = content.scrollHeight

		// 6. Animate shell height to new value
		await animateShell(
			shell,
			{ height: newH },
			{ duration: 0.36, ease: [0.16, 1, 0.3, 1] }
		)

		// 7. Release fixed height so content can reflow freely (e.g. checkbox expanding)
		shell.style.height = 'auto'
		shell.style.overflow = ''

		// 8. Fade in new content
		await animateContent(content, { opacity: 1 }, { duration: 0.15, ease: 'easeInOut' })

		transitioning.current = false
	}

	const handleProviderClick = async (provider: 'gmail' | 'outlook') => {
		setLoading(provider)
		try {
			const { url } = await invoke<{ url: string; port: number }>('start_oauth_flow', { provider })
			await opener.openUrl(url)
		} catch (e) {
			console.error(e)
			setLoading(null)
		}
	}

	const handleManualSuccess = () => {
		setShowManualForm(false)
		setOpen(false)
		onAccountAdded ? onAccountAdded() : fetchAccounts()
	}

	const handleOpenChange = (newOpen: boolean) => {
		setOpen(newOpen)
		if (!newOpen) {
			setShowManualForm(false)
			setLoading(null)
			transitioning.current = false
			if (shellScope.current) {
				(shellScope.current as HTMLDivElement).style.height = 'auto'
				;(shellScope.current as HTMLDivElement).style.overflow = ''
			}
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

			<DialogContent className='overflow-hidden border-slate-800 bg-slate-900/95 p-0 text-slate-100 backdrop-blur-xl'>
				{/* Shell: we pin its height in px during the resize animation */}
				<div ref={shellScope} className='w-full'>
					{/* Content wrapper: we fade this independently */}
					<div ref={contentScope} className='p-6'>
						<DialogHeader className='mb-2'>
							<DialogTitle className='text-xl font-bold'>{t('settings:accounts.title')}</DialogTitle>
							<DialogDescription className='text-slate-400'>{t('settings:accounts.subtitle')}</DialogDescription>
						</DialogHeader>

						{!showManualForm ? (
							<div className='grid gap-3 py-4'>
								<ProviderOption
									title={t('settings:accounts.providers.gmail.title')}
									icon={Mail}
									brandColor={availableProviders.includes('gmail') ? 'from-red-500 to-orange-500' : 'from-slate-500 to-slate-400'}
									onClick={() => handleProviderClick('gmail')}
									isLoading={loading === 'gmail'}
									disabled={!availableProviders.includes('gmail') || loading !== null}
								/>
								<ProviderOption
									title={t('settings:accounts.providers.outlook.title')}
									icon={Mail}
									brandColor={availableProviders.includes('outlook') ? 'from-blue-500 to-cyan-500' : 'from-slate-500 to-slate-400'}
									onClick={() => handleProviderClick('outlook')}
									isLoading={loading === 'outlook'}
									disabled={!availableProviders.includes('outlook') || loading !== null}
								/>
								<ProviderOption
									title={t('settings:accounts.providers.imap.title')}
									icon={Settings}
									brandColor='from-slate-500 to-slate-400'
									onClick={() => switchTo(true)}
									isLoading={false}
									disabled={loading !== null}
								/>
							</div>
						) : (
							<ManualAccountForm
								onSuccess={handleManualSuccess}
								onCancel={() => switchTo(false)}
							/>
						)}
					</div>
				</div>
			</DialogContent>
		</Dialog>
	)
}