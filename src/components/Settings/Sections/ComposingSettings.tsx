import { motion } from 'framer-motion'
import {
	Mail,
	Save,
	SpellCheck,
	ArrowUpFromLine,
	ArrowDownToLine,
	PenLine,
	Paperclip,
} from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { SettingCard } from '@/components/ui/custom/SettingCard'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useSettingsStore } from '@/stores/settingsStore'
import { useThemeStore } from '@/stores/themeStore'

function SectionTitle({ children }: { children: React.ReactNode }) {
	return (
		<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
			{children}
		</h2>
	)
}

interface OptionCardProps {
	value: string | number
	selected: boolean
	onClick: () => void
	accentColor: string
	icon: React.ReactNode
	label: string
	description: string
}

function OptionCard({ selected, onClick, accentColor, icon, label, description }: OptionCardProps) {
	return (
		<button
			type='button'
			onClick={onClick}
			className='flex w-full items-center gap-4 rounded-2xl border p-4 text-left transition-all duration-200'
			style={{
				borderColor: selected ? `${accentColor}55` : 'var(--border-faint)',
				backgroundColor: selected
					? `rgba(var(--accent-rgb), 0.06)`
					: 'var(--surface-panel)',
			}}>
			<div
				className='flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ring-1 transition-all duration-200'
				style={
					selected
						? {
								backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
								boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.25)`,
							}
						: {
								backgroundColor: 'var(--surface-active)',
								boxShadow: 'inset 0 0 0 1px var(--border-subtle)',
							}
				}>
				<span style={{ color: selected ? accentColor : 'var(--text-secondary)' }}>
					{icon}
				</span>
			</div>
			<div className='flex-1'>
				<p className='text-sm font-semibold text-[var(--text-primary)]'>{label}</p>
				<p className='text-xs text-[var(--text-secondary)]'>{description}</p>
			</div>
			<div
				className='h-4 w-4 shrink-0 rounded-full transition-all duration-200'
				style={{
					backgroundColor: selected ? accentColor : 'transparent',
					boxShadow: selected
						? `0 0 0 2px ${accentColor}`
						: '0 0 0 2px var(--border-subtle)',
				}}
			/>
		</button>
	)
}

interface InlineSelectProps {
	value: string | number
	options: { value: string | number; label: string }[]
	onChange: (v: string | number) => void
	accentColor: string
}

function InlineSelect({ value, options, onChange, accentColor }: InlineSelectProps) {
	return (
		<div className='flex flex-wrap gap-1'>
			{options.map((opt) => (
				<button
					key={String(opt.value)}
					type='button'
					onClick={() => onChange(opt.value)}
					className='rounded-lg px-3 py-1.5 text-xs font-semibold transition-all duration-150'
					style={
						value === opt.value
							? { backgroundColor: accentColor, color: '#fff' }
							: {
									color: 'var(--text-secondary)',
									boxShadow: 'inset 0 0 0 1px var(--border-subtle)',
								}
					}>
					{opt.label}
				</button>
			))}
		</div>
	)
}

const ATTACHMENT_WARN_OPTIONS = [
	{ value: 0, labelKey: 'disabled' },
	{ value: 10, labelKey: '10' },
	{ value: 25, labelKey: '25' },
	{ value: 50, labelKey: '50' },
] as const

export function ComposingSettings() {
	const { t } = useSettingsTranslation()
	const animationsEnabled = useAnimationsEnabled()
	const { settings, setSetting } = useSettingsStore()
	const accentColor = useThemeStore((s) => s.accentColor)

	const fade = (delay = 0) =>
		animationsEnabled
			? {
					initial: { opacity: 0, y: 14 },
					animate: { opacity: 1, y: 0 },
					transition: { delay, duration: 0.35 },
				}
			: {}

	const attachmentWarnOptions = ATTACHMENT_WARN_OPTIONS.map((o) => ({
		value: o.value,
		label: t(`settings:composing.sending.warnLargeAttachment.options.${o.labelKey}`),
	}))

	return (
		<div className='mx-auto flex w-full max-w-3xl flex-col space-y-8 p-8 pb-16'>
			<motion.div
				{...(animationsEnabled
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-[var(--text-primary)]'>
					{t('settings:composing.title')}
				</h1>
				<p className='mt-1 text-[var(--text-secondary)]'>
					{t('settings:composing.subtitle')}
				</p>
			</motion.div>

			<div className='space-y-8'>
				{/* Sending */}
				<motion.section {...fade(0.05)}>
					<SectionTitle>{t('settings:composing.sending.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Mail}
							label={t('settings:composing.sending.readReceipts.label')}
							description={t('settings:composing.sending.readReceipts.description')}
							value={settings['read-receipts-enabled']}
							onChange={(val) => setSetting('read-receipts-enabled', val)}
						/>
						<SettingCard
							icon={Paperclip}
							label={t('settings:composing.sending.warnLargeAttachment.label')}
							description={t(
								'settings:composing.sending.warnLargeAttachment.description'
							)}>
							<InlineSelect
								value={settings['warn-large-attachment-mb']}
								options={attachmentWarnOptions}
								onChange={(v) =>
									setSetting('warn-large-attachment-mb', v as number)
								}
								accentColor={accentColor}
							/>
						</SettingCard>
					</div>
				</motion.section>

				{/* Drafts */}
				<motion.section {...fade(0.08)}>
					<SectionTitle>{t('settings:composing.drafts.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Save}
							label={t('settings:composing.drafts.autoSave.label')}
							description={t('settings:composing.drafts.autoSave.description')}
							value={settings['auto-save-drafts']}
							onChange={(val) => setSetting('auto-save-drafts', val)}
						/>
					</div>
				</motion.section>

				{/* Editor */}
				<motion.section {...fade(0.11)}>
					<SectionTitle>{t('settings:composing.editor.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={SpellCheck}
							label={t('settings:composing.editor.spellCheck.label')}
							description={t('settings:composing.editor.spellCheck.description')}
							value={settings['spell-check']}
							onChange={(val) => setSetting('spell-check', val)}
						/>
						<div className='rounded-2xl border border-[var(--border-faint)] bg-[var(--surface-panel)] p-4 transition-all duration-200 hover:border-[var(--border-subtle)] hover:shadow-lg'>
							<div className='mb-4 flex items-center gap-4'>
								<div
									className='flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ring-1 transition-all duration-300'
									style={{
										backgroundColor: `rgba(var(--accent-rgb), 0.08)`,
										boxShadow: `inset 0 0 0 1px rgba(var(--accent-rgb), 0.2)`,
									}}>
									<PenLine
										className='h-[18px] w-[18px]'
										style={{ color: accentColor }}
									/>
								</div>
								<div className='flex flex-col gap-0.5'>
									<h3 className='text-sm font-semibold text-[var(--text-primary)]'>
										{t('settings:composing.editor.replyPosition.label')}
									</h3>
									<p className='text-xs text-[var(--text-secondary)]'>
										{t('settings:composing.editor.replyPosition.description')}
									</p>
								</div>
							</div>
							<div className='grid grid-cols-2 gap-3'>
								<OptionCard
									value='top'
									selected={settings['default-reply-position'] === 'top'}
									onClick={() => setSetting('default-reply-position', 'top')}
									accentColor={accentColor}
									icon={<ArrowUpFromLine className='h-4 w-4' />}
									label={
										t(
											'settings:composing.editor.replyPosition.options.top'
										).split(' (')[0]
									}
									description={
										t('settings:composing.editor.replyPosition.options.top')
											.split(' (')[1]
											?.replace(')', '') ?? 'Reply above quoted text'
									}
								/>
								<OptionCard
									value='bottom'
									selected={settings['default-reply-position'] === 'bottom'}
									onClick={() => setSetting('default-reply-position', 'bottom')}
									accentColor={accentColor}
									icon={<ArrowDownToLine className='h-4 w-4' />}
									label={
										t(
											'settings:composing.editor.replyPosition.options.bottom'
										).split(' (')[0]
									}
									description={
										t('settings:composing.editor.replyPosition.options.bottom')
											.split(' (')[1]
											?.replace(')', '') ?? 'Reply below quoted text'
									}
								/>
							</div>
						</div>
					</div>
				</motion.section>
			</div>
		</div>
	)
}
