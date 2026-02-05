import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'

// Import translation resources
import common from './resources/en/common.json'
import welcome from './resources/en/welcome.json'
import security from './resources/en/security.json'
import errors from './resources/en/errors.json'
import inbox from './resources/en/inbox.json'
import validation from './resources/en/validation.json'
import settings from './resources/en/settings.json'

const resources = {
	en: {
		common,
		welcome,
		security,
		errors,
		inbox,
		validation,
		settings,
	},
}

i18n.use(LanguageDetector)
	.use(initReactI18next)
	.init({
		resources,
		lng: 'en', // Default language
		fallbackLng: 'en',
		debug: import.meta.env.DEV,

		ns: ['common', 'welcome', 'security', 'errors', 'inbox', 'validation', 'settings'],
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
