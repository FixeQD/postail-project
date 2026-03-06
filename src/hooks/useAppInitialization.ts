import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen, Event } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { toast } from '@/components/ui/custom/Toaster'
import { useAccountStore } from '@/stores/accountStore'
import type { AccountMeta } from '@/types/accounts'
import { useSettingsStore } from '@/stores/settingsStore'
import { useThemeStore } from '@/stores/themeStore'
import { useTranslation } from 'react-i18next'

export type AppState =
	| 'init'
	| 'welcome'
	| 'data-dir'
	| 'customize'
	| 'security'
	| 'accounts'
	| 'argon2-setup'
	| 'dashboard'
	| 'argon2-unlock'
	| 'settings'
	| 'recovery-setup'
	| 'tpm-unlock-failed'
	| 'recovery-reencrypt'

export function useAppInitialization() {
	const { t } = useTranslation()
	const [currentState, setCurrentState] = useState<AppState>('init')
	const [tpmUnlockError, setTpmUnlockError] = useState<{
		message: string
		cancelled: boolean
	} | null>(null)
	const loadSettings = useSettingsStore((s) => s.loadSettings)
	const { persistTheme } = useThemeStore()
	const fetchAccountsData = useAccountStore((s) => s.fetchAccounts)
	const activeAccount = useAccountStore((s) => s.activeAccount)

	const [tempPassphrase, setTempPassphrase] = useState<string | null>(null)
	const [isRecoveryReencrypt, setIsRecoveryReencrypt] = useState(false)
	const [showRecoveryVerify, setShowRecoveryVerify] = useState(false)
	const [recoveryReencryptSource, setRecoveryReencryptSource] = useState<AppState | null>(null)
	const [pendingMailboxRoleAccountId, setPendingMailboxRoleAccountId] = useState<string | null>(
		null
	)
	// Guard against double-init from React StrictMode or dependency re-fires
	const isInitializingRef = useRef(false)
	const hasInitializedRef = useRef(false)

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

	const handleRecoveryPhraseVerified = useCallback(
		(source?: AppState) => {
			setRecoveryReencryptSource(source ?? currentState)
			setIsRecoveryReencrypt(true)
			setCurrentState('recovery-reencrypt')
		},
		[currentState]
	)

	// Called by EncryptionChoice when in recovery-reencrypt mode.
	const handleRecoveryReencrypt = async (method: string) => {
		if (method === 'argon2') {
			setIsRecoveryReencrypt(true)
			setCurrentState('argon2-setup')
			return
		}
		try {
			await invoke('change_security_method', { method })
			await persistTheme()
			await loadSettings()
			await fetchAccounts({ forceShowAccountsOnEmpty: true })
		} catch (error) {
			console.error(`Failed to change security method to ${method}:`, error)
		}
	}

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
			if (isRecoveryReencrypt) {
				// Re-store existing master key with new argon2 method (don't reinitialize DB)
				await invoke('change_security_method', {
					method: 'argon2',
					passphrase: tempPassphrase,
				})
				setIsRecoveryReencrypt(false)
			} else {
				await invoke('initialize_security', {
					method: 'argon2',
					passphrase: tempPassphrase,
				})
			}
			setShowRecoveryVerify(false)
			await new Promise((resolve) => setTimeout(resolve, 100))
			await persistTheme()
			await loadSettings()
			await fetchAccounts({ forceShowAccountsOnEmpty: true })
		} catch (error) {
			console.error('Failed to change security method:', error)
			toast.error(
				typeof error === 'string'
					? error
					: t(
							'app.securityChangeFailed',
							'Failed to change security method. Please try again.'
						)
			)
		}
	}

	useEffect(() => {
		const init = async () => {
			if (currentState !== 'init') return
			if (isInitializingRef.current || hasInitializedRef.current) return
			isInitializingRef.current = true
			try {
				const { status, method } = await invoke<{ status: string; method: string | null }>(
					'get_app_initialization_status'
				)
				if (status === 'Locked') {
					if (method === 'argon2') {
						hasInitializedRef.current = true
						setCurrentState('argon2-unlock')
					} else if (method === 'tpm' || method === 'keyring') {
						const lastMethod = method as string
						try {
							await invoke('initialize_security', { method: lastMethod })
							hasInitializedRef.current = true
							await loadSettings()
							await fetchAccounts()
						} catch (e) {
							const errorData = String(e)
							let msg = errorData
							let cancelled = false

							try {
								if (errorData.startsWith('{') && errorData.endsWith('}')) {
									const parsed = JSON.parse(errorData)
									msg = parsed.message || msg
									cancelled = parsed.errorType === 'cancelled'
								}
							} catch (err) {
								console.error('Failed to parse structured error:', err)
							}

							console.error(`Auto-unlock failed for ${lastMethod}`, msg)
							hasInitializedRef.current = true
							setTpmUnlockError({ message: msg, cancelled })
							setCurrentState('tpm-unlock-failed')
						}
					} else {
						hasInitializedRef.current = true
						setCurrentState('security')
					}
				} else {
					hasInitializedRef.current = true
					setCurrentState('welcome')
				}
			} catch (e) {
				console.error('Failed to get initialization status', e)
				hasInitializedRef.current = true
				setCurrentState('welcome')
			} finally {
				isInitializingRef.current = false
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
				const { pendingReauthAccountId, setPendingReauthAccountId, updateAccount } =
					useAccountStore.getState()

				try {
					if (pendingReauthAccountId) {
						const updated = await invoke<AccountMeta>('complete_reauth_flow', {
							accountId: pendingReauthAccountId,
							code: event.payload.code,
							state: event.payload.state,
							codeVerifier: event.payload.code_verifier,
							providerType: event.payload.provider_type,
						})

						updateAccount(updated)
						setPendingReauthAccountId(null)
						toast.success(t('app.reauthSuccess', 'Re-authenticated successfully'))
					} else {
						const newAccount = await invoke<AccountMeta>('complete_oauth_flow', {
							code: event.payload.code,
							state: event.payload.state,
							codeVerifier: event.payload.code_verifier,
							providerType: event.payload.provider_type,
						})

						setPendingMailboxRoleAccountId(newAccount.id)
					}
					await getCurrentWindow().maximize()
				} catch (error) {
					console.error('Failed to complete OAuth flow:', error)
					setPendingReauthAccountId(null)
					toast.error(
						t('errors.oauth.failed', 'Failed to connect account. Please try again.')
					)
				}
			}
		)

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [handleAccountAdded, t])

	const retryTpmUnlock = useCallback(() => {
		setTpmUnlockError(null)
		setIsRecoveryReencrypt(false)
		hasInitializedRef.current = false
		setCurrentState('init')
	}, [])

	const resetInitialization = useCallback(() => {
		hasInitializedRef.current = false
		setCurrentState('init')
	}, [])

	const handleMailboxRolesDone = useCallback(async () => {
		setPendingMailboxRoleAccountId(null)
		await getCurrentWindow().maximize()
		await handleAccountAdded()
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
		tpmUnlockError,
		retryTpmUnlock,
		resetInitialization,
		handleRecoveryPhraseVerified,
		handleRecoveryReencrypt,
		isRecoveryReencrypt,
		setIsRecoveryReencrypt,
		recoveryReencryptSource,
		pendingMailboxRoleAccountId,
		handleMailboxRolesDone,
	}
}
