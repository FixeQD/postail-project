import { useState, useEffect } from 'react'
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
import type { AccountMeta } from './types/accounts'
import './i18n'

type AppState = 'init' | 'welcome' | 'security' | 'accounts' | 'argon2-setup' | 'dashboard' | 'unlock' | 'argon2-unlock' | 'settings'

function App() {
	const [currentState, setCurrentState] = useState<AppState>('init')
	const [accounts, setAccounts] = useState<AccountMeta[]>([])

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
                console.error("Failed to get initialization status", e)
                setCurrentState('welcome') // Fallback
            }
        }
        init()
    }, [])

	const handleGetStarted = () => {
		setCurrentState('security')
	}

	const handleSecurityChoice = (method: string) => {
		if (method === 'argon2') {
			setCurrentState('argon2-setup')
		} else {
            // For TPM/Keyring, backend init happens in EncryptionChoice potentially?
            // Actually EncryptionChoice just calls initialize_security. 
            // After init, we should go to accounts to add one if none exist, or dashboard.
            // But usually first run means no accounts.
			setCurrentState('accounts')
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

	const fetchAccounts = async () => {
		try {
			const fetchedAccounts = await invoke<AccountMeta[]>('list_accounts')
			setAccounts(fetchedAccounts)
            // Navigation logic based on accounts presence
			if (fetchedAccounts.length > 0) {
				setCurrentState('dashboard')
			} else {
                if (currentState !== 'welcome' && currentState !== 'security' && currentState !== 'argon2-setup') {
                     setCurrentState('accounts') // Force add account if none exist (after unlock)
                }
            }
		} catch (error) {
			console.error('Failed to fetch accounts:', error)
		}
	}

	const handleAccountAdded = async () => {
		await fetchAccounts()
        // If we were in settings/accounts, go to dashboard
		setCurrentState('dashboard')
	}

	const handleRemoveAccount = async (id: string) => {
		try {
			await invoke('remove_account', { id })
			setAccounts(prev => prev.filter(a => a.id !== id))
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
	}, [])

	const renderCurrentScreen = () => {
		switch (currentState) {
            case 'init':
                return <div className="flex h-full items-center justify-center text-slate-500">Loading...</div>
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
                            fetchAccounts() 
                            // If no accounts, will go to 'accounts' via fetchAccounts logic? 
                            // Actually fetchAccounts updates state.
                            setCurrentState('accounts')
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
                return (
                    <Argon2Unlock 
                        onBack={handleBack}
                        onUnlock={handleUnlockSuccess}
                    />
                )
			case 'accounts':
            case 'settings':
				return (
                    <div className="flex flex-col h-full">
                        {currentState === 'settings' && (
                            <div className="p-4 border-b border-slate-800 bg-slate-900">
                                <button onClick={handleBack} className="text-sm text-slate-400 hover:text-white">
                                    &larr; Back to Mail
                                </button>
                            </div>
                        )}
                        <AccountsScreen 
                            accounts={accounts}
                            onAccountAdded={handleAccountAdded}
                            onRemoveAccount={handleRemoveAccount}
                            onSyncAccount={handleSyncAccount}
                        />
                    </div>
				)
			case 'dashboard':
				return (
					<InboxScreen 
                        accounts={accounts}
                        onOpenSettings={() => setCurrentState('settings')}
                    />
				)
			default:
				return null
		}
	}

	// Only show title bar for suitable screens
	const shouldShowTitleBar = true

	return (
		<div className='flex h-screen flex-col bg-slate-950 text-slate-100'>
			{shouldShowTitleBar && <TitleBar />}
			<main className='flex-1 overflow-y-auto'>{renderCurrentScreen()}</main>
		</div>
	)
}

export default App

