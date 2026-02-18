import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen, Event } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { toast } from '@/components/ui/custom/Toaster'
import { useAccountStore } from '@/stores/accountStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { useThemeStore } from '@/stores/themeStore'
import { useTranslation } from 'react-i18next'

export type AppState =
	| 'init'
	| 'welcome'
	| 'customize'
	| 'security'
	| 'accounts'
	| 'argon2-setup'
	| 'dashboard'
	| 'argon2-unlock'
	| 'settings'
	| 'recovery-setup'

export function useAppInitialization() {
	const { t } = useTranslation()
	const [currentState, setCurrentState] = useState<AppState>('init')
	const loadSettings = useSettingsStore((s) => s.loadSettings)
	const { persistTheme } = useThemeStore()
	const fetchAccountsData = useAccountStore((s) => s.fetchAccounts)
	const activeAccount = useAccountStore((s) => s.activeAccount)

	const [tempPassphrase, setTempPassphrase] = useState<string | null>(null)
	const [showRecoveryVerify, setShowRecoveryVerify] = useState(false)

	const fetchAccounts = useCallback(
		async (options?: { forceShowAccountsOnEmpty?: boolean }) => {
			try {
				const fetched = await fetchAccountsData()

				if (fetched.length > 0) {
					setCurrentState('dashboard')
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
		[currentState, fetchAccountsData]
	)

	const handleAccountAdded = useCallback(async () => {
		toast.success(t('app.accountAdded', 'Account added successfully'))
		await fetchAccounts()
	}, [fetchAccounts, t])

	const handleUnlockSuccess = useCallback(async () => {
		await loadSettings()
		await fetchAccounts()
	}, [fetchAccounts, loadSettings])

	const handleSecurityChoice = async (method: string) => {
		if (method === 'argon2') {
			setCurrentState('argon2-setup')
		} else {
			try {
				await invoke('initialize_security', { method })
				await new Promise((resolve) => setTimeout(resolve, 100))
				await persistTheme()
				await loadSettings()
				await fetchAccounts({ forceShowAccountsOnEmpty: true })
			} catch (error) {
				console.error(`Failed to initialize ${method} security:`, error)
				setCurrentState('security')
			}
		}
	}

	const handleRecoveryVerified = async () => {
		try {
			await invoke('initialize_security', {
				method: 'argon2',
				passphrase: tempPassphrase,
			})
			setShowRecoveryVerify(false)
			await new Promise((resolve) => setTimeout(resolve, 100))
			await persistTheme()
			await fetchAccounts({ forceShowAccountsOnEmpty: true })
		} catch (error) {
			console.error('Failed to initialize Argon2 security:', error)
		}
	}

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
						try {
							await invoke('initialize_security', { method: lastMethod })
							await loadSettings()
							await fetchAccounts()
						} catch (e) {
							console.error(`Auto-unlock failed for ${lastMethod}`, e)
							setCurrentState('security')
						}
					} else {
						setCurrentState('security')
					}
				} else {
					setCurrentState('welcome')
				}
			} catch (e) {
				console.error('Failed to get initialization status', e)
				setCurrentState('welcome')
			}
		}
		init()
	}, [fetchAccounts, currentState, loadSettings])

	useEffect(() => {
		const unlisten = listen(
			'oauth_callback',
			async (
				event: Event<{
					code: string
					state: string
					code_verifier: string
					provider_type: string
				}>
			) => {
				try {
					await invoke('complete_oauth_flow', {
						code: event.payload.code,
						state: event.payload.state,
						codeVerifier: event.payload.code_verifier,
						providerType: event.payload.provider_type,
					})

					handleAccountAdded()
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

	return {
		currentState,
		setCurrentState,
		fetchAccounts,
		handleAccountAdded,
		handleUnlockSuccess,
		handleSecurityChoice,
		handleRecoveryVerified,
		tempPassphrase,
		setTempPassphrase,
		showRecoveryVerify,
		setShowRecoveryVerify,
		activeAccount,
	}
}
