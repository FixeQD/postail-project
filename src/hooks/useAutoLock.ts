import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export const useAutoLock = () => {
	const [isLocked, setIsLocked] = useState(false)
	const [useEncryptionPassword, setUseEncryptionPassword] = useState(false)
	const activityTimerRef = useRef<NodeJS.Timeout | null>(null)

	useEffect(() => {
		const checkLockStatus = async () => {
			const locked = await invoke<boolean>('is_app_locked')
			setIsLocked(locked)
		}

		checkLockStatus()

		const unlisten = listen('app:locked', () => {
			setIsLocked(true)
		})

		return () => {
			unlisten.then((fn) => fn())
		}
	}, [])

	const recordActivity = useCallback(() => {
		if (activityTimerRef.current) return

		activityTimerRef.current = setTimeout(() => {
			invoke('record_lock_activity')
			activityTimerRef.current = null
		}, 1000)
	}, [])

	const unlock = useCallback(() => {
		setIsLocked(false)
	}, [])

	useEffect(() => {
		const handleActivity = () => {
			if (!isLocked) {
				recordActivity()
			}
		}

		const options = { passive: true, capture: true }

		document.addEventListener('mousedown', handleActivity, options)
		document.addEventListener('keydown', handleActivity, options)
		document.addEventListener('scroll', handleActivity, options)
		document.addEventListener('touchstart', handleActivity, options)

		return () => {
			document.removeEventListener('mousedown', handleActivity, options)
			document.removeEventListener('keydown', handleActivity, options)
			document.removeEventListener('scroll', handleActivity, options)
			document.removeEventListener('touchstart', handleActivity, options)
		}
	}, [isLocked, recordActivity])

	useEffect(() => {
		const loadSettings = async () => {
			const useEncryption = await invoke<boolean>('is_lock_using_encryption_password')
			setUseEncryptionPassword(useEncryption)
		}

		loadSettings()
	}, [])

	return {
		isLocked,
		unlock,
		useEncryptionPassword,
	}
}
