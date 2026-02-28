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

export interface TPMUnlockFailedProps {
	error: { message: string; cancelled: boolean } | null
	onRetry: () => void
	onUnlock: () => void
	onRecoveryVerified: () => void
}

export interface TPMInitDialogProps {
	open: boolean
	onClose: () => void
	onSuccess: () => void
	requiresElevation?: boolean
}
