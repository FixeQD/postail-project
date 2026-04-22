import { useTranslation } from 'react-i18next'
import { TranslationNamespace } from '../i18n/types'

export const useTypedTranslation = (ns?: TranslationNamespace | TranslationNamespace[]) => {
	const { t, i18n, ready } = useTranslation(ns)
	return { t, i18n, ready }
}

export const useCommonTranslation = () => {
	return useTypedTranslation('common')
}

export const useWelcomeTranslation = () => {
	return useTypedTranslation(['common', 'welcome'])
}

export const useSecurityTranslation = () => {
	return useTypedTranslation(['common', 'security'])
}

export const useAccountsTranslation = () => {
	return useTypedTranslation(['common', 'settings'])
}

export const useErrorsTranslation = () => {
	return useTypedTranslation(['common', 'errors'])
}

export const useSettingsTranslation = () => {
	return useTypedTranslation(['common', 'settings'])
}

export const useContactsTranslation = () => {
	return useTypedTranslation(['common', 'contacts'])
}
