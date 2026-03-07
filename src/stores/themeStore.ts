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

export const DARK_BACKGROUND_PRESETS: BackgroundPreset[] = [
	{ id: 'slate', name: 'Slate', bg: '#020617', class: 'bg-slate-950' },
	{ id: 'pitch', name: 'Pitch Black', bg: '#000000', class: 'bg-black' },
	{ id: 'dark', name: 'Pure Dark', bg: '#0a0a0a', class: 'bg-neutral-950' },
	{ id: 'charcoal', name: 'Charcoal', bg: '#111111', class: 'bg-[#111111]' },
	{ id: 'zinc', name: 'Zinc', bg: '#09090b', class: 'bg-zinc-950' },
	{ id: 'midnight', name: 'Midnight', bg: '#0f172a', class: 'bg-slate-900' },
	{ id: 'navy', name: 'Navy', bg: '#020420', class: 'bg-[#020420]' },
	{ id: 'abyss', name: 'Abyss', bg: '#010409', class: 'bg-[#010409]' },
	{ id: 'warm', name: 'Warm Dark', bg: '#0c0a09', class: 'bg-stone-950' },
	{ id: 'volcano', name: 'Volcano', bg: '#0d0500', class: 'bg-[#0d0500]' },
	{ id: 'forest', name: 'Forest', bg: '#020d05', class: 'bg-[#020d05]' },
	{ id: 'eclipse', name: 'Eclipse', bg: '#0a0514', class: 'bg-[#0a0514]' },
	{ id: 'rosewood', name: 'Rosewood', bg: '#0d0208', class: 'bg-[#0d0208]' },
]

// alias for backward compat
export const BACKGROUND_PRESETS = DARK_BACKGROUND_PRESETS

export const LIGHT_BACKGROUND_PRESETS: BackgroundPreset[] = [
	{ id: 'white', name: 'White', bg: '#ffffff', class: 'bg-white' },
	{ id: 'pearl', name: 'Pearl', bg: '#f8f9fa', class: 'bg-gray-50' },
	{ id: 'cream', name: 'Cream', bg: '#fffdf7', class: 'bg-[#fffdf7]' },
	{ id: 'linen', name: 'Linen', bg: '#faf7f2', class: 'bg-[#faf7f2]' },
	{ id: 'zinc-light', name: 'Zinc', bg: '#f4f4f5', class: 'bg-zinc-100' },
	{ id: 'slate-light', name: 'Slate', bg: '#f1f5f9', class: 'bg-slate-100' },
	{ id: 'sky-light', name: 'Sky', bg: '#f0f9ff', class: 'bg-sky-50' },
	{ id: 'mint-light', name: 'Mint', bg: '#f0fdf4', class: 'bg-green-50' },
	{ id: 'rose-light', name: 'Rose', bg: '#fff1f2', class: 'bg-rose-50' },
	{ id: 'violet-light', name: 'Violet', bg: '#f5f3ff', class: 'bg-violet-50' },
	{ id: 'amber-light', name: 'Amber', bg: '#fffbeb', class: 'bg-amber-50' },
	{ id: 'stone-light', name: 'Stone', bg: '#fafaf9', class: 'bg-stone-50' },
	{ id: 'ocean-light', name: 'Ocean', bg: '#ecfeff', class: 'bg-cyan-50' },
]

export function accentToBackground(accentHex: string): string {
	const { r, g, b } = hexToRgb(accentHex)
	const { h, s } = rgbToHsl(r, g, b)
	return hslToHex(h, Math.min(s * 0.35, 30), 5)
}

export function accentToLightBackground(accentHex: string): string {
	const { r, g, b } = hexToRgb(accentHex)
	const { h, s } = rgbToHsl(r, g, b)
	return hslToHex(h, Math.min(s * 0.15, 12), 97)
}

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

function generateShades(hex: string): { light: string; main: string; dark: string } {
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

function applyBackgroundCSSVariable(bgHex: string) {
	document.documentElement.style.setProperty('--app-bg', bgHex)
}

function applyDarkModeClass(dark: boolean) {
	if (dark) {
		document.documentElement.classList.add('dark')
		document.documentElement.classList.remove('light')
	} else {
		document.documentElement.classList.remove('dark')
		document.documentElement.classList.add('light')
	}
}

function resolveBackground(id: string, darkMode: boolean, accentColor: string): string {
	if (id === 'auto') {
		return darkMode ? accentToBackground(accentColor) : accentToLightBackground(accentColor)
	}
	const presets = darkMode ? DARK_BACKGROUND_PRESETS : LIGHT_BACKGROUND_PRESETS
	return presets.find((p) => p.id === id)?.bg ?? (darkMode ? '#020617' : '#ffffff')
}

interface ThemeState {
	accentColor: string
	darkBackgroundId: string
	lightBackgroundId: string
	darkMode: boolean
	animationsEnabled: boolean
	isLoaded: boolean

	setAccentColor: (hex: string) => void
	setBackgroundId: (id: string) => void
	setDarkMode: (dark: boolean) => void
	setAnimationsEnabled: (enabled: boolean) => void
	persistTheme: () => Promise<void>
	loadTheme: () => Promise<void>
	applyTheme: () => void
}

const DEFAULT_ACCENT = '#f97316'
const DEFAULT_DARK_BG_ID = 'auto'
const DEFAULT_LIGHT_BG_ID = 'auto'

export const useThemeStore = create<ThemeState>((set, get) => ({
	accentColor: DEFAULT_ACCENT,
	darkBackgroundId: DEFAULT_DARK_BG_ID,
	lightBackgroundId: DEFAULT_LIGHT_BG_ID,
	darkMode: true,
	animationsEnabled: true,
	isLoaded: false,

	setAccentColor: (hex: string) => {
		set({ accentColor: hex })
		applyAccentCSSVariables(hex)
		const { darkMode, darkBackgroundId, lightBackgroundId } = get()
		const bgId = darkMode ? darkBackgroundId : lightBackgroundId
		if (bgId === 'auto') {
			applyBackgroundCSSVariable(resolveBackground('auto', darkMode, hex))
		}
	},

	setBackgroundId: (id: string) => {
		const { darkMode, accentColor } = get()
		if (darkMode) {
			set({ darkBackgroundId: id })
		} else {
			set({ lightBackgroundId: id })
		}
		applyBackgroundCSSVariable(resolveBackground(id, darkMode, accentColor))
	},

	setDarkMode: (dark: boolean) => {
		set({ darkMode: dark })
		applyDarkModeClass(dark)
		const { accentColor, darkBackgroundId, lightBackgroundId } = get()
		const bgId = dark ? darkBackgroundId : lightBackgroundId
		applyBackgroundCSSVariable(resolveBackground(bgId, dark, accentColor))
	},

	setAnimationsEnabled: (enabled: boolean) => {
		set({ animationsEnabled: enabled })
	},

	persistTheme: async () => {
		const { accentColor, darkBackgroundId, lightBackgroundId, animationsEnabled, darkMode } =
			get()
		try {
			await invoke('set_theme_config', {
				accentColor,
				background: darkBackgroundId,
				lightBackground: lightBackgroundId,
				animationsEnabled,
				darkMode,
			})
		} catch (e) {
			console.error('Failed to persist theme:', e)
		}
	},

	loadTheme: async () => {
		try {
			const theme = await invoke<{
				accent_color: string
				background: string
				light_background: string
				animations_enabled: boolean
				dark_mode: boolean
			}>('get_theme_config')

			const accent = theme.accent_color || DEFAULT_ACCENT
			const darkBgId = theme.background || DEFAULT_DARK_BG_ID
			const lightBgId = theme.light_background || DEFAULT_LIGHT_BG_ID
			const animations = theme.animations_enabled ?? true
			const dark = theme.dark_mode ?? true

			set({
				accentColor: accent,
				darkBackgroundId: darkBgId,
				lightBackgroundId: lightBgId,
				animationsEnabled: animations,
				darkMode: dark,
				isLoaded: true,
			})

			applyDarkModeClass(dark)
			applyAccentCSSVariables(accent)
			const bgId = dark ? darkBgId : lightBgId
			applyBackgroundCSSVariable(resolveBackground(bgId, dark, accent))
		} catch {
			set({ isLoaded: true })
			applyDarkModeClass(true)
			applyAccentCSSVariables(DEFAULT_ACCENT)
			applyBackgroundCSSVariable(accentToBackground(DEFAULT_ACCENT))
		}
	},

	applyTheme: () => {
		const { accentColor, darkBackgroundId, lightBackgroundId, darkMode } = get()
		applyDarkModeClass(darkMode)
		applyAccentCSSVariables(accentColor)
		const bgId = darkMode ? darkBackgroundId : lightBackgroundId
		applyBackgroundCSSVariable(resolveBackground(bgId, darkMode, accentColor))
	},
}))
