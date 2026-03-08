import type { AppState } from '@/types/hooks'

export interface DevToolsProps {
	currentState: AppState
	setCurrentState: (state: AppState) => void
}

export type DevToolsSection = 'appstate' | 'reset' | 'stores' | 'commands' | 'settings'

export interface ResetOptions {
	messages: boolean
	emlCache: boolean
	bodyCache: boolean
	attachments: boolean
	contacts: boolean
	settings: boolean
	outbox: boolean
}
