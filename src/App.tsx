import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { TitleBar } from './components/TitleBar'
import { WelcomeScreen } from './components/Welcome/WelcomeScreen'
import { AccentColorStep } from './components/Welcome/steps/AccentColorStep'
import { DataDirStep } from './components/Welcome/steps/DataDirStep'
import { EncryptionChoice } from './components/Welcome/encryption/EncryptionChoice'
import { Argon2Setup } from './components/Welcome/encryption/Argon2Setup'
import { Argon2Unlock } from './components/Welcome/encryption/Argon2Unlock'
import { RecoveryStep } from './components/Welcome/recovery/RecoveryStep'
import { TPMUnlockFailed } from './components/Welcome/tpm/TPMUnlockFailed'
import { SettingsScreen } from './components/Settings/SettingsScreen'
import { InboxScreen } from './components/Inbox/InboxScreen'
import { OutboxPanel } from './components/Outbox/OutboxPanel'
import { StatusBar } from './components/StatusBar'
import { LockScreen } from './components/LockScreen'
import { Toaster } from './components/ui/custom/Toaster'
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts'
import { useAutoLock } from './hooks/useAutoLock'
import { useNotifications } from './hooks/useNotifications'
import { useSettingsStore } from './stores/settingsStore'
import { useThemeStore } from './stores/themeStore'
import { useAnimationsEnabled } from './hooks/useMotion'
import { useAppInitialization } from './hooks/useAppInitialization'
import { DevTools } from './components/DevTools/DevTools'
import { useAccountStore } from '@/stores/accountStore'
import { MailboxRoleDialog } from './components/Settings/Sections/Account/MailboxRoleDialog'
import icon from './assets/icon.png'
import './i18n'

function App() {
	const loadSettings = useSettingsStore((s) => s.loadSettings)
	const { loadTheme, accentColor, darkMode } = useThemeStore()
	const animationsEnabled = useAnimationsEnabled()
	const { isLocked, unlock, useEncryptionPassword } = useAutoLock()

	const {
		currentState,
		setCurrentState,
		handleAccountAdded,
		handleUnlockSuccess,
		handleSecurityChoice,
		handleRecoveryVerified,
		setTempPassphrase,
		activeAccount,
		tpmUnlockError,
		retryTpmUnlock,
		resetInitialization,
		handleRecoveryPhraseVerified,
		handleRecoveryReencrypt,
		recoveryReencryptSource,
		pendingMailboxRoleAccountId,
		handleMailboxRolesDone,
	} = useAppInitialization()

	useEffect(() => {
		// Apply dark class immediately so there's no flash before loadTheme resolves
		document.documentElement.classList.add('dark')
		loadTheme()
	}, [loadTheme])

	useEffect(() => {
		const ua = navigator.userAgent
		const isWebKitGTK =
			ua.includes('WebKitGTK') ||
			(ua.includes('Linux') && ua.includes('WebKit') && !ua.includes('Chrome'))
		document.documentElement.setAttribute('data-renderer', isWebKitGTK ? 'gtk' : 'webkit')
	}, [])

	useEffect(() => {
		if (darkMode) {
			document.documentElement.classList.add('dark')
			document.documentElement.classList.remove('light')
		} else {
			document.documentElement.classList.remove('dark')
			document.documentElement.classList.add('light')
		}
	}, [darkMode])

	useEffect(() => {
		// only call the DB when it's actually initialized
		const dbReady =
			currentState !== 'init' &&
			currentState !== 'argon2-unlock' &&
			currentState !== 'welcome' &&
			currentState !== 'security' &&
			currentState !== 'argon2-setup' &&
			currentState !== 'tpm-unlock-failed' &&
			!isLocked
		if (dbReady) {
			loadSettings()
		}
	}, [loadSettings, currentState, isLocked])

	useEffect(() => {
		document.documentElement.setAttribute('data-animations', animationsEnabled ? 'on' : 'off')
	}, [animationsEnabled])

	const [outboxOpen, setOutboxOpen] = useState(false)

	useNotifications(
		currentState === 'dashboard' || currentState === 'settings' || currentState === 'accounts'
	)
	useGlobalShortcuts({
		onNewMessage: () => {
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
				// There should be refresh logic :/
			}
		},
		onGoToInbox: () => {
			if (currentState === 'dashboard') {
				useAccountStore.getState().setActiveMailbox('INBOX')
			}
		},
		onGoToOutbox: () => {
			if (activeAccount) setOutboxOpen(true)
		},
		onGoToDrafts: () => {
			if (currentState === 'dashboard') {
				useAccountStore.getState().setActiveMailbox('Drafts')
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

	const handleGetStarted = () => {
		setCurrentState('customize')
	}

	const handleExistingData = () => {
		setCurrentState('data-dir')
	}

	const handleCustomizeDone = () => {
		setCurrentState('security')
	}

	const handleBack = () => {
		if (currentState === 'customize') {
			setCurrentState('welcome')
		} else if (currentState === 'data-dir') {
			setCurrentState('welcome')
		} else if (currentState === 'security') {
			setCurrentState('customize')
		} else if (currentState === 'settings') {
			setCurrentState('dashboard')
		}
	}

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
				return (
					<WelcomeScreen
						onGetStarted={handleGetStarted}
						onExistingData={handleExistingData}
					/>
				)
			case 'data-dir':
				return <DataDirStep onBack={handleBack} onDataDirSet={resetInitialization} />
			case 'customize':
				return <AccentColorStep onNext={handleCustomizeDone} onBack={handleBack} />
			case 'security':
				return (
					<EncryptionChoice onChoiceSelected={handleSecurityChoice} onBack={handleBack} />
				)
			case 'tpm-unlock-failed':
				return (
					<TPMUnlockFailed
						error={tpmUnlockError}
						onRetry={retryTpmUnlock}
						onUnlock={handleUnlockSuccess}
						onRecoveryVerified={handleRecoveryPhraseVerified}
					/>
				)
			case 'reencrypt':
				return (
					<EncryptionChoice
						onChoiceSelected={handleRecoveryReencrypt}
						onBack={() =>
							recoveryReencryptSource && setCurrentState(recoveryReencryptSource)
						}
					/>
				)
			case 'argon2-setup':
				return (
					<Argon2Setup
						onBack={handleBack}
						onComplete={() => {}}
						onInitialize={async (pass) => {
							setTempPassphrase(pass)
							setCurrentState('recovery-setup')
						}}
					/>
				)
			case 'recovery-setup':
				return (
					<RecoveryStep
						onNext={() => handleRecoveryVerified()}
						encryptionMethod='argon2'
					/>
				)
			case 'argon2-unlock':
				return (
					<Argon2Unlock
						onUnlock={handleUnlockSuccess}
						onRecoveryVerified={handleRecoveryPhraseVerified}
					/>
				)
			case 'accounts':
			case 'settings':
				return (
					<SettingsScreen
						onBack={() => setCurrentState('dashboard')}
						canGoBack={currentState === 'settings'}
						showSidebar={currentState === 'settings'}
						onAccountAdded={
							currentState === 'accounts' ? handleAccountAdded : undefined
						}
						onReencrypt={() => {
							handleRecoveryPhraseVerified()
						}}
					/>
				)
			case 'dashboard':
				return (
					<>
						<InboxScreen onOpenSettings={() => setCurrentState('settings')} />
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
			className='noise-overlay text-foreground relative flex h-screen flex-col transition-colors duration-200 ease-out'
			style={{ backgroundColor: 'var(--app-bg, #020617)' }}>
			<TitleBar
				isDashboard={currentState === 'dashboard'}
				onOpenSettings={() => setCurrentState('settings')}
				onSearch={() => {
					/* handle search */
				}}
			/>
			<main className='flex-1 overflow-y-auto'>
				{animationsEnabled ? (
					<AnimatePresence mode='wait'>
						<motion.div
							key={currentState}
							initial={false}
							exit={{ opacity: 0 }}
							transition={{ duration: 0.12, ease: 'easeOut' }}
							className='h-full'>
							{renderCurrentScreen()}
						</motion.div>
					</AnimatePresence>
				) : (
					<div className='h-full'>{renderCurrentScreen()}</div>
				)}
			</main>
			{currentState === 'dashboard' && <StatusBar onOpenOutbox={() => setOutboxOpen(true)} />}
			<MailboxRoleDialog
				accountId={pendingMailboxRoleAccountId}
				onDone={handleMailboxRolesDone}
			/>
			{import.meta.env.DEV && (
				<DevTools currentState={currentState} setCurrentState={setCurrentState} />
			)}
			<Toaster />
			<LockScreen
				isLocked={isLocked}
				onUnlock={unlock}
				useEncryptionPassword={useEncryptionPassword}
			/>
		</div>
	)
}

export default App
