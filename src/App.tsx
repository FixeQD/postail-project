import { useState, useEffect, useCallback } from 'react'
import { listen, Event } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { TitleBar } from './components/TitleBar'
import { WelcomeScreen } from './components/Welcome/WelcomeScreen'
import { EncryptionChoice } from './components/Welcome/EncryptionChoice'
import { Argon2Setup } from './components/Welcome/Argon2Setup'
import { UnlockScreen } from './components/Welcome/UnlockScreen'
import { Argon2Unlock } from './components/Welcome/Argon2Unlock'
import { AccountsScreen } from './components/Account/AccountsScreen'
import { InboxScreen } from './components/Inbox/InboxScreen'
import { OutboxPanel } from './components/Outbox/OutboxPanel'
import { StatusBar } from './components/StatusBar'
import { Toaster } from 'sonner'
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts'
import type { AccountMeta } from './types/accounts'
import './i18n'

type AppState =
	| 'init'
	| 'welcome'
	| 'security'
	| 'accounts'
	| 'argon2-setup'
	| 'dashboard'
	| 'unlock'
	| 'argon2-unlock'
	| 'settings'

/**
 * Top-level React component that manages application state, handles lifecycle events and global shortcuts, and renders the main UI screens.
 *
 * Manages initialization, authentication/security flows, account list and selection, mailbox navigation (inbox/outbox/drafts), OAuth callbacks, and UI-level actions (settings, outbox, title bar, status bar, and toasts).
 *
 * @returns The root JSX element for the application UI
 */
function App() {
	const [currentState, setCurrentState] = useState<AppState>('init')
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

	useEffect(() => {
		const init = async () => {
			try {
				const status = await invoke<string>('get_app_initialization_status')
				if (status === 'Locked') {
					setCurrentState('unlock')
				} else {
					setCurrentState('welcome')
				}
			} catch (e) {
				console.error('Failed to get initialization status', e)
				setCurrentState('welcome') // Fallback
			}
		}
		init()
	}, [])

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
				await fetchAccounts({ forceShowAccountsOnEmpty: true })
			} catch (error) {
				console.error(`Failed to initialize ${method} security:`, error)
				// Reset to security screen to allow retry
				setCurrentState('security')
			}
		}
	}

	const handleUnlockSuccess = async () => {
		await fetchAccounts()
	}

	const handleBack = () => {
		if (currentState === 'security') {
			setCurrentState('welcome')
		} else if (currentState === 'accounts') {
			setCurrentState('security')
		} else if (currentState === 'argon2-setup') {
			setCurrentState('security')
		} else if (currentState === 'argon2-unlock') {
			setCurrentState('unlock')
		} else if (currentState === 'settings') {
			setCurrentState('dashboard')
		}
	}

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
		await fetchAccounts()
	}, [fetchAccounts])

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
			case 'unlock':
				return (
					<UnlockScreen
						onChoiceSelected={(method) => {
							if (method === 'argon2') {
								setCurrentState('argon2-unlock')
							}
						}}
						onSuccess={handleUnlockSuccess}
					/>
				)
			case 'argon2-unlock':
				return <Argon2Unlock onBack={handleBack} onUnlock={handleUnlockSuccess} />
			case 'accounts':
			case 'settings':
				return (
					<div className='flex h-full flex-col'>
						{currentState === 'settings' && (
							<div className='border-b border-slate-800 bg-slate-900 p-4'>
								<button
									type='button'
									onClick={handleBack}
									className='text-sm text-slate-400 hover:text-white'>
									&larr; Back to Mail
								</button>
							</div>
						)}
						<AccountsScreen
							accounts={accounts}
							onRemoveAccount={handleRemoveAccount}
							onSyncAccount={handleSyncAccount}
						/>
					</div>
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
			{currentState === 'dashboard' && activeAccount && (
				<StatusBar onOpenOutbox={() => setOutboxOpen(true)} />
			)}
			<Toaster />
		</div>
	)
}

export default App