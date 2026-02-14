import { ReactNode } from 'react'
import { invoke } from '@tauri-apps/api/core'
import * as opener from '@tauri-apps/plugin-opener'
import { useAccountsTranslation } from '../../../../hooks/useTypedTranslation'
import { Plus, Settings, ArrowLeft, Loader2 } from 'lucide-react'

// A generic card component for provider selection
const ProviderCard = ({
	title,
	description,
	icon,
	onClick,
	disabled,
	isLoading,
	accentColor,
}: {
	title: string
	description: string
	icon: ReactNode
	onClick: () => void
	disabled: boolean
	isLoading: boolean
	accentColor: 'red' | 'blue' | 'slate'
}) => {
	const colors = {
		red: 'hover:ring-red-500/50',
		blue: 'hover:ring-blue-500/50',
		slate: 'hover:ring-slate-500/50',
	}

	return (
		<button
			type='button'
			onClick={onClick}
			disabled={disabled}
			className={`group relative flex w-full items-center justify-between rounded-xl bg-slate-800/50 p-6 text-left ring-1 ring-slate-700/50 transition-all duration-200 hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-60 ${
				!disabled ? colors[accentColor] : ''
			}`}>
			<div className='flex items-center'>
				<div className='mr-4'>{icon}</div>
				<div>
					<h3 className='font-semibold text-slate-100'>{title}</h3>
					<p className='text-sm text-slate-400'>{description}</p>
				</div>
			</div>
			{isLoading ? (
				<Loader2 className='h-6 w-6 animate-spin text-slate-400' />
			) : (
				<Plus className='h-6 w-6 text-slate-500 transition-colors group-hover:text-slate-300' />
			)}
		</button>
	)
}

const GmailIcon = () => (
	<div className='flex h-12 w-12 items-center justify-center rounded-lg bg-slate-900/50 ring-1 ring-slate-700/50 group-hover:ring-red-500/50'>
		<svg className='h-6 w-6 text-red-400' viewBox='0 0 24 24' fill='currentColor'>
			<path d='M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z' />
			<path d='M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z' />
			<path d='M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z' />
			<path d='M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z' />
		</svg>
	</div>
)

const OutlookIcon = () => (
	<div className='flex h-12 w-12 items-center justify-center rounded-lg bg-slate-900/50 ring-1 ring-slate-700/50 group-hover:ring-blue-500/50'>
		<svg className='h-6 w-6 text-blue-400' viewBox='0 0 24 24' fill='currentColor'>
			<path d='M12 2L2 7v10c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V7l-10-5z' />
		</svg>
	</div>
)

const IMAPIcon = () => (
	<div className='flex h-12 w-12 items-center justify-center rounded-lg bg-slate-900/50 ring-1 ring-slate-700/50 group-hover:ring-slate-500/50'>
		<Settings className='h-6 w-6 text-slate-400' />
	</div>
)

export const AddAccountScreen = ({
	onBack,
	loading,
	setLoading,
}: {
	onBack: () => void
	onAccountAdded: () => void
	loading: string | null
	setLoading: (loading: string | null) => void
}) => {
	const { t } = useAccountsTranslation()

	const handleProviderClick = async (provider: 'gmail' | 'outlook') => {
		setLoading(provider)
		try {
			const { url } = await invoke<{ url: string; port: number }>('start_oauth_flow', {
				provider,
			})
			await opener.openUrl(url)
		} catch (error) {
			console.error(`Failed to start ${provider} OAuth:`, error)
			setLoading(null) // Reset loading state on error
		}
	}

	const handleIMAPClick = () => {
		console.log('IMAP configuration not implemented yet')
	}

	return (
		<div className='flex h-full flex-col'>
			<div className='border-b border-slate-800 bg-slate-900/50 px-4 py-6 backdrop-blur-lg'>
				<div className='container mx-auto'>
					<button
						type='button'
						onClick={onBack}
						className='mb-6 flex items-center gap-2 text-sm text-slate-300 transition-colors hover:text-slate-100'>
						<ArrowLeft className='h-4 w-4' />
						{t('common:actions.back')}
					</button>
					<h1 className='text-4xl font-bold tracking-tight text-slate-100'>
						{t('settings:accounts.title')}
					</h1>
					<p className='mt-2 text-slate-400'>{t('settings:accounts.subtitle')}</p>
				</div>
			</div>

			<div className='container mx-auto flex-1 px-4 py-8'>
				<div className='mx-auto max-w-2xl'>
					<div className='grid gap-4'>
						<ProviderCard
							title={t('settings:accounts.providers.gmail.title')}
							description={t('settings:accounts.providers.gmail.description')}
							icon={<GmailIcon />}
							accentColor='red'
							onClick={() => handleProviderClick('gmail')}
							isLoading={loading === 'gmail'}
							disabled={loading !== null}
						/>
						<ProviderCard
							title={t('settings:accounts.providers.outlook.title')}
							description={t('settings:accounts.providers.outlook.description')}
							icon={<OutlookIcon />}
							accentColor='blue'
							onClick={() => handleProviderClick('outlook')}
							isLoading={loading === 'outlook'}
							disabled={loading !== null}
						/>
						<ProviderCard
							title={t('settings:accounts.providers.imap.title')}
							description={t('settings:accounts.providers.imap.description')}
							icon={<IMAPIcon />}
							accentColor='slate'
							onClick={handleIMAPClick}
							isLoading={false}
							disabled={loading !== null}
						/>
					</div>
				</div>
			</div>
		</div>
	)
}
