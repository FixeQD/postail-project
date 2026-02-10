import { useState, useEffect, useCallback } from 'react'
import { listen, Event } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { motion, AnimatePresence } from 'framer-motion'
import { TitleBar } from './components/TitleBar'
import { WelcomeScreen } from './components/Welcome/WelcomeScreen'
import { AccentColorStep } from './components/Welcome/AccentColorStep'
import { EncryptionChoice } from './components/Welcome/EncryptionChoice'
import { Argon2Setup } from './components/Welcome/Argon2Setup'
import { Argon2Unlock } from './components/Welcome/Argon2Unlock'
import { SettingsScreen } from './components/Settings/SettingsScreen'
import { InboxScreen } from './components/Inbox/InboxScreen'
import { OutboxPanel } from './components/Outbox/OutboxPanel'
import { StatusBar } from './components/StatusBar'
import { LockScreen } from './components/LockScreen'
import { Toaster, toast } from 'sonner'
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts'
import { useAutoLock } from './hooks/useAutoLock'
import type { AccountMeta } from './types/accounts'
import { useSettingsStore } from './stores/settingsStore'
import { useThemeStore } from './stores/themeStore'
import { useAnimationsEnabled } from './hooks/useMotion'
import icon from './assets/icon.png'
import './i18n'
import { useTranslation } from 'react-i18next'

type AppState =
	| 'init'
	| 'welcome'
	| 'customize'
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
	const { loadTheme, accentColor, persistTheme } = useThemeStore()
	const animationsEnabled = useAnimationsEnabled()
	const { isLocked, unlock, useEncryptionPassword } = useAutoLock()

	useEffect(() => {
		loadSettings()
		loadTheme()
	}, [loadSettings, loadTheme])

	useEffect(() => {
		document.documentElement.setAttribute('data-animations', animationsEnabled ? 'on' : 'off')
	}, [animationsEnabled])
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
		toast.success(t('app.accountAdded', 'Account added successfully'))
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
		setCurrentState('customize')
	}

	const handleCustomizeDone = () => {
		// Don't persist yet - DB isn't initialized. Theme is in memory + CSS vars.
		// It will be persisted after security init completes.
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
				await persistTheme()
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
		if (currentState === 'customize') {
			setCurrentState('welcome')
		} else if (currentState === 'security') {
			setCurrentState('customize')
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
					<div className='flex h-full flex-col items-center justify-center gap-5'>
						<motion.div
							initial={{ opacity: 0, scale: 0.8 }}
							animate={{ opacity: 1, scale: 1 }}
							transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
							className='animate-subtle-float'>
							<div className='relative flex h-16 w-16 items-center justify-center rounded-2xl bg-slate-800/80 shadow-xl ring-1 ring-white/[0.08]'>
								<img src={icon} alt='Postail' className='h-12 w-12' />
								<div
									className='animate-glow-breathe absolute -inset-3 -z-10 rounded-3xl blur-xl'
									style={{ backgroundColor: `rgba(var(--accent-rgb), 0.1)` }}
								/>
							</div>
						</motion.div>
						<motion.div
							initial={{ opacity: 0, y: 8 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ delay: 0.15, duration: 0.4 }}
							className='flex flex-col items-center gap-2'>
							<div className='relative h-5 w-5'>
								<div
									className='absolute inset-0 animate-spin rounded-full border-2 border-transparent'
									style={{ borderTopColor: accentColor }}
								/>
							</div>
						</motion.div>
					</div>
				)
			case 'welcome':
				return <WelcomeScreen onGetStarted={handleGetStarted} />
			case 'customize':
				return <AccentColorStep onNext={handleCustomizeDone} onBack={handleBack} />
			case 'security':
				return (
					<EncryptionChoice onChoiceSelected={handleSecurityChoice} onBack={handleBack} />
				)
			case 'argon2-setup':
				return (
					<Argon2Setup
						onBack={handleBack}
						onComplete={async () => {
							await persistTheme()
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
						canGoBack={currentState === 'settings'}
						showSidebar={currentState === 'settings'}
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

	return (
		<div
			className='noise-overlay relative flex h-screen flex-col text-slate-100 transition-colors duration-500 ease-in-out'
			style={{ backgroundColor: 'var(--app-bg, #020617)' }}>
			<TitleBar
				isDashboard={currentState === 'dashboard'}
				activeAccount={activeAccount}
				onOpenSettings={() => setCurrentState('settings')}
				onSearch={(q) => console.log('Search:', q)}
				onOpenOutbox={() => setOutboxOpen(true)}
			/>
			<main className='flex-1 overflow-y-auto'>
				{animationsEnabled ? (
					<AnimatePresence mode='wait'>
						<motion.div
							key={currentState}
							initial={{ opacity: 0 }}
							animate={{ opacity: 1 }}
							exit={{ opacity: 0 }}
							transition={{ duration: 0.2, ease: 'easeOut' }}
							className='h-full'>
							{renderCurrentScreen()}
						</motion.div>
					</AnimatePresence>
				) : (
					<div className='h-full'>{renderCurrentScreen()}</div>
				)}
			</main>
			{currentState === 'dashboard' && (
				<StatusBar onOpenOutbox={() => setOutboxOpen(true)} accounts={accounts} />
			)}
			<Toaster
				toastOptions={{
					style: {
						background: 'rgba(15, 23, 42, 0.95)',
						border: '1px solid rgba(255, 255, 255, 0.06)',
						color: '#e2e8f0',
						backdropFilter: 'blur(12px)',
						boxShadow: '0 8px 32px -4px rgba(0, 0, 0, 0.4)',
					},
				}}
			/>
			<LockScreen
				isLocked={isLocked}
				onUnlock={unlock}
				useEncryptionPassword={useEncryptionPassword}
			/>
		</div>
	)
}

export default App
