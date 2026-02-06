import { useState, useEffect, useCallback } from 'react'
import { listen, Event } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { TitleBar } from './components/TitleBar'
import { WelcomeScreen } from './components/Welcome/WelcomeScreen'
import { EncryptionChoice } from './components/Welcome/EncryptionChoice'
import { Argon2Setup } from './components/Welcome/Argon2Setup'
import { Argon2Unlock } from './components/Welcome/Argon2Unlock'
import { SettingsScreen } from './components/Settings/SettingsScreen'
import { InboxScreen } from './components/Inbox/InboxScreen'
import { OutboxPanel } from './components/Outbox/OutboxPanel'
import { StatusBar } from './components/StatusBar'
import { Toaster, toast } from 'sonner'
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts'
import type { AccountMeta } from './types/accounts'
import { useSettingsStore } from './stores/settingsStore'
import './i18n'
import { useTranslation } from 'react-i18next'

type AppState =
	| 'init'
	| 'welcome'
	| 'security'
	| 'accounts'
	| 'argon2-setup'
	| 'dashboard'
	| 'argon2-unlock'
	| 'settings'

function App() {
	const { t } = useTranslation()
	const [currentState, setCurrentState] = useState<AppState>('init')
	const loadSettings = useSettingsStore((s) => s.loadSettings)

	useEffect(() => {
		loadSettings()
	}, [loadSettings])
	const [accounts, setAccounts] = useState<AccountMeta[]>([])
	const [activeAccount, setActiveAccount] = useState<AccountMeta | null>(null)
	const [activeMailbox, setActiveMailbox] = useState('INBOX')
	const [outboxOpen, setOutboxOpen] = useState(false)

	useGlobalShortcuts({
		onNewMessage: () => {
			console.log('New message shortcut, state:', currentState)
			if (currentState === 'dashboard' && activeAccount) {
				window.dispatchEvent(new CustomEvent('compose:new'))
			}
		},
		onFocusSearch: () => {
			const searchInput = document.querySelector('[data-search-input]') as HTMLElement
			searchInput?.focus()
		},
		onRefresh: () => {
			if (currentState === 'dashboard' && activeAccount) {
				console.log('Refresh shortcut triggered')
				// TODO: Implement refresh/sync
			}
		},
		onGoToInbox: () => {
			if (currentState === 'dashboard') {
				setActiveMailbox('INBOX')
			}
		},
		onGoToOutbox: () => {
			if (currentState === 'dashboard') {
				setOutboxOpen(true)
			}
		},
		onGoToDrafts: () => {
			if (currentState === 'dashboard') {
				setActiveMailbox('Drafts')
			}
		},
		onGoToAccounts: () => {
			setCurrentState('accounts')
		},
		onOpenSettings: () => {
			setCurrentState('settings')
		},
		enabled: currentState === 'dashboard' || currentState === 'accounts',
	})

	const fetchAccounts = useCallback(
		async (options?: { forceShowAccountsOnEmpty?: boolean }) => {
			try {
				const fetchedAccounts = await invoke<AccountMeta[]>('list_accounts')
				setAccounts(fetchedAccounts)

				if (fetchedAccounts.length > 0) {
					setCurrentState('dashboard')
					if (!activeAccount) setActiveAccount(fetchedAccounts[0])
				} else {
					if (
						options?.forceShowAccountsOnEmpty ||
						(currentState !== 'welcome' &&
							currentState !== 'security' &&
							currentState !== 'argon2-setup')
					) {
						setCurrentState('accounts')
					}
				}
			} catch (error) {
				console.error('Failed to fetch accounts:', error)
			}
		},
		[currentState, activeAccount]
	)

	const handleAccountAdded = useCallback(async () => {
		toast.success(t('app.accountAdded', 'Account added. Starting sync...'))
		await fetchAccounts()
	}, [fetchAccounts, t])

	const handleUnlockSuccess = useCallback(async () => {
		await loadSettings()
		await fetchAccounts()
	}, [fetchAccounts, loadSettings])

	useEffect(() => {
		const init = async () => {
			if (currentState !== 'init') return
			try {
				const { status, method } = await invoke<{ status: string; method: string | null }>(
					'get_app_initialization_status'
				)
				if (status === 'Locked') {
					if (method === 'argon2') {
						setCurrentState('argon2-unlock')
					} else if (method === 'tpm' || method === 'keyring') {
						const lastMethod = method as string
						console.log(`Auto-unlocking with ${lastMethod}...`)
						try {
							await invoke('initialize_security', { method: lastMethod })
							await loadSettings()
							await fetchAccounts()
						} catch (e) {
							console.error(`Auto-unlock failed for ${lastMethod}`, e)
							setCurrentState('security')
						}
					} else {
						// Fallback if method unknown/missing but DB exists
						setCurrentState('security')
					}
				} else {
					setCurrentState('welcome')
				}
			} catch (e) {
				console.error('Failed to get initialization status', e)
				setCurrentState('welcome') // Fallback
			}
		}
		init()
	}, [fetchAccounts, currentState])

	useEffect(() => {
		if (accounts.length > 0 && !activeAccount) {
			setActiveAccount(accounts[0])
		}
	}, [accounts, activeAccount])

	const handleGetStarted = () => {
		setCurrentState('security')
	}

	const handleSecurityChoice = async (method: string) => {
		if (method === 'argon2') {
			setCurrentState('argon2-setup')
		} else {
			try {
				console.log(`Initializing ${method} security...`)
				await invoke('initialize_security', { method })
				console.log(`${method} security initialized successfully, switching to accounts`)
				await new Promise((resolve) => setTimeout(resolve, 100))
				await loadSettings()
				await fetchAccounts({ forceShowAccountsOnEmpty: true })
			} catch (error) {
				console.error(`Failed to initialize ${method} security:`, error)
				// Reset to security screen to allow retry
				setCurrentState('security')
			}
		}
	}

	const handleBack = () => {
		if (currentState === 'security') {
			setCurrentState('welcome')
		} else if (currentState === 'accounts') {
			setCurrentState('security')
		} else if (currentState === 'argon2-setup') {
			setCurrentState('security')
		} else if (currentState === 'argon2-unlock') {
			setCurrentState('security')
		} else if (currentState === 'settings') {
			setCurrentState('dashboard')
		}
	}

	const handleRemoveAccount = async (id: string) => {
		try {
			await invoke('remove_account', { id })
			setAccounts((prev) => prev.filter((a) => a.id !== id))
			if (activeAccount?.id === id) {
				setActiveAccount(null) // Efffect will pick next one or UI shows error
			}
		} catch (error) {
			console.error('Failed to remove account:', error)
		}
	}

	const handleSyncAccount = async (id: string) => {
		try {
			await invoke('start_sync', { accountId: id })
		} catch (error) {
			console.error('Failed to sync account:', error)
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
				}
			}
		)

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [handleAccountAdded])

	const renderCurrentScreen = () => {
		switch (currentState) {
			case 'init':
				return (
					<div className='flex h-full items-center justify-center text-slate-500'>
						Loading...
					</div>
				)
			case 'welcome':
				return <WelcomeScreen onGetStarted={handleGetStarted} />
			case 'security':
				return (
					<EncryptionChoice onChoiceSelected={handleSecurityChoice} onBack={handleBack} />
				)
			case 'argon2-setup':
				return (
					<Argon2Setup
						onBack={handleBack}
						onComplete={() => {
							fetchAccounts({ forceShowAccountsOnEmpty: true })
						}}
					/>
				)
			case 'argon2-unlock':
				return <Argon2Unlock onBack={handleBack} onUnlock={handleUnlockSuccess} />
			case 'accounts':
			case 'settings':
				return (
					<SettingsScreen
						accounts={accounts}
						onRemoveAccount={handleRemoveAccount}
						onSyncAccount={handleSyncAccount}
						onBack={handleBack}
					/>
				)
			case 'dashboard':
				return (
					<>
						<InboxScreen
							accounts={accounts}
							activeAccount={activeAccount}
							setActiveAccount={setActiveAccount}
							activeMailbox={activeMailbox}
							setActiveMailbox={setActiveMailbox}
							onOpenSettings={() => setCurrentState('settings')}
						/>
						{outboxOpen && activeAccount && (
							<OutboxPanel
								accountId={activeAccount.id}
								isOpen={outboxOpen}
								onClose={() => setOutboxOpen(false)}
							/>
						)}
					</>
				)
			default:
				return null
		}
	}

	// Only show title bar for suitable screens
	const shouldShowTitleBar = true

	return (
		<div className='flex h-screen flex-col bg-slate-950 text-slate-100'>
			{shouldShowTitleBar && (
				<TitleBar
					isDashboard={currentState === 'dashboard'}
					activeAccount={activeAccount}
					onOpenSettings={() => setCurrentState('settings')}
					onSearch={(q) => console.log('Search:', q)}
					onOpenOutbox={() => setOutboxOpen(true)}
				/>
			)}
			<main className='flex-1 overflow-y-auto'>{renderCurrentScreen()}</main>
			{currentState === 'dashboard' && (
				<StatusBar onOpenOutbox={() => setOutboxOpen(true)} accounts={accounts} />
			)}
			<Toaster />
		</div>
	)
}

export default App
