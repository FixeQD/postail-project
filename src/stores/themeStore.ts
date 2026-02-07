import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface AccentPreset {
	id: string
	name: string
	hex: string
}

export interface BackgroundPreset {
	id: string
	name: string
	bg: string
	class: string
}

export const ACCENT_PRESETS: AccentPreset[] = [
	{ id: 'orange', name: 'Orange', hex: '#f97316' },
	{ id: 'blue', name: 'Blue', hex: '#3b82f6' },
	{ id: 'purple', name: 'Purple', hex: '#a855f7' },
	{ id: 'green', name: 'Green', hex: '#22c55e' },
	{ id: 'rose', name: 'Rose', hex: '#f43f5e' },
	{ id: 'cyan', name: 'Cyan', hex: '#06b6d4' },
	{ id: 'indigo', name: 'Indigo', hex: '#6366f1' },
	{ id: 'amber', name: 'Amber', hex: '#f59e0b' },
	{ id: 'teal', name: 'Teal', hex: '#14b8a6' },
	{ id: 'pink', name: 'Pink', hex: '#ec4899' },
]

export const BACKGROUND_PRESETS: BackgroundPreset[] = [
	{ id: 'slate', name: 'Slate', bg: '#020617', class: 'bg-slate-950' },
	{ id: 'dark', name: 'Pure Dark', bg: '#0a0a0a', class: 'bg-neutral-950' },
	{ id: 'warm', name: 'Warm Dark', bg: '#0c0a09', class: 'bg-stone-950' },
	{ id: 'navy', name: 'Navy', bg: '#020420', class: 'bg-[#020420]' },
	{ id: 'charcoal', name: 'Charcoal', bg: '#111111', class: 'bg-[#111111]' },
	{ id: 'midnight', name: 'Midnight', bg: '#0f172a', class: 'bg-slate-900' },
]

function hexToRgb(hex: string): { r: number; g: number; b: number } {
	const cleaned = hex.replace('#', '')
	return {
		r: parseInt(cleaned.slice(0, 2), 16),
		g: parseInt(cleaned.slice(2, 4), 16),
		b: parseInt(cleaned.slice(4, 6), 16),
	}
}

function rgbToHsl(r: number, g: number, b: number): { h: number; s: number; l: number } {
	r /= 255
	g /= 255
	b /= 255
	const max = Math.max(r, g, b)
	const min = Math.min(r, g, b)
	let h = 0
	let s = 0
	const l = (max + min) / 2

	if (max !== min) {
		const d = max - min
		s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
		switch (max) {
			case r:
				h = ((g - b) / d + (g < b ? 6 : 0)) / 6
				break
			case g:
				h = ((b - r) / d + 2) / 6
				break
			case b:
				h = ((r - g) / d + 4) / 6
				break
		}
	}
	return { h: h * 360, s: s * 100, l: l * 100 }
}

function hslToHex(h: number, s: number, l: number): string {
	s /= 100
	l /= 100
	const a = s * Math.min(l, 1 - l)
	const f = (n: number) => {
		const k = (n + h / 30) % 12
		const color = l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1)
		return Math.round(255 * Math.max(0, Math.min(1, color)))
			.toString(16)
			.padStart(2, '0')
	}
	return `#${f(0)}${f(8)}${f(4)}`
}

function generateShades(hex: string): {
	light: string
	main: string
	dark: string
} {
	const { r, g, b } = hexToRgb(hex)
	const { h, s, l } = rgbToHsl(r, g, b)
	return {
		light: hslToHex(h, Math.min(s + 5, 100), Math.min(l + 10, 85)),
		main: hex,
		dark: hslToHex(h, s, Math.max(l - 10, 15)),
	}
}

// Relative luminance per WCAG 2.0
function relativeLuminance(r: number, g: number, b: number): number {
	const [rs, gs, bs] = [r, g, b].map((c) => {
		c /= 255
		return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4)
	})
	return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs
}

function contrastTextColor(hex: string): string {
	const { r, g, b } = hexToRgb(hex)
	return relativeLuminance(r, g, b) > 0.55 ? '#000000' : '#ffffff'
}

function applyAccentCSSVariables(hex: string) {
	const root = document.documentElement
	const { r, g, b } = hexToRgb(hex)
	const shades = generateShades(hex)
	const lightRgb = hexToRgb(shades.light)
	const darkRgb = hexToRgb(shades.dark)
	const textColor = contrastTextColor(hex)
	const textRgb = hexToRgb(textColor)

	root.style.setProperty('--accent-color', hex)
	root.style.setProperty('--accent-rgb', `${r}, ${g}, ${b}`)
	root.style.setProperty('--accent-light', shades.light)
	root.style.setProperty('--accent-light-rgb', `${lightRgb.r}, ${lightRgb.g}, ${lightRgb.b}`)
	root.style.setProperty('--accent-dark', shades.dark)
	root.style.setProperty('--accent-dark-rgb', `${darkRgb.r}, ${darkRgb.g}, ${darkRgb.b}`)
	root.style.setProperty('--accent-text', textColor)
	root.style.setProperty('--accent-text-rgb', `${textRgb.r}, ${textRgb.g}, ${textRgb.b}`)
}

function applyBackgroundCSSVariables(bgHex: string) {
	const root = document.documentElement
	root.style.setProperty('--app-bg', bgHex)
}

interface ThemeState {
	accentColor: string
	backgroundId: string
	isLoaded: boolean

	setAccentColor: (hex: string) => void
	setBackgroundId: (id: string) => void
	persistTheme: () => Promise<void>
	loadTheme: () => Promise<void>
	applyTheme: () => void
}

const DEFAULT_ACCENT = '#f97316'
const DEFAULT_BG_ID = 'slate'

export const useThemeStore = create<ThemeState>((set, get) => ({
	accentColor: DEFAULT_ACCENT,
	backgroundId: DEFAULT_BG_ID,
	isLoaded: false,

	setAccentColor: (hex: string) => {
		set({ accentColor: hex })
		applyAccentCSSVariables(hex)
	},

	setBackgroundId: (id: string) => {
		const preset = BACKGROUND_PRESETS.find((p) => p.id === id)
		if (!preset) return
		set({ backgroundId: id })
		applyBackgroundCSSVariables(preset.bg)
	},

	persistTheme: async () => {
		const { accentColor, backgroundId } = get()
		try {
			await invoke('set_theme_config', {
				accentColor,
				background: backgroundId,
			})
		} catch (e) {
			console.error('Failed to persist theme:', e)
		}
	},

	loadTheme: async () => {
		try {
			const theme = await invoke<{ accent_color: string; background: string }>(
				'get_theme_config'
			)

			const accent = theme.accent_color || DEFAULT_ACCENT
			const bgId = theme.background || DEFAULT_BG_ID

			set({ accentColor: accent, backgroundId: bgId, isLoaded: true })

			applyAccentCSSVariables(accent)
			const preset = BACKGROUND_PRESETS.find((p) => p.id === bgId)
			if (preset) applyBackgroundCSSVariables(preset.bg)
		} catch {
			set({ isLoaded: true })
			applyAccentCSSVariables(DEFAULT_ACCENT)
			const preset = BACKGROUND_PRESETS.find((p) => p.id === DEFAULT_BG_ID)
			if (preset) applyBackgroundCSSVariables(preset.bg)
		}
	},

	applyTheme: () => {
		const { accentColor, backgroundId } = get()
		applyAccentCSSVariables(accentColor)
		const preset = BACKGROUND_PRESETS.find((p) => p.id === backgroundId)
		if (preset) applyBackgroundCSSVariables(preset.bg)
	},
}))
