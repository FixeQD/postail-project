import { useState, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { ColorPicker } from '@/components/ui/custom/ColorPicker'

import { ArrowLeft, ArrowRight, Palette, Check, Pipette, Sparkles } from 'lucide-react'
import {
	useThemeStore,
	ACCENT_PRESETS,
	BACKGROUND_PRESETS,
	accentToBackground,
} from '@/stores/themeStore'
import { useTranslation } from 'react-i18next'
import type { AccentColorStepProps } from '@/types/components/welcome'

export const AccentColorStep = ({ onNext, onBack }: AccentColorStepProps) => {
	const { t } = useTranslation()
	const { accentColor, setAccentColor, backgroundId, setBackgroundId } = useThemeStore()
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
				initial={{ opacity: 0, y: -10 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
				className='relative border-b border-white/[0.06] bg-slate-900/40 px-4 py-6 backdrop-blur-lg'>
				<div className='pointer-events-none absolute inset-x-0 bottom-0 h-px bg-gradient-to-r from-transparent via-[rgba(var(--accent-rgb),0.1)] to-transparent' />

				<div className='container mx-auto'>
					<button
						type='button'
						onClick={onBack}
						className='group mb-6 flex items-center gap-2 text-sm text-slate-400 transition-colors hover:text-slate-100'>
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
							<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
								{t('welcome:customize.title')}
							</h1>
							<p className='mt-1 text-sm text-slate-400'>
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
						initial={{ opacity: 0, y: 20 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.1, duration: 0.5, ease: [0.16, 1, 0.3, 1] }}>
						<h2 className='mb-1 text-sm font-bold tracking-widest text-slate-500 uppercase'>
							{t('welcome:customize.accentColor')}
						</h2>
						<p className='mb-5 text-xs text-slate-600'>
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
										initial={{ opacity: 0, scale: 0.8 }}
										animate={{ opacity: 1, scale: 1 }}
										transition={{
											delay: 0.15 + i * 0.03,
											duration: 0.35,
											ease: [0.16, 1, 0.3, 1],
										}}
										whileHover={{ scale: 1.12 }}
										whileTap={{ scale: 0.92 }}
										className='group relative flex flex-col items-center gap-1.5'
										title={preset.name}>
										<div
											className='flex h-11 w-11 items-center justify-center rounded-xl shadow-lg transition-all duration-200'
											style={{
												backgroundColor: preset.hex,
												boxShadow: isSelected
													? `0 0 0 2px #020617, 0 0 0 4px ${preset.hex}, 0 4px 20px -2px ${preset.hex}40`
													: `0 4px 12px -2px ${preset.hex}30`,
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
											className={`text-[10px] font-medium transition-colors ${isSelected ? 'text-slate-200' : 'text-slate-600'}`}>
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
									className='flex h-11 w-11 items-center justify-center rounded-xl ring-1 ring-white/[0.12] transition-all duration-200 hover:ring-white/[0.2]'
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
									className={`text-[10px] font-medium ${showCustomPicker || !isPresetSelected ? 'text-slate-200' : 'text-slate-600'}`}>
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
									<div className='flex items-start gap-6 rounded-2xl border border-white/[0.08] bg-slate-950/40 p-5 shadow-inner backdrop-blur-md'>
										<ColorPicker
											color={customColor}
											onChange={handleCustomColorChange}
										/>
										<div className='flex flex-col gap-3'>
											<label className='text-[10px] font-bold tracking-widest text-slate-500 uppercase'>
												Hex
											</label>
											<div className='flex items-center gap-2'>
												<div
													className='h-9 w-9 rounded-lg shadow-lg ring-1 ring-white/[0.1]'
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
													className='w-28 rounded-lg bg-slate-900/80 px-3 py-2 font-mono text-sm text-slate-100 ring-1 ring-white/[0.08] transition-all focus:ring-white/[0.2] focus:outline-none'
													maxLength={7}
												/>
											</div>

											{/* Preview mini elements */}
											<div className='mt-3 space-y-3'>
												<p className='text-[10px] font-bold tracking-widest text-slate-500 uppercase'>
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
						<h2 className='mb-1 text-sm font-bold tracking-widest text-slate-500 uppercase'>
							{t('welcome:customize.background')}
						</h2>
						<p className='mb-5 text-xs text-slate-600'>
							{t(
								'welcome:customize.backgroundHint',
								'Sets the base tone of the interface'
							)}
						</p>

						<div className='grid grid-cols-4 gap-3 sm:grid-cols-7'>
							{/* Auto tile */}
							{(() => {
								const autoBg = accentToBackground(accentColor)
								const isSelected = backgroundId === 'auto'
								return (
									<motion.button
										key='auto'
										type='button'
										onClick={() => setBackgroundId('auto')}
										initial={{ opacity: 0, y: 12 }}
										animate={{ opacity: 1, y: 0 }}
										transition={{
											delay: 0.3,
											duration: 0.35,
											ease: [0.16, 1, 0.3, 1],
										}}
										whileHover={{ scale: 1.05 }}
										whileTap={{ scale: 0.95 }}
										className='group flex flex-col items-center gap-2'>
										<div
											className='relative flex h-14 w-full items-center justify-center overflow-hidden rounded-xl ring-1 transition-all duration-200'
											style={{
												backgroundColor: autoBg,
												boxShadow: isSelected
													? `0 0 0 2px ${accentColor}`
													: 'none',
												borderColor: isSelected
													? 'transparent'
													: 'rgba(255,255,255,0.08)',
											}}>
											{/* Accent tint overlay */}
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
												<div className='h-1.5 w-6 rounded-full bg-white/10' />
												<div className='h-1.5 w-4 rounded-full bg-white/5' />
											</div>
											<Sparkles
												className='absolute top-1.5 right-1.5 h-2.5 w-2.5'
												style={{ color: accentColor, opacity: 0.7 }}
											/>
										</div>
										<span
											className={`text-[10px] font-medium transition-colors ${isSelected ? 'text-slate-200' : 'text-slate-600'}`}>
											Auto
										</span>
									</motion.button>
								)
							})()}

							{BACKGROUND_PRESETS.map((preset, i) => {
								const isSelected = backgroundId === preset.id
								return (
									<motion.button
										key={preset.id}
										type='button'
										onClick={() => setBackgroundId(preset.id)}
										initial={{ opacity: 0, y: 12 }}
										animate={{ opacity: 1, y: 0 }}
										transition={{
											delay: 0.34 + i * 0.04,
											duration: 0.35,
											ease: [0.16, 1, 0.3, 1],
										}}
										whileHover={{ scale: 1.05 }}
										whileTap={{ scale: 0.95 }}
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
													style={{
														backgroundColor: accentColor || '#f97316',
													}}
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
					</motion.section>

					{/* Live preview card */}
					<motion.section
						initial={{ opacity: 0, y: 20 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ delay: 0.4, duration: 0.5, ease: [0.16, 1, 0.3, 1] }}>
						<h2 className='mb-4 text-sm font-bold tracking-widest text-slate-500 uppercase'>
							{t('welcome:customize.preview')}
						</h2>
						<div
							className='overflow-hidden rounded-2xl ring-1 ring-white/[0.06]'
							style={{
								backgroundColor:
									backgroundId === 'auto'
										? accentToBackground(accentColor)
										: BACKGROUND_PRESETS.find((p) => p.id === backgroundId)
												?.bg || '#020617',
							}}>
							{/* Mini titlebar */}
							<div className='flex items-center gap-2 border-b border-white/[0.06] px-4 py-2.5'>
								<div className='flex gap-1.5'>
									<div className='h-2.5 w-2.5 rounded-full bg-red-500/60' />
									<div className='h-2.5 w-2.5 rounded-full bg-amber-500/60' />
									<div className='h-2.5 w-2.5 rounded-full bg-green-500/60' />
								</div>
								<div className='flex-1' />
								<div className='h-5 w-32 rounded-md bg-white/[0.04]' />
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
								<div className='w-36 border-r border-white/[0.06] p-3'>
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
				className='relative border-t border-white/[0.06] bg-slate-900/30 px-4 py-5 backdrop-blur-lg'>
				<div className='pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-white/[0.06] to-transparent' />
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
