export interface SettingsScreenProps {
	onBack: () => void
	canGoBack?: boolean
	showSidebar?: boolean
	onAccountAdded?: () => void
}

export interface SettingsNavItemProps {
	section: { id: string; label: string; icon: React.ElementType }
	isActive: boolean
	accentColor: string
	animationsEnabled: boolean
	onClick: () => void
}

export interface AccountsScreenProps {
	accounts: import('../../types/accounts').AccountMeta[]
	onRemoveAccount: (id: string) => void
	onSyncAccount: (id: string) => void
	onAccountAdded?: () => void
}
