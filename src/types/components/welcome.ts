export interface SecurityOptions {
	tpm_available: boolean
	tpm_requires_elevation: boolean
	keyring_available: boolean
	argon2_available: boolean
}

export interface AccentColorStepProps {
	onNext: () => void
	onBack: () => void
}

/**
 * Mirrors `postail_security::TpmErrorKind`, serialized as camelCase by serde
 * 'unknown' covers any TPM2 response code we haven't named yet + the Linux pkexec-elevation error
 * types that reuse this same `{ errorType, message }` shape but aren't TPM2 response codes at all
 */
export type TpmUnlockErrorType = 'integrity' | 'policyFail' | 'badAuth' | 'unknown'

export interface TPMUnlockFailedProps {
	error: { message: string; cancelled: boolean; errorType?: TpmUnlockErrorType } | null
	onRetry: () => void
	onUnlock: () => void
	onRecoveryVerified: () => void
}

export interface TPMInitDialogProps {
	open: boolean
	onClose: () => void
	onSuccess: () => void | Promise<void>
	requiresElevation?: boolean
}

export interface RecoveryStepProps {
	onNext: (phrase: string) => void | Promise<void>
	variant?: 'page' | 'embedded'
	encryptionMethod?: 'tpm' | 'argon2'
}
