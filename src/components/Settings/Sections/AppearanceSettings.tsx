import { useState, useRef } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Minimize2, UserCircle, Sparkles, Pipette, Check, Moon, Sun } from 'lucide-react'
import { ColorPicker } from '@/components/ui/custom/ColorPicker'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import {
	useThemeStore,
	ACCENT_PRESETS,
	DARK_BACKGROUND_PRESETS,
	LIGHT_BACKGROUND_PRESETS,
	accentToBackground,
	accentToLightBackground,
} from '@/stores/themeStore'

export function AppearanceSettings() {
	const { t } = useSettingsTranslation()
	const {
		accentColor,
		setAccentColor,
		darkBackgroundId,
		lightBackgroundId,
		setBackgroundId,
		animationsEnabled,
		setAnimationsEnabled,
		darkMode,
		setDarkMode,
		persistTheme,
	} = useThemeStore()
	const [showCustomPicker, setShowCustomPicker] = useState(false)
	const [customColor, setCustomColor] = useState(accentColor)
	const prevDarkMode = useRef(darkMode)

	const backgroundId = darkMode ? darkBackgroundId : lightBackgroundId
	const isPresetSelected = ACCENT_PRESETS.some((p) => p.hex === accentColor)

	const handlePresetClick = (hex: string) => {
		setAccentColor(hex)
		setCustomColor(hex)
		setShowCustomPicker(false)
		persistTheme()
	}

	const handleCustomColorChange = (hex: string) => {
		setCustomColor(hex)
		setAccentColor(hex)
	}

	const handleCustomColorComplete = () => {
		persistTheme()
	}

	const handleBackgroundChange = (id: string) => {
		setBackgroundId(id)
		persistTheme()
	}

	const handleDarkModeToggle = () => {
		prevDarkMode.current = darkMode
		setDarkMode(!darkMode)
		persistTheme()
	}

	// slide direction: going to light → slide right; going to dark → slide left
	const slideDir = darkMode ? -1 : 1

	const fade = (delay = 0) =>
		animationsEnabled
			? {
					initial: { opacity: 0, y: 16 } as const,
					animate: { opacity: 1, y: 0 } as const,
					transition: { delay, duration: 0.4 },
				}
			: {}

	const hover = animationsEnabled ? { whileHover: { scale: 1.15 }, whileTap: { scale: 0.9 } } : {}
	const hoverSmall = animationsEnabled
		? { whileHover: { scale: 1.05 }, whileTap: { scale: 0.95 } }
		: {}

	const activePresets = darkMode ? DARK_BACKGROUND_PRESETS : LIGHT_BACKGROUND_PRESETS
	const autoBg = darkMode ? accentToBackground(accentColor) : accentToLightBackground(accentColor)

	// for light mode tiles, dots/lines need to be dark since bg is bright
	const tileDotsClass = darkMode ? 'bg-white/10' : 'bg-black/10'
	const tileDotsFaintClass = darkMode ? 'bg-white/5' : 'bg-black/5'

	return (
		<div className='mx-auto flex h-full w-full max-w-3xl flex-col space-y-8 overflow-y-auto p-8'>
			<motion.div
				{...(animationsEnabled
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-foreground text-3xl font-bold tracking-tight'>
					{t('settings:appearance.title')}
				</h1>
				<p className='text-muted-foreground mt-1'>{t('settings:appearance.subtitle')}</p>
			</motion.div>

			<div className='space-y-8'>
				{/* Accent Color */}
				<motion.section {...fade(0.05)}>
					<h2 className='text-muted-foreground mb-2 ml-2 text-xs font-bold tracking-widest uppercase'>
						{t('settings:appearance.accentColor.title')}
					</h2>
					<p className='text-tertiary mb-4 ml-2 text-xs'>
						{t('settings:appearance.accentColor.description')}
					</p>

					<div
						className='rounded-2xl border bg-[var(--surface-panel)] p-5'
						style={{ borderColor: 'var(--border-subtle)' }}>
						{/* Preset swatches */}
						<div className='mb-4 flex flex-wrap gap-3'>
							{ACCENT_PRESETS.map((preset) => {
								const isSelected = accentColor === preset.hex
								return (
									<motion.button
										key={preset.id}
										type='button'
										onClick={() => handlePresetClick(preset.hex)}
										{...hover}
										className='group relative flex flex-col items-center gap-1.5'
										title={preset.name}>
										<div
											className='flex h-10 w-10 items-center justify-center rounded-xl shadow-lg transition-all duration-200'
											style={{
												backgroundColor: preset.hex,
												boxShadow: isSelected
													? `0 0 0 2px var(--app-bg, #020617), 0 0 0 4px ${preset.hex}, 0 4px 16px -2px ${preset.hex}40`
													: `0 4px 12px -2px ${preset.hex}30`,
											}}>
											{animationsEnabled ? (
												<AnimatePresence>
													{isSelected && (
														<motion.div
															initial={{ scale: 0 }}
															animate={{ scale: 1 }}
															exit={{ scale: 0 }}
															transition={{
																type: 'spring',
																stiffness: 500,
																damping: 25,
															}}>
															<Check className='text-accent-contrast h-4 w-4 drop-shadow-md' />
														</motion.div>
													)}
												</AnimatePresence>
											) : (
												isSelected && (
													<Check className='text-accent-contrast h-4 w-4 drop-shadow-md' />
												)
											)}
										</div>
										<span
											className={`text-[10px] font-medium transition-colors ${isSelected ? 'text-foreground' : 'text-tertiary'}`}>
											{preset.name}
										</span>
									</motion.button>
								)
							})}

							{/* Custom color trigger */}
							<motion.button
								type='button'
								onClick={() => setShowCustomPicker(!showCustomPicker)}
								{...hover}
								className='group flex flex-col items-center gap-1.5'
								title={t('settings:appearance.accentColor.custom')}>
								<div
									className='flex h-10 w-10 items-center justify-center rounded-xl ring-1 ring-white/[0.12] transition-all duration-200 hover:ring-white/[0.2]'
									style={{
										background: !isPresetSelected
											? accentColor
											: 'conic-gradient(from 0deg, #f43f5e, #f97316, #eab308, #22c55e, #06b6d4, #3b82f6, #a855f7, #f43f5e)',
										boxShadow:
											showCustomPicker || !isPresetSelected
												? `0 0 0 2px var(--app-bg, #020617), 0 0 0 4px ${!isPresetSelected ? accentColor : '#6366f1'}`
												: 'none',
									}}>
									{!isPresetSelected ? (
										<Check className='text-accent-contrast h-4 w-4 drop-shadow-md' />
									) : (
										<Pipette className='text-accent-contrast h-4 w-4 drop-shadow-md' />
									)}
								</div>
								<span
									className={`text-[10px] font-medium ${showCustomPicker || !isPresetSelected ? 'text-foreground' : 'text-tertiary'}`}>
									{t('settings:appearance.accentColor.custom')}
								</span>
							</motion.button>
						</div>

						{/* Custom picker expand */}
						{animationsEnabled ? (
							<AnimatePresence>
								{showCustomPicker && (
									<motion.div
										initial={{ opacity: 0, height: 0 }}
										animate={{ opacity: 1, height: 'auto' }}
										exit={{ opacity: 0, height: 0 }}
										transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
										className='overflow-hidden'>
										{renderCustomPicker()}
									</motion.div>
								)}
							</AnimatePresence>
						) : (
							showCustomPicker && renderCustomPicker()
						)}
					</div>
				</motion.section>

				{/* Background */}
				<motion.section {...fade(0.1)}>
					{/* Header row with dark/light toggle */}
					<div className='mb-2 ml-2 flex items-center justify-between'>
						<div>
							<h2 className='text-muted-foreground text-xs font-bold tracking-widest uppercase'>
								{t('settings:appearance.background.title')}
							</h2>
							<p className='text-tertiary mt-1 text-xs'>
								{t('settings:appearance.background.description')}
							</p>
						</div>

						{/* Dark / Light mode toggle */}
						<motion.button
							type='button'
							onClick={handleDarkModeToggle}
							whileHover={animationsEnabled ? { scale: 1.05 } : {}}
							whileTap={animationsEnabled ? { scale: 0.95 } : {}}
							className='flex shrink-0 items-center gap-2 rounded-full border bg-[var(--surface-panel)] px-3 py-1.5 transition-colors hover:bg-[var(--surface-hover)]'
							style={{ borderColor: 'var(--border-subtle)' }}>
							<div className='relative h-3.5 w-3.5'>
								<AnimatePresence mode='wait'>
									{darkMode ? (
										<motion.div
											key='moon'
											className='absolute inset-0 flex items-center justify-center'
											initial={{ rotate: 90, opacity: 0, scale: 0.5 }}
											animate={{ rotate: 0, opacity: 1, scale: 1 }}
											exit={{ rotate: -90, opacity: 0, scale: 0.5 }}
											transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}>
											<Moon className='text-muted-foreground h-3.5 w-3.5' />
										</motion.div>
									) : (
										<motion.div
											key='sun'
											className='absolute inset-0 flex items-center justify-center'
											initial={{ rotate: -90, opacity: 0, scale: 0.5 }}
											animate={{ rotate: 0, opacity: 1, scale: 1 }}
											exit={{ rotate: 90, opacity: 0, scale: 0.5 }}
											transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}>
											<Sun
												className='h-3.5 w-3.5'
												style={{ color: accentColor }}
											/>
										</motion.div>
									)}
								</AnimatePresence>
							</div>
							<span className='text-muted-foreground text-[11px] font-medium'>
								{darkMode ? 'Dark' : 'Light'}
							</span>
						</motion.button>
					</div>

					<div
						className='overflow-hidden rounded-2xl border bg-[var(--surface-panel)] p-5'
						style={{ borderColor: 'var(--border-subtle)' }}>
						<AnimatePresence mode='wait'>
							<motion.div
								key={darkMode ? 'dark-grid' : 'light-grid'}
								initial={
									animationsEnabled
										? { opacity: 0, x: slideDir * 32 }
										: { opacity: 1, x: 0 }
								}
								animate={{ opacity: 1, x: 0 }}
								exit={
									animationsEnabled
										? { opacity: 0, x: slideDir * -32 }
										: { opacity: 0, x: 0 }
								}
								transition={{ duration: 0.28, ease: [0.16, 1, 0.3, 1] }}
								className='grid grid-cols-4 gap-3 sm:grid-cols-7'>
								{/* Auto tile */}
								<motion.button
									key='auto'
									type='button'
									onClick={() => handleBackgroundChange('auto')}
									{...hoverSmall}
									className='group flex flex-col items-center gap-2'>
									<div
										className='relative flex h-14 w-full items-center justify-center overflow-hidden rounded-xl ring-1 transition-all duration-300'
										style={{
											backgroundColor: autoBg,
											boxShadow:
												backgroundId === 'auto'
													? `0 0 0 2px var(--accent-color, #f97316)`
													: 'none',
											borderColor:
												backgroundId === 'auto'
													? 'transparent'
													: 'rgba(255,255,255,0.08)',
										}}>
										<div
											className='pointer-events-none absolute inset-0 rounded-xl'
											style={{
												background: `radial-gradient(ellipse at 50% 0%, rgba(var(--accent-rgb), 0.18) 0%, transparent 70%)`,
											}}
										/>
										<div className='relative flex items-center gap-1.5'>
											<div
												className='h-2 w-2 rounded-full opacity-70'
												style={{ backgroundColor: accentColor }}
											/>
											<div
												className={`h-1.5 w-6 rounded-full ${tileDotsClass}`}
											/>
											<div
												className={`h-1.5 w-4 rounded-full ${tileDotsFaintClass}`}
											/>
										</div>
										<Sparkles
											className='absolute top-1.5 right-1.5 h-2.5 w-2.5'
											style={{ color: accentColor, opacity: 0.7 }}
										/>
									</div>
									<span
										className={`text-[10px] font-medium transition-colors ${backgroundId === 'auto' ? 'text-foreground' : 'text-tertiary'}`}>
										Auto
									</span>
								</motion.button>

								{activePresets.map((preset) => {
									const isSelected = backgroundId === preset.id
									return (
										<motion.button
											key={preset.id}
											type='button'
											onClick={() => handleBackgroundChange(preset.id)}
											{...hoverSmall}
											className='group flex flex-col items-center gap-2'>
											<div
												className='flex h-14 w-full items-center justify-center rounded-xl ring-1 transition-all duration-200'
												style={{
													backgroundColor: preset.bg,
													boxShadow: isSelected
														? `0 0 0 2px var(--accent-color, #f97316)`
														: 'none',
													borderColor: isSelected
														? 'transparent'
														: 'rgba(255,255,255,0.08)',
												}}>
												<div className='flex items-center gap-1.5'>
													<div
														className='h-2 w-2 rounded-full opacity-60'
														style={{ backgroundColor: accentColor }}
													/>
													<div
														className={`h-1.5 w-6 rounded-full ${tileDotsClass}`}
													/>
													<div
														className={`h-1.5 w-4 rounded-full ${tileDotsFaintClass}`}
													/>
												</div>
											</div>
											<span
												className={`text-[10px] font-medium transition-colors ${isSelected ? 'text-foreground' : 'text-tertiary'}`}>
												{preset.name}
											</span>
										</motion.button>
									)
								})}
							</motion.div>
						</AnimatePresence>
					</div>
				</motion.section>

				{/* Layout */}
				<motion.section {...fade(0.15)}>
					<h2 className='text-muted-foreground mb-4 ml-2 text-xs font-bold tracking-widest uppercase'>
						{t('settings:appearance.layout.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Minimize2}
							label={t('settings:appearance.layout.compactMode.label')}
							description={t('settings:appearance.layout.compactMode.description')}
							value={false}
							onChange={() => {}}
						/>
						<ToggleSetting
							icon={UserCircle}
							label={t('settings:appearance.layout.showAvatars.label')}
							description={t('settings:appearance.layout.showAvatars.description')}
							value={true}
							onChange={() => {}}
						/>
						<ToggleSetting
							icon={Sparkles}
							label={t('settings:appearance.layout.animations.label')}
							description={t('settings:appearance.layout.animations.description')}
							value={animationsEnabled}
							onChange={(val) => {
								setAnimationsEnabled(val)
								persistTheme()
							}}
						/>
					</div>
				</motion.section>
			</div>
		</div>
	)

	function renderCustomPicker() {
		return (
			<div
				className='mt-2 flex items-start gap-6 rounded-xl border bg-[var(--surface-panel)] p-4'
				style={{ borderColor: 'var(--border-subtle)' }}>
				<div className='color-picker-container'>
					<ColorPicker color={customColor} onChange={handleCustomColorChange} />
				</div>
				<div className='flex flex-col gap-3'>
					<label className='text-muted-foreground text-xs font-medium'>Hex</label>
					<div className='flex items-center gap-2'>
						<div
							className='h-8 w-8 rounded-lg shadow-inner ring-1 ring-[var(--border-subtle)]'
							style={{ backgroundColor: customColor }}
						/>
						<input
							type='text'
							value={customColor}
							onChange={(e) => {
								const val = e.target.value
								if (/^#[0-9a-fA-F]{0,6}$/.test(val)) {
									setCustomColor(val)
									if (val.length === 7) setAccentColor(val)
								}
							}}
							onBlur={handleCustomColorComplete}
							className='text-foreground w-24 rounded-lg bg-[var(--surface-panel)] px-3 py-1.5 font-mono text-sm ring-1 ring-[var(--border-subtle)] transition-all focus:ring-[var(--accent-color)] focus:outline-none'
							maxLength={7}
						/>
					</div>

					{/* Mini preview */}
					<div className='mt-2 space-y-2'>
						<p className='text-tertiary text-[10px] font-semibold tracking-wider uppercase'>
							Preview
						</p>
						<button
							type='button'
							className='text-accent-contrast rounded-lg px-4 py-1.5 text-xs font-semibold shadow-md transition-transform hover:scale-105'
							style={{
								background: `linear-gradient(to right, ${customColor}, ${customColor}dd)`,
								boxShadow: `0 4px 12px -2px ${customColor}40`,
							}}>
							Button
						</button>
						<div className='flex items-center gap-2'>
							<div
								className='h-2 w-2 rounded-full'
								style={{ backgroundColor: customColor }}
							/>
							<span className='text-xs font-medium' style={{ color: customColor }}>
								Active text
							</span>
						</div>
					</div>

					<button
						type='button'
						onClick={handleCustomColorComplete}
						className='text-foreground/80 hover:text-foreground mt-2 rounded-lg px-3 py-1.5 text-xs font-medium ring-1 ring-[var(--border-subtle)] transition-colors hover:bg-[var(--surface-hover)]'>
						Apply
					</button>
				</div>
			</div>
		)
	}
}
