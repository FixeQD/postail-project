import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'

// Import translation resources
import common from './resources/en/common.json'
import welcome from './resources/en/welcome.json'
import security from './resources/en/security.json'
import accounts from './resources/en/accounts.json'
import errors from './resources/en/errors.json'
import inbox from './resources/en/inbox.json'

const resources = {
	en: {
		common,
		welcome,
		security,
		accounts,
		errors,
        inbox,
	},
}

i18n.use(LanguageDetector)
	.use(initReactI18next)
	.init({
		resources,
		lng: 'en', // Default language
		fallbackLng: 'en',
		debug: import.meta.env.DEV,

		ns: ['common', 'welcome', 'security', 'accounts', 'errors', 'inbox'],
		defaultNS: 'common',

		interpolation: {
			escapeValue: false, // React already escapes
		},

		detection: {
			order: ['localStorage', 'navigator', 'htmlTag'],
			caches: ['localStorage'],
		},
	})

export default i18n
