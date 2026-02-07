export interface CommonTranslations {
	app: {
		name: string
		tagline: string
	}
	actions: {
		start: string
		next: string
		back: string
		cancel: string
		save: string
		continue: string
		retry: string
		close: string
	}
	status: {
		loading: string
		success: string
		error: string
		available: string
		unavailable: string
		recommended: string
	}
}

export interface WelcomeTranslations {
	title: string
	subtitle: string
	description: string
	getStarted: string
}

export interface SecurityTranslations {
	title: string
	subtitle: string
	options: {
		tpm: {
			title: string
			description: string
			status: {
				available: string
				unavailable: string
			}
		}
		keyring: {
			title: string
			description: string
			status: {
				available: string
				unavailable: string
			}
		}
		argon2: {
			title: string
			description: string
			status: {
				available: string
			}
		}
	}
	argon2: {
		title: string
		subtitle: string
		password: string
		confirmPassword: string
		strength: {
			weak: string
			fair: string
			good: string
			strong: string
		}
		requirements: string
		mismatch: string
		setup: string
	}
}

export interface ErrorTranslations {
	network: {
		title: string
		description: string
	}
	auth: {
		title: string
		description: string
	}
	security: {
		title: string
		description: string
	}
	unknown: {
		title: string
		description: string
	}
}

export interface InboxTranslations {
	sidebar: {
		newMessage: string
		mailboxes: {
			inbox: string
			sent: string
			drafts: string
			trash: string
			archive: string
		}
	}
	search: {
		placeholder: string
	}
	messageList: {
		empty: {
			title: string
			subtitle: string
		}
		error: string
		actions: {
			archive: string
			delete: string
			markRead: string
			markUnread: string
		}
		date: {
			yesterday: string
		}
	}
}

export interface SettingsTranslations {
	title: string
	sections: {
		general: string
		accounts: string
		security: string
		appearance: string
		notifications: string
	}
	general: {
		title: string
		subtitle: string
		interface: {
			title: string
			zenMode: {
				label: string
				description: string
			}
		}
		behavior: {
			title: string
			strategicDelay: {
				label: string
				description: string
			}
		}
		security: {
			title: string
			shieldImages: {
				label: string
				description: string
			}
		}
		storage: {
			title: string
			dataNomat: {
				label: string
				description: string
			}
			change: string
			defaultPath: string
		}
	}
	accounts: {
		title: string
		subtitle: string
		providers: {
			gmail: { title: string; description: string; button: string }
			outlook: { title: string; description: string; button: string }
			imap: { title: string; description: string; button: string }
		}
		oauth: {
			title: string
			subtitle: string
			success: string
			error: string
			cancelled: string
			instructions: string
			codePlaceholder: string
			submit: string
		}
		list: {
			title: string
			empty: string
			add_another: string
			add: string
			remove: string
			removeConfirm: string
		}
	}
}

export interface TranslationResources {
	common: CommonTranslations
	welcome: WelcomeTranslations
	security: SecurityTranslations
	errors: ErrorTranslations
	inbox: InboxTranslations
	validation: any
	settings: SettingsTranslations
}

export type TranslationNamespace = keyof TranslationResources
