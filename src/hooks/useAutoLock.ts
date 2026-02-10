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
		if (activityTimerRef.current) {
			clearTimeout(activityTimerRef.current)
		}

		activityTimerRef.current = setTimeout(() => {
			invoke('record_lock_activity')
		}, 100)
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

		document.addEventListener('mousedown', handleActivity)
		document.addEventListener('keydown', handleActivity)
		document.addEventListener('scroll', handleActivity)
		document.addEventListener('touchstart', handleActivity)

		return () => {
			document.removeEventListener('mousedown', handleActivity)
			document.removeEventListener('keydown', handleActivity)
			document.removeEventListener('scroll', handleActivity)
			document.removeEventListener('touchstart', handleActivity)
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
