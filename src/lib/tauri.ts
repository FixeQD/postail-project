import { invoke } from '@tauri-apps/api/core'

export async function invokeWithErrorLog<T>(
	cmd: string,
	args?: Record<string, unknown>,
	context?: string
): Promise<T | null> {
	try {
		return await invoke<T>(cmd, args)
	} catch (err) {
		console.error(`[${context ?? cmd}] Failed:`, err)
		return null
	}
}
