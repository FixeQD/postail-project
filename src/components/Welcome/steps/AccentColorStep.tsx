import { useState, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { ColorPicker } from '@/components/ui/custom/ColorPicker'

import { ArrowLeft, ArrowRight, Palette, Check, Pipette, Sparkles, Moon, Sun } from 'lucide-react'
import {
	useThemeStore,
	ACCENT_PRESETS,
	BACKGROUND_PRESETS,
	LIGHT_BACKGROUND_PRESETS,
	accentToBackground,
	accentToLightBackground,
} from '@/stores/themeStore'
import { useTranslation } from 'react-i18next'
import type { AccentColorStepProps } from '@/types/components/welcome'

export const AccentColorStep = ({ onNext, onBack }: AccentColorStepProps) => {
	const { t } = useTranslation()
	const {
		accentColor,
		setAccentColor,
		darkBackgroundId,
		lightBackgroundId,
		darkMode,
		setDarkMode,
		setBackgroundId,
	} = useThemeStore()
	const backgroundId = darkMode ? darkBackgroundId : lightBackgroundId
	const [showCustomPicker, setShowCustomPicker] = useState(false)
	const [customColor, setCustomColor] = useState(accentColor)

	const handlePresetClick = useCallback(
		(hex: string) => {
			setAccentColor(hex)
			setCustomColor(hex)
			setShowCustomPicker(false)
		},
		[setAccentColor]
	)

	const handleCustomColorChange = useCallback(
		(hex: string) => {
			setCustomColor(hex)
			setAccentColor(hex)
		},
		[setAccentColor]
	)

	const isPresetSelected = ACCENT_PRESETS.some((p) => p.hex === accentColor)

	return (
		<div className='noise-overlay relative flex h-full flex-col'>
			{/* Header */}
			<motion.div
				initial={{ opacity: 0, y: -20, filter: 'blur(8px)' }}
				animate={{ opacity: 1, y: 0, filter: 'blur(0px)' }}
				transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
				className='relative border-b border-black/5 bg-white/10 px-4 py-6 shadow-sm backdrop-blur-[32px] dark:border-white/5 dark:bg-black/20'>
				<div className='pointer-events-none absolute inset-x-0 bottom-0 h-px bg-gradient-to-r from-transparent via-[var(--accent-color)] to-transparent opacity-20' />

				<div className='container mx-auto'>
					<button
						type='button'
						onClick={onBack}
						className='text-muted-foreground hover:text-foreground group mb-6 flex items-center gap-2 text-sm transition-colors'>
						<ArrowLeft className='h-4 w-4 transition-transform group-hover:-translate-x-0.5' />
						{t('common:actions.back')}
					</button>
					<div className='flex items-center gap-3'>
						<div
							className='flex h-10 w-10 items-center justify-center rounded-xl ring-1'
							style={{
								backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
								boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
							}}>
							<Palette className='h-5 w-5' style={{ color: accentColor }} />
						</div>
						<div>
							<h1 className='text-foreground text-3xl font-bold tracking-tight'>
								{t('welcome:customize.title')}
							</h1>
							<p className='text-muted-foreground mt-1 text-sm'>
								{t(
									'welcome:customize.subtitle',
									'Pick your accent color and background'
								)}
							</p>
						</div>
					</div>
				</div>
			</motion.div>

			{/* Content */}
			<div className='container mx-auto flex-1 overflow-y-auto px-4 py-8'>
				<div className='mx-auto max-w-2xl space-y-10'>
					{/* Accent color section */}
					<motion.section
						initial={{ opacity: 0, y: 24, filter: 'blur(4px)' }}
						animate={{ opacity: 1, y: 0, filter: 'blur(0px)' }}
						transition={{ delay: 0.1, duration: 0.6, ease: [0.16, 1, 0.3, 1] }}>
						<h2 className='text-muted-foreground mb-1 text-sm font-bold tracking-widest uppercase'>
							{t('welcome:customize.accentColor')}
						</h2>
						<p className='text-tertiary mb-5 text-xs'>
							{t(
								'welcome:customize.accentColorHint',
								'Used for buttons, highlights, and active states'
							)}
						</p>

						{/* Preset grid */}
						<div className='mb-4 flex flex-wrap gap-3'>
							{ACCENT_PRESETS.map((preset, i) => {
								const isSelected = accentColor === preset.hex
								return (
									<motion.button
										key={preset.id}
										type='button'
										onClick={() => handlePresetClick(preset.hex)}
										initial={{ opacity: 0, scale: 0.8, y: 10 }}
										animate={{ opacity: 1, scale: 1, y: 0 }}
										transition={{
											delay: 0.15 + i * 0.04,
											duration: 0.4,
											ease: [0.16, 1, 0.3, 1],
										}}
										whileHover={{ scale: 1.15, y: -2 }}
										whileTap={{ scale: 0.9 }}
										className='group relative flex flex-col items-center gap-1.5 outline-none'
										title={preset.name}>
										<div
											className='flex h-12 w-12 items-center justify-center rounded-2xl shadow-lg transition-all duration-300 group-focus-visible:ring-2 group-focus-visible:ring-offset-2 group-focus-visible:ring-offset-[var(--app-bg)]'
											style={{
												backgroundColor: preset.hex,
												boxShadow: isSelected
													? `0 0 0 2px var(--app-bg), 0 0 0 4px ${preset.hex}, 0 8px 24px -4px ${preset.hex}66`
													: `0 4px 12px -2px ${preset.hex}40`,
											}}>
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
														<Check className='text-accent-contrast h-5 w-5 drop-shadow-md' />
													</motion.div>
												)}
											</AnimatePresence>
										</div>
										<span
											className={`text-[10px] font-medium transition-colors ${isSelected ? 'text-foreground' : 'text-tertiary'}`}>
											{preset.name}
										</span>
									</motion.button>
								)
							})}

							{/* Custom color button */}
							<motion.button
								type='button'
								onClick={() => setShowCustomPicker(!showCustomPicker)}
								initial={{ opacity: 0, scale: 0.8 }}
								animate={{ opacity: 1, scale: 1 }}
								transition={{
									delay: 0.15 + ACCENT_PRESETS.length * 0.03,
									duration: 0.35,
								}}
								whileHover={{ scale: 1.12 }}
								whileTap={{ scale: 0.92 }}
								className='group flex flex-col items-center gap-1.5'
								title='Custom'>
								<div
									className='flex h-11 w-11 items-center justify-center rounded-xl ring-1 ring-[var(--border-subtle)] transition-all duration-200 hover:ring-[var(--border-subtle)]'
									style={{
										background: !isPresetSelected
											? accentColor
											: 'conic-gradient(from 0deg, #f43f5e, #f97316, #eab308, #22c55e, #06b6d4, #3b82f6, #a855f7, #f43f5e)',
										boxShadow:
											showCustomPicker || !isPresetSelected
												? `0 0 0 2px #020617, 0 0 0 4px ${!isPresetSelected ? accentColor : '#6366f1'}`
												: 'none',
									}}>
									{!isPresetSelected ? (
										<Check className='text-accent-contrast h-5 w-5 drop-shadow-md' />
									) : (
										<Pipette className='text-accent-contrast h-4 w-4 drop-shadow-md' />
									)}
								</div>
								<span
									className={`text-[10px] font-medium ${showCustomPicker || !isPresetSelected ? 'text-foreground' : 'text-tertiary'}`}>
									Custom
								</span>
							</motion.button>
						</div>

						{/* Custom color picker */}
						<AnimatePresence>
							{showCustomPicker && (
								<motion.div
									initial={{ opacity: 0, height: 0 }}
									animate={{ opacity: 1, height: 'auto' }}
									exit={{ opacity: 0, height: 0 }}
									transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
									className='overflow-hidden'>
									<div
										className='flex items-start gap-6 rounded-2xl border bg-[var(--surface-panel)] p-5 shadow-inner backdrop-blur-md'
										style={{ borderColor: 'var(--border-subtle)' }}>
										<ColorPicker
											color={customColor}
											onChange={handleCustomColorChange}
										/>
										<div className='flex flex-col gap-3'>
											<label className='text-muted-foreground text-[10px] font-bold tracking-widest uppercase'>
												Hex
											</label>
											<div className='flex items-center gap-2'>
												<div
													className='h-9 w-9 rounded-lg shadow-lg ring-1 ring-[var(--border-subtle)]'
													style={{ backgroundColor: customColor }}
												/>
												<input
													type='text'
													value={customColor}
													onChange={(e) => {
														const val = e.target.value
														if (/^#[0-9a-fA-F]{0,6}$/.test(val)) {
															setCustomColor(val)
															if (val.length === 7)
																setAccentColor(val)
														}
													}}
													className='text-foreground w-28 rounded-lg bg-[var(--surface-panel)] px-3 py-2 font-mono text-sm ring-1 ring-[var(--border-subtle)] transition-all focus:ring-[var(--accent-color)] focus:outline-none'
													maxLength={7}
												/>
											</div>

											{/* Preview mini elements */}
											<div className='mt-3 space-y-3'>
												<p className='text-muted-foreground text-[10px] font-bold tracking-widest uppercase'>
													Preview
												</p>
												<button
													type='button'
													className='text-accent-contrast rounded-xl px-5 py-2 text-xs font-bold shadow-lg transition-transform hover:scale-105 active:scale-95'
													style={{
														background: `linear-gradient(to right, ${customColor}, ${customColor}dd)`,
														boxShadow: `0 8px 20px -4px ${customColor}40`,
													}}>
													Button
												</button>
												<div className='flex items-center gap-2.5'>
													<div
														className='h-2.5 w-2.5 rounded-full shadow-sm'
														style={{ backgroundColor: customColor }}
													/>
													<span
														className='text-xs font-bold'
														style={{ color: customColor }}>
														Active text
													</span>
												</div>
											</div>
										</div>
									</div>
								</motion.div>
							)}
						</AnimatePresence>
					</motion.section>

					{/* Background section */}
					<motion.section
						initial={{ opacity: 0, y: 20 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.25, duration: 0.5, ease: [0.16, 1, 0.3, 1] }}>
						<div className='mb-5 flex items-start justify-between'>
							<div>
								<h2 className='text-muted-foreground mb-1 text-sm font-bold tracking-widest uppercase'>
									{t('welcome:customize.background')}
								</h2>
								<p className='text-tertiary text-xs'>
									{t(
										'welcome:customize.backgroundHint',
										'Sets the base tone of the interface'
									)}
								</p>
							</div>

							{/* Dark / Light toggle */}
							<motion.button
								type='button'
								onClick={() => setDarkMode(!darkMode)}
								whileHover={{ scale: 1.05 }}
								whileTap={{ scale: 0.95 }}
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
												transition={{
													duration: 0.2,
													ease: [0.16, 1, 0.3, 1],
												}}>
												<Moon className='text-muted-foreground h-3.5 w-3.5' />
											</motion.div>
										) : (
											<motion.div
												key='sun'
												className='absolute inset-0 flex items-center justify-center'
												initial={{ rotate: -90, opacity: 0, scale: 0.5 }}
												animate={{ rotate: 0, opacity: 1, scale: 1 }}
												exit={{ rotate: 90, opacity: 0, scale: 0.5 }}
												transition={{
													duration: 0.2,
													ease: [0.16, 1, 0.3, 1],
												}}>
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

						<div className='-m-1 overflow-hidden p-1'>
							<AnimatePresence mode='wait'>
								<motion.div
									key={darkMode ? 'dark-grid' : 'light-grid'}
									initial={{ opacity: 0, x: darkMode ? -28 : 28 }}
									animate={{ opacity: 1, x: 0 }}
									exit={{ opacity: 0, x: darkMode ? 28 : -28 }}
									transition={{ duration: 0.28, ease: [0.16, 1, 0.3, 1] }}
									className='grid grid-cols-4 gap-3 sm:grid-cols-7'>
									{/* Auto tile */}
									{(() => {
										const autoBg = darkMode
											? accentToBackground(accentColor)
											: accentToLightBackground(accentColor)
										const isSelected = backgroundId === 'auto'
										const dotsClass = darkMode ? 'bg-white/10' : 'bg-black/10'
										const dotsFaintClass = darkMode
											? 'bg-white/5'
											: 'bg-black/5'
										return (
											<motion.button
												key='auto'
												type='button'
												onClick={() => setBackgroundId('auto')}
												whileHover={{ scale: 1.05 }}
												whileTap={{ scale: 0.95 }}
												className='group flex flex-col items-center gap-2'>
												<div
													className='relative flex h-14 w-full items-center justify-center overflow-hidden rounded-xl ring-1 transition-all duration-300'
													style={{
														backgroundColor: autoBg,
														boxShadow: isSelected
															? `0 0 0 2px ${accentColor}`
															: 'none',
														borderColor: isSelected
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
															className={`h-1.5 w-6 rounded-full ${dotsClass}`}
														/>
														<div
															className={`h-1.5 w-4 rounded-full ${dotsFaintClass}`}
														/>
													</div>
													<Sparkles
														className='absolute top-1.5 right-1.5 h-2.5 w-2.5'
														style={{ color: accentColor, opacity: 0.7 }}
													/>
												</div>
												<span
													className={`text-[10px] font-medium transition-colors ${isSelected ? 'text-foreground' : 'text-tertiary'}`}>
													Auto
												</span>
											</motion.button>
										)
									})()}

									{(darkMode ? BACKGROUND_PRESETS : LIGHT_BACKGROUND_PRESETS).map(
										(preset) => {
											const isSelected = backgroundId === preset.id
											const dotsClass = darkMode
												? 'bg-white/10'
												: 'bg-black/10'
											const dotsFaintClass = darkMode
												? 'bg-white/5'
												: 'bg-black/5'
											return (
												<motion.button
													key={preset.id}
													type='button'
													onClick={() => setBackgroundId(preset.id)}
													whileHover={{ scale: 1.05 }}
													whileTap={{ scale: 0.95 }}
													className='group flex flex-col items-center gap-2'>
													<div
														className='flex h-14 w-full items-center justify-center rounded-xl ring-1 transition-all duration-200'
														style={{
															backgroundColor: preset.bg,
															boxShadow: isSelected
																? `0 0 0 2px ${accentColor}`
																: 'none',
															borderColor: isSelected
																? 'transparent'
																: 'rgba(255,255,255,0.08)',
														}}>
														<div className='flex items-center gap-1.5'>
															<div
																className='h-2 w-2 rounded-full opacity-60'
																style={{
																	backgroundColor: accentColor,
																}}
															/>
															<div
																className={`h-1.5 w-6 rounded-full ${dotsClass}`}
															/>
															<div
																className={`h-1.5 w-4 rounded-full ${dotsFaintClass}`}
															/>
														</div>
													</div>
													<span
														className={`text-[10px] font-medium transition-colors ${isSelected ? 'text-foreground' : 'text-tertiary'}`}>
														{preset.name}
													</span>
												</motion.button>
											)
										}
									)}
								</motion.div>
							</AnimatePresence>
						</div>
					</motion.section>

					{/* Live preview card */}
					<motion.section
						initial={{ opacity: 0, y: 20 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.4, duration: 0.5, ease: [0.16, 1, 0.3, 1] }}>
						<h2 className='text-muted-foreground mb-4 text-sm font-bold tracking-widest uppercase'>
							{t('welcome:customize.preview')}
						</h2>
						<div
							className='overflow-hidden rounded-2xl ring-1 ring-[var(--border-subtle)]'
							style={{
								backgroundColor:
									backgroundId === 'auto'
										? darkMode
											? accentToBackground(accentColor)
											: accentToLightBackground(accentColor)
										: [...BACKGROUND_PRESETS, ...LIGHT_BACKGROUND_PRESETS].find(
												(p) => p.id === backgroundId
											)?.bg || '#020617',
							}}>
							{/* Mini titlebar */}
							<div
								className='flex items-center gap-2 border-b px-4 py-2.5'
								style={{ borderColor: 'var(--border-subtle)' }}>
								<div className='flex gap-1.5'>
									<div className='h-2.5 w-2.5 rounded-full bg-red-500/60' />
									<div className='h-2.5 w-2.5 rounded-full bg-amber-500/60' />
									<div className='h-2.5 w-2.5 rounded-full bg-green-500/60' />
								</div>
								<div className='flex-1' />
								<div className='h-5 w-32 rounded-md bg-[var(--surface-panel)]' />
								<div className='flex-1' />
								<div
									className='h-5 w-5 rounded-full'
									style={{
										background: `linear-gradient(135deg, ${accentColor}, ${accentColor}cc)`,
									}}
								/>
							</div>
							{/* Mini content */}
							<div className='flex'>
								{/* Mini sidebar */}
								<div
									className='w-36 border-r p-3'
									style={{ borderColor: 'var(--border-subtle)' }}>
									<div
										className='text-accent-contrast mb-2 rounded-lg px-3 py-1.5 text-[10px] font-semibold'
										style={{
											background: `linear-gradient(to right, ${accentColor}, ${accentColor}dd)`,
										}}>
										New Message
									</div>
									<div className='space-y-1'>
										<div
											className='flex items-center gap-2 rounded-lg px-2 py-1.5'
											style={{
												backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
											}}>
											<div
												className='h-1.5 w-1.5 rounded-full'
												style={{ backgroundColor: accentColor }}
											/>
											<div className='h-2 w-12 rounded bg-white/20' />
										</div>
										<div className='flex items-center gap-2 rounded-lg px-2 py-1.5'>
											<div className='h-1.5 w-1.5 rounded-full bg-white/10' />
											<div className='h-2 w-10 rounded bg-white/8' />
										</div>
										<div className='flex items-center gap-2 rounded-lg px-2 py-1.5'>
											<div className='h-1.5 w-1.5 rounded-full bg-white/10' />
											<div className='h-2 w-8 rounded bg-white/8' />
										</div>
									</div>
								</div>
								{/* Mini message list */}
								<div className='flex-1 p-3'>
									{[1, 2, 3].map((i) => (
										<div
											key={i}
											className='mb-1.5 flex items-center gap-2 rounded-lg border border-white/[0.04] px-3 py-2'>
											<div
												className='h-2 w-2 shrink-0 rounded-full'
												style={{
													backgroundColor:
														i === 1 ? accentColor : 'transparent',
												}}
											/>
											<div className='h-2 w-16 rounded bg-white/15' />
											<div className='h-2 w-24 rounded bg-white/8' />
											<div className='flex-1' />
											<div className='h-2 w-8 rounded bg-white/5' />
										</div>
									))}
								</div>
							</div>
						</div>
					</motion.section>
				</div>
			</div>

			{/* Footer / Continue button */}
			<motion.div
				initial={{ opacity: 0, y: 16 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ delay: 0.5, duration: 0.4 }}
				className='relative border-t border-black/5 bg-white/10 px-4 py-5 backdrop-blur-[32px] dark:border-white/5 dark:bg-black/20'>
				<div className='pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-black/[0.05] to-transparent dark:via-white/[0.06]' />
				<div className='container mx-auto flex justify-end'>
					<motion.button
						type='button'
						onClick={onNext}
						whileHover={{ scale: 1.03, y: -1 }}
						whileTap={{ scale: 0.97 }}
						className='text-accent-contrast flex items-center gap-2.5 rounded-xl px-8 py-3 text-sm font-semibold shadow-lg transition-shadow hover:shadow-xl'
						style={{
							background: `linear-gradient(to right, ${accentColor}, ${accentColor}dd)`,
							boxShadow: `0 8px 24px -4px ${accentColor}30`,
						}}>
						{t('welcome:customize.continue')}
						<ArrowRight className='h-4 w-4' />
					</motion.button>
				</div>
			</motion.div>
		</div>
	)
}
