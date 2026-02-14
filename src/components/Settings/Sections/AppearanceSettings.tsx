import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Moon, Minimize2, UserCircle, Sparkles, Pipette, Check } from 'lucide-react'
import { ColorPicker } from '@/components/ui/custom/ColorPicker'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useThemeStore, ACCENT_PRESETS, BACKGROUND_PRESETS } from '@/stores/themeStore'

export function AppearanceSettings() {
	const { t } = useSettingsTranslation()
	const {
		accentColor,
		setAccentColor,
		backgroundId,
		setBackgroundId,
		animationsEnabled,
		setAnimationsEnabled,
		persistTheme,
	} = useThemeStore()
	const [showCustomPicker, setShowCustomPicker] = useState(false)
	const [customColor, setCustomColor] = useState(accentColor)

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

	return (
		<div className='mx-auto flex h-full w-full max-w-3xl flex-col space-y-8 overflow-y-auto p-8'>
			<motion.div
				{...(animationsEnabled
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
					{t('settings:appearance.title')}
				</h1>
				<p className='mt-1 text-slate-400'>{t('settings:appearance.subtitle')}</p>
			</motion.div>

			<div className='space-y-8'>
				{/* Accent Color */}
				<motion.section {...fade(0.05)}>
					<h2 className='mb-2 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:appearance.accentColor.title')}
					</h2>
					<p className='mb-4 ml-2 text-xs text-slate-600'>
						{t('settings:appearance.accentColor.description')}
					</p>

					<div className='rounded-2xl border border-white/[0.05] bg-white/[0.03] p-5'>
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
											className={`text-[10px] font-medium transition-colors ${isSelected ? 'text-slate-200' : 'text-slate-600'}`}>
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
									className={`text-[10px] font-medium ${showCustomPicker || !isPresetSelected ? 'text-slate-200' : 'text-slate-600'}`}>
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
					<h2 className='mb-2 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:appearance.background.title')}
					</h2>
					<p className='mb-4 ml-2 text-xs text-slate-600'>
						{t('settings:appearance.background.description')}
					</p>

					<div className='rounded-2xl border border-white/[0.05] bg-white/[0.03] p-5'>
						<div className='grid grid-cols-3 gap-3 sm:grid-cols-6'>
							{BACKGROUND_PRESETS.map((preset) => {
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
												<div className='h-1.5 w-6 rounded-full bg-white/10' />
												<div className='h-1.5 w-4 rounded-full bg-white/5' />
											</div>
										</div>
										<span
											className={`text-[10px] font-medium transition-colors ${isSelected ? 'text-slate-200' : 'text-slate-600'}`}>
											{preset.name}
										</span>
									</motion.button>
								)
							})}
						</div>
					</div>
				</motion.section>

				{/* Theme */}
				<motion.section {...fade(0.15)}>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
						{t('settings:appearance.theme.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Moon}
							label={t('settings:appearance.theme.darkMode.label')}
							description={t('settings:appearance.theme.darkMode.description')}
							value={true}
							onChange={() => {}}
						/>
					</div>
				</motion.section>

				{/* Layout */}
				<motion.section {...fade(0.2)}>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
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
			<div className='mt-2 flex items-start gap-6 rounded-xl border border-white/[0.06] bg-white/[0.02] p-4'>
				<div className='color-picker-container'>
					<ColorPicker color={customColor} onChange={handleCustomColorChange} />
				</div>
				<div className='flex flex-col gap-3'>
					<label className='text-xs font-medium text-slate-400'>Hex</label>
					<div className='flex items-center gap-2'>
						<div
							className='h-8 w-8 rounded-lg shadow-inner ring-1 ring-white/[0.1]'
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
							className='w-24 rounded-lg bg-slate-800/60 px-3 py-1.5 font-mono text-sm text-slate-200 ring-1 ring-white/[0.08] transition-all focus:ring-white/[0.2] focus:outline-none'
							maxLength={7}
						/>
					</div>

					{/* Mini preview */}
					<div className='mt-2 space-y-2'>
						<p className='text-[10px] font-semibold tracking-wider text-slate-600 uppercase'>
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
						className='mt-2 rounded-lg px-3 py-1.5 text-xs font-medium text-slate-300 ring-1 ring-white/[0.1] transition-colors hover:bg-white/[0.06] hover:text-white'>
						Apply
					</button>
				</div>
			</div>
		)
	}
}
