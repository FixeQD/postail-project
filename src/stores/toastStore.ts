import { create } from 'zustand'

export type ToastType = 'success' | 'error' | 'info' | 'warning' | 'loading'

export interface Toast {
	id: string
	message: string
	description?: string
	type: ToastType
	duration?: number
}

interface ToastOptions {
	id?: string
	description?: string
	duration?: number
}

interface ToastStore {
	toasts: Toast[]
	addToast: (message: string, type: ToastType, options?: ToastOptions) => void
	removeToast: (id: string) => void
}

export const useToastStore = create<ToastStore>((set) => ({
	toasts: [],
	addToast: (message, type, options) => {
		const id = options?.id ?? Math.random().toString(36).substring(2, 9)
		const duration = options?.duration ?? 4000
		set((state) => ({
			toasts: [
				...state.toasts.filter((t) => t.id !== id),
				{ id, message, description: options?.description, type, duration },
			],
		}))
		
		if (type !== 'loading') {
			setTimeout(() => {
				set((state) => ({
					toasts: state.toasts.filter((t) => t.id !== id),
				}))
			}, duration)
		}
	},
	removeToast: (id) =>
		set((state) => ({
			toasts: state.toasts.filter((t) => t.id !== id),
		})),
}))

export const toast = {
	success: (msg: string, opts?: ToastOptions) =>
		useToastStore.getState().addToast(msg, 'success', opts),
	error: (msg: string, opts?: ToastOptions) =>
		useToastStore.getState().addToast(msg, 'error', opts),
	info: (msg: string, opts?: ToastOptions) =>
		useToastStore.getState().addToast(msg, 'info', opts),
	warning: (msg: string, opts?: ToastOptions) =>
		useToastStore.getState().addToast(msg, 'warning', opts),
	loading: (msg: string, opts?: ToastOptions) =>
		useToastStore.getState().addToast(msg, 'loading', opts),
}
