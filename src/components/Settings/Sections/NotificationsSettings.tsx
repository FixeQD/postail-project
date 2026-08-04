import { type ComponentType } from 'react'
import { motion } from 'framer-motion'
import {
	Bell,
	Volume2,
	Inbox,
	Star,
	Send,
	AlertTriangle,
	User,
	FileText,
	Layers,
	Hash,
	BellOff,
} from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { SettingCard } from '@/components/ui/custom/SettingCard'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useNotificationStore, MIN_COUNT_OPTIONS } from '@/stores/notificationStore'
import { useThemeStore } from '@/stores/themeStore'

// ── Inline select component ────────────────────────────────────────
function SelectSetting({
	icon: Icon,
	label,
	description,
	value,
	options,
	onChange,
	disabled,
}: {
	icon: ComponentType<{ className?: string; style?: React.CSSProperties }>
	label: string
	description: string
	value: number
	options: readonly number[]
	onChange: (v: number) => void
	disabled?: boolean
}) {
	const accentColor = useThemeStore((s) => s.accentColor)
	return (
		<SettingCard
			icon={Icon}
			label={label}
			description={description}
			disabled={disabled}>
			<div className='flex gap-1'>
				{options.map((opt) => (
					<button
						key={opt}
						type='button'
						disabled={disabled}
						onClick={() => !disabled && onChange(opt)}
						className={`min-w-[36px] rounded-lg px-2 py-1.5 text-xs font-semibold transition-all duration-150 ${
							value === opt
								? 'text-white shadow-md'
								: 'text-[var(--text-secondary)] ring-1 ring-[var(--border-subtle)] hover:text-[var(--text-primary)] hover:ring-[var(--border-subtle)]'
						}`}
						style={value === opt ? { backgroundColor: accentColor } : {}}>
						{opt === 1 ? 'All' : `${opt}+`}
					</button>
				))}
			</div>
		</SettingCard>
	)
}

// ── Section heading ────────────────────────────────────────────────
function SectionTitle({ children }: { children: React.ReactNode }) {
	return (
		<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
			{children}
		</h2>
	)
}

// ── Main component ─────────────────────────────────────────────────
export function NotificationsSettings() {
	const { t } = useSettingsTranslation()
	const anim = useAnimationsEnabled()
	const prefs = useNotificationStore((s) => s.prefs)
	const setPref = useNotificationStore((s) => s.setPref)

	const off = !prefs.enabled // master switch off → disable most controls
	const inboxLocked = prefs.inboxOnly // inboxOnly makes importantOnly irrelevant

	const fade = (delay = 0) =>
		anim
			? {
					initial: { opacity: 0, y: 14 },
					animate: { opacity: 1, y: 0 },
					transition: { delay, duration: 0.35 },
				}
			: {}

	return (
		<div className='mx-auto flex w-full max-w-3xl flex-col space-y-8 p-8 pb-16'>
			{/* Header */}
			<motion.div
				{...(anim
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-[var(--text-primary)]'>
					{t('settings:notifications.title')}
				</h1>
				<p className='mt-1 text-[var(--text-secondary)]'>
					{t('settings:notifications.subtitle')}
				</p>
			</motion.div>

			<div className='space-y-8'>
				{/* ── DELIVERY ──────────────────────────────────────────── */}
				<motion.section {...fade(0.05)}>
					<SectionTitle>{t('settings:notifications.delivery.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Bell}
							label={t('settings:notifications.delivery.desktop.label')}
							description={t('settings:notifications.delivery.desktop.description')}
							value={prefs.enabled}
							onChange={(v) => setPref('enabled', v)}
						/>
						<ToggleSetting
							icon={Inbox}
							label={t('settings:notifications.delivery.showInCenter.label')}
							description={t(
								'settings:notifications.delivery.showInCenter.description'
							)}
							value={prefs.showInCenter}
							onChange={(v) => setPref('showInCenter', v)}
						/>
						<ToggleSetting
							icon={Volume2}
							label={t('settings:notifications.delivery.sound.label')}
							description={t('settings:notifications.delivery.sound.description')}
							value={prefs.sound}
							onChange={(v) => setPref('sound', v)}
							disabled={off}
						/>
					</div>
				</motion.section>

				{/* ── FILTERS ───────────────────────────────────────────── */}
				<motion.section {...fade(0.1)}>
					<SectionTitle>{t('settings:notifications.filters.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Inbox}
							label={t('settings:notifications.filters.inboxOnly.label')}
							description={t('settings:notifications.filters.inboxOnly.description')}
							value={prefs.inboxOnly}
							onChange={(v) => setPref('inboxOnly', v)}
							disabled={off && !prefs.showInCenter}
						/>
						<ToggleSetting
							icon={Star}
							label={t('settings:notifications.filters.importantOnly.label')}
							description={t(
								'settings:notifications.filters.importantOnly.description'
							)}
							value={prefs.importantOnly}
							onChange={(v) => setPref('importantOnly', v)}
							disabled={(off && !prefs.showInCenter) || inboxLocked}
						/>
						<ToggleSetting
							icon={Send}
							label={t('settings:notifications.filters.showForSent.label')}
							description={t(
								'settings:notifications.filters.showForSent.description'
							)}
							value={prefs.showForSent}
							onChange={(v) => setPref('showForSent', v)}
							disabled={(off && !prefs.showInCenter) || inboxLocked}
						/>
						<ToggleSetting
							icon={AlertTriangle}
							label={t('settings:notifications.filters.syncErrors.label')}
							description={t('settings:notifications.filters.syncErrors.description')}
							value={prefs.syncErrors}
							onChange={(v) => setPref('syncErrors', v)}
							disabled={!prefs.enabled && !prefs.showInCenter}
						/>
					</div>
				</motion.section>

				{/* ── CONTENT PREVIEW ───────────────────────────────────── */}
				<motion.section {...fade(0.15)}>
					<SectionTitle>{t('settings:notifications.preview.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={User}
							label={t('settings:notifications.preview.previewSender.label')}
							description={t(
								'settings:notifications.preview.previewSender.description'
							)}
							value={prefs.previewSender}
							onChange={(v) => setPref('previewSender', v)}
							disabled={off && !prefs.showInCenter}
						/>
						<ToggleSetting
							icon={FileText}
							label={t('settings:notifications.preview.previewSubject.label')}
							description={t(
								'settings:notifications.preview.previewSubject.description'
							)}
							value={prefs.previewSubject}
							onChange={(v) => setPref('previewSubject', v)}
							disabled={off && !prefs.showInCenter}
						/>
					</div>
				</motion.section>

				{/* ── GROUPING ──────────────────────────────────────────── */}
				<motion.section {...fade(0.2)}>
					<SectionTitle>{t('settings:notifications.grouping.title')}</SectionTitle>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Layers}
							label={t('settings:notifications.grouping.bundle.label')}
							description={t('settings:notifications.grouping.bundle.description')}
							value={prefs.bundleMultiple}
							onChange={(v) => setPref('bundleMultiple', v)}
							disabled={off}
						/>
						<SelectSetting
							icon={Hash}
							label={t('settings:notifications.grouping.minCount.label')}
							description={t('settings:notifications.grouping.minCount.description')}
							value={prefs.minCountToNotify}
							options={MIN_COUNT_OPTIONS}
							onChange={(v) => setPref('minCountToNotify', v)}
							disabled={off && !prefs.showInCenter}
						/>
					</div>
				</motion.section>

				{/* ── OFF HINT ──────────────────────────────────────────── */}
				{off && !prefs.showInCenter && (
					<motion.div
						{...fade(0.25)}
						className='flex items-center gap-3 rounded-2xl border border-status-warning/30 bg-status-warning/15 p-4'>
						<BellOff className='h-5 w-5 flex-shrink-0 text-status-warning' />
						<p className='text-sm text-status-warning'>
							{t('settings:notifications.allOffHint')}
						</p>
					</motion.div>
				)}
			</div>
		</div>
	)
}
