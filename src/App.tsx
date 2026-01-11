import { useState, useEffect } from 'react'
import { listen, Event } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { TitleBar } from './components/TitleBar'
import { WelcomeScreen } from './components/Welcome/WelcomeScreen'
import { EncryptionChoice } from './components/Welcome/EncryptionChoice'
import { Argon2Setup } from './components/Welcome/Argon2Setup'
import { AddAccountScreen } from './components/Account/AddAccountScreen'
import './i18n'

type AppState = 'welcome' | 'security' | 'accounts' | 'argon2-setup' | 'dashboard'

interface AccountMeta {
	id: string
	name: string
	email: string
	provider_type: string
	auth_type: string
	imap_host: string
	imap_port: number
	imap_tls: boolean
	smtp_host: string
	smtp_port: number
	smtp_tls: boolean
	encryption_mode: string
	created_at: string
}

function App() {
	const [currentState, setCurrentState] = useState<AppState>('welcome')
	const [loading, setLoading] = useState<string | null>(null)
	const [accounts, setAccounts] = useState<AccountMeta[]>([])

	const handleGetStarted = () => {
		setCurrentState('security')
	}

	const handleSecurityChoice = (method: string) => {
		if (method === 'argon2') {
			setCurrentState('argon2-setup')
		} else {
			// For TPM and Keyring, proceed to accounts
			setCurrentState('accounts')
		}
	}

	const handleBack = () => {
		if (currentState === 'security') {
			setCurrentState('welcome')
		} else if (currentState === 'accounts') {
			setCurrentState('security')
		} else if (currentState === 'argon2-setup') {
			setCurrentState('security')
		}
	}

	const handleAccountAdded = async () => {
		try {
			const fetchedAccounts = await invoke<AccountMeta[]>('list_accounts')
			setAccounts(fetchedAccounts)
			setCurrentState('dashboard')
		} catch (error) {
			console.error('Failed to fetch accounts:', error)
		}
	}

	useEffect(() => {
		const unlisten = listen(
			'oauth_callback',
			async (event: Event<{ code: string; state: string }>) => {
				console.log('OAuth callback received:', event.payload)
				try {
					await invoke('complete_oauth_flow', {
						code: event.payload.code,
						state: event.payload.state,
					})

					handleAccountAdded()

					// Maximize window on success
					await getCurrentWindow().maximize()
				} catch (error) {
					console.error('Failed to complete OAuth flow:', error)
					// TODO: Show an error message to the user in the UI
				} finally {
					setLoading(null)
				}
			}
		)

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [])

	// Fetch accounts on app load
	useEffect(() => {
		const fetchAccounts = async () => {
			try {
				const fetchedAccounts = await invoke<AccountMeta[]>('list_accounts')
				setAccounts(fetchedAccounts)
				if (fetchedAccounts.length > 0) {
					setCurrentState('dashboard')
				}
			} catch (error) {
				console.error('Failed to fetch accounts:', error)
			}
		}
		fetchAccounts()
	}, [])

	const renderCurrentScreen = () => {
		switch (currentState) {
			case 'welcome':
				return <WelcomeScreen onGetStarted={handleGetStarted} />
			case 'security':
				return (
					<EncryptionChoice onChoiceSelected={handleSecurityChoice} onBack={handleBack} />
				)
			case 'accounts':
				return (
					<AddAccountScreen
						onBack={handleBack}
						onAccountAdded={handleAccountAdded}
						loading={loading}
						setLoading={setLoading}
					/>
				)
			case 'argon2-setup':
				return (
					<Argon2Setup
						onBack={handleBack}
						onComplete={() => setCurrentState('accounts')}
					/>
				)
			// Temporary dashboard
			case 'dashboard':
				return (
					<div className='flex h-full flex-col p-8'>
						<h1 className='mb-6 text-3xl font-bold text-slate-100'>Your Accounts</h1>
						<div className='grid gap-4'>
							{accounts.map((account) => (
								<div
									key={account.id}
									className='rounded-xl bg-slate-800/50 p-6 ring-1 ring-slate-700/50'>
									<div className='flex items-center justify-between'>
										<div>
											<h3 className='font-semibold text-slate-100'>
												{account.name}
											</h3>
											<p className='text-sm text-slate-400'>
												{account.email}
											</p>
											<p className='text-xs text-slate-500'>
												{account.provider_type} • {account.auth_type}
											</p>
										</div>
										<div className='text-right text-xs text-slate-500'>
											<p>
												IMAP: {account.imap_host}:{account.imap_port}
											</p>
											<p>
												SMTP: {account.smtp_host}:{account.smtp_port}
											</p>
										</div>
									</div>
								</div>
							))}
						</div>
						<button
							type='button'
							onClick={() => setCurrentState('accounts')}
							className='mt-6 rounded-lg bg-slate-700 px-6 py-2 text-slate-100 transition-colors hover:bg-slate-600'>
							Add Another Account
						</button>
					</div>
				)
			default:
				return null
		}
	}

	// Only show title bar for welcome, security, accounts, and dashboard screens
	const shouldShowTitleBar = ['welcome', 'security', 'accounts', 'dashboard'].includes(
		currentState
	)

	return (
		<div className='flex h-screen flex-col bg-slate-900 text-slate-100'>
			{shouldShowTitleBar && <TitleBar />}
			<main className='flex-1 overflow-y-auto'>{renderCurrentScreen()}</main>
		</div>
	)
}

export default App
