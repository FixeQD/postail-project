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
	oauth: {
		failed: string
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
			starred: string
		}
		folders: {
			sectionTitle: string
			new: string
			newPlaceholder: string
			creating: string
			create: string
			createSuccess: string
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
		loadingMore: string
		moving: string
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
	folderMenu: {
		newSubfolder: string
		rename: string
		hide: string
		show: string
		delete: string
		renamePlaceholder: string
		renameTitle: string
		renameConfirm: string
		subfolderTitle: string
		subfolderPlaceholder: string
		subfolderConfirm: string
		deleteTitle: string
		deleteDescription: string
		deleteConfirm: string
		deleting: string
		cancel: string
		movedTo: string
		renamed: string
		created: string
		hidden: string
		shown: string
		deleted: string
		nestedWarning: string
		ariaOptions: string
	}
	messageView: {
		back: string
		loading: string
		error: string
		errorRetry: string
		notFound: string
		noSubject: string
		deleted: string
		deleteError: string
		markUnreadError: string
		archived: string
		archiveError: string
		moveError: string
		from: string
		to: string
		toLabel: string
		cc: string
		date: string
		attachments: {
			label: string
			one: string
			other: string
			downloadSuccess: string
			downloadError: string
			errorNotFound: string
		}
		downloadAttachment: string
		viewMode: { html: string; plain: string }
		actions: {
			reply: string
			replyAll: string
			forward: string
			delete: string
			markUnread: string
			moveTo: string
			moveToPlaceholder: string
			moveToEmpty: string
			archive: string
			print: string
			star: string
			unstar: string
			viewSource: string
		}
		tags: {
			title: string
			add: string
			addAria: string
			placeholder: string
			applied: string
			more: string
			create: string
			noMatch: string
			empty: string
			formatToast: string
			formatDesc: string
		}
		renderError: { title: string; description: string; fallback: string }
		cspBlocked: { label: string; allow: string }
		loadingExternal: string
		readReceipt: {
			label: string
			send: string
			sending: string
			sent: string
			error: string
			dismiss: string
		}
		noReply: { title: string; description: string; cancel: string; confirm: string }
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
		composing: string
		signatures: string
		tags: string
		filters: string
		shortcuts: string
		templates: string
		about: string
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
	signatures: {
		title: string
		addSignature: string
		noSignatures: string
		name: string
		isDefault: string
		defaultBadge: string
		placeholder: string
		deleteConfirm: {
			title: string
			description: string
			confirm: string
			cancel: string
		}
	}
	templates: {
		title: string
		subtitle: string
		addTemplate: string
		noTemplates: string
		noTemplatesDesc: string
		name: string
		subject: string
		body: string
		placeholder: string
		saveAsTemplate: string
		saveCurrent: string
		templateName: string
		saveSuccess: string
		saveError: string
		applyConfirm: {
			title: string
			description: string
			confirm: string
			cancel: string
		}
		deleteConfirm: {
			title: string
			description: string
			confirm: string
			cancel: string
		}
		gallery: {
			title: string
			search: string
			manage: string
			empty: string
			apply: string
			replaceSubject: string
			replaceSubjectDesc: string
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
	mailboxRoles: {
		title: string
		subtitle: string
		noMailboxes: string
		skip: string
		confirm: string
		saving: string
		dialogTitle: string
		dialogDescription: string
		roles: {
			inbox: string
			sent: string
			drafts: string
			trash: string
			archive: string
			junk: string
			flagged: string
			all: string
			other: string
		}
	}
}

export interface ValidationTranslations {
	compatibilityPanel: {
		title: string
		toggleTooltip: string
		issues: {
			title: string
			none: string
			error_one: string
			error_other: string
			warning_one: string
			warning_other: string
			info_one: string
			info_other: string
		}
		severity: {
			error: string
			warning: string
			info: string
		}
		actions: {
			autoFix: string
			dismiss: string
			checkAgain: string
			close: string
		}
	}
	sendWarning: {
		title: string
		description: string
		confirm: string
		cancel: string
	}
}

export interface TranslationResources {
	common: CommonTranslations
	welcome: WelcomeTranslations
	security: SecurityTranslations
	errors: ErrorTranslations
	inbox: InboxTranslations
	validation: ValidationTranslations
	settings: SettingsTranslations
}

export type TranslationNamespace = keyof TranslationResources
