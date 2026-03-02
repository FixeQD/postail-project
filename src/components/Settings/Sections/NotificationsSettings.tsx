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
		<div
			className={`flex items-center justify-between rounded-2xl border border-white/[0.05] bg-white/[0.03] p-4 transition-all duration-200 ${
				disabled
					? 'cursor-not-allowed opacity-50'
					: 'hover:border-white/[0.08] hover:bg-white/[0.06]'
			}`}>
			<div className='flex items-center gap-4'>
				<div
					className='flex h-10 w-10 items-center justify-center rounded-xl ring-1 ring-white/[0.08]'
					style={{ backgroundColor: `rgba(var(--accent-rgb), 0.08)` }}>
					<Icon className='h-5 w-5' style={{ color: accentColor }} />
				</div>
				<div>
					<p className='text-sm font-semibold text-slate-200'>{label}</p>
					<p className='text-xs text-slate-500'>{description}</p>
				</div>
			</div>
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
								: 'text-slate-500 ring-1 ring-white/[0.08] hover:text-slate-300 hover:ring-white/[0.15]'
						}`}
						style={value === opt ? { backgroundColor: accentColor } : {}}>
						{opt === 1 ? 'All' : `${opt}+`}
					</button>
				))}
			</div>
		</div>
	)
}

// ── Section heading ────────────────────────────────────────────────
function SectionTitle({ children }: { children: React.ReactNode }) {
	return (
		<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-slate-500 uppercase'>
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
		<div className='mx-auto flex h-full w-full max-w-3xl flex-col space-y-8 overflow-y-auto p-8'>
			{/* Header */}
			<motion.div
				{...(anim
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
					{t('settings:notifications.title')}
				</h1>
				<p className='mt-1 text-slate-400'>{t('settings:notifications.subtitle')}</p>
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
						className='flex items-center gap-3 rounded-2xl border border-amber-500/20 bg-amber-500/5 p-4'>
						<BellOff className='h-5 w-5 flex-shrink-0 text-amber-500/70' />
						<p className='text-sm text-amber-400/80'>
							{t('settings:notifications.allOffHint')}
						</p>
					</motion.div>
				)}
			</div>
		</div>
	)
}
