import { useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { motion, AnimatePresence } from 'framer-motion'
import {
	Bug,
	X,
	ChevronDown,
	ChevronRight,
	Trash2,
	RefreshCw,
	Database,
	Settings,
	Layers,
	Terminal,
	Copy,
	Check,
	AlertTriangle,
	Zap,
} from 'lucide-react'
import { useAccountStore } from '@/stores/accountStore'
import { useSettingsStore } from '@/stores/settingsStore'
import { useSyncStore } from '@/stores/syncStore'
import { useThemeStore } from '@/stores/themeStore'
import { APP_STATES } from '@/types/hooks'
import type { DevToolsProps, DevToolsSection, ResetOptions } from '@/types/components/devtools'

function useCopyToClipboard() {
	const [copied, setCopied] = useState(false)
	const copy = (text: string) => {
		navigator.clipboard.writeText(text)
		setCopied(true)
		setTimeout(() => setCopied(false), 1500)
	}
	return { copied, copy }
}

function Section({
	label,
	icon,
	open,
	onToggle,
	children,
}: {
	label: string
	icon: React.ReactNode
	open: boolean
	onToggle: () => void
	children: React.ReactNode
}) {
	return (
		<div className='border-b border-white/[0.06]'>
			<button
				type='button'
				onClick={onToggle}
				className='flex w-full items-center gap-2 px-3 py-2 text-left transition-colors hover:bg-white/[0.04]'>
				<span className='text-white/30'>{icon}</span>
				<span className='flex-1 text-[11px] font-semibold tracking-wider text-white/50 uppercase'>
					{label}
				</span>
				{open ? (
					<ChevronDown className='h-3 w-3 text-white/20' />
				) : (
					<ChevronRight className='h-3 w-3 text-white/20' />
				)}
			</button>
			{open && <div className='pb-2'>{children}</div>}
		</div>
	)
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
	return (
		<div className='flex items-center justify-between gap-3 px-3 py-1'>
			<span className='shrink-0 text-[10px] text-white/30'>{label}</span>
			<div className='min-w-0'>{children}</div>
		</div>
	)
}

function Tag({
	children,
	color = 'default',
}: {
	children: React.ReactNode
	color?: 'default' | 'green' | 'red' | 'yellow' | 'blue'
}) {
	const colors = {
		default: 'bg-white/10 text-white/60',
		green: 'bg-emerald-500/20 text-emerald-400',
		red: 'bg-red-500/20 text-red-400',
		yellow: 'bg-amber-500/20 text-amber-400',
		blue: 'bg-sky-500/20 text-sky-400',
	}
	return (
		<span className={`rounded px-1.5 py-0.5 font-mono text-[10px] ${colors[color]}`}>
			{children}
		</span>
	)
}

function DevLog({ entries }: { entries: string[] }) {
	const ref = useRef<HTMLDivElement>(null)
	useEffect(() => {
		if (ref.current) ref.current.scrollTop = ref.current.scrollHeight
	}, [entries])
	if (!entries.length) return null
	return (
		<div
			ref={ref}
			className='mx-3 mb-2 max-h-24 overflow-y-auto rounded bg-black/40 p-2 font-mono text-[9px] text-emerald-400'>
			{entries.map((e, i) => (
				<div key={i}>
					{'> '}
					{e}
				</div>
			))}
		</div>
	)
}

export function DevTools({ currentState, setCurrentState }: DevToolsProps) {
	const [open, setOpen] = useState(false)
	const [sections, setSections] = useState<Record<DevToolsSection, boolean>>({
		appstate: true,
		reset: false,
		stores: false,
		commands: false,
		settings: false,
	})
	const [resetOpts, setResetOpts] = useState<ResetOptions>({
		messages: true,
		emlCache: true,
		bodyCache: true,
		attachments: false,
		contacts: false,
		settings: false,
		outbox: false,
	})
	const [log, setLog] = useState<string[]>([])
	const [busy, setBusy] = useState(false)
	const { copied, copy } = useCopyToClipboard()

	const accounts = useAccountStore((s) => s.accounts)
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const settings = useSettingsStore((s) => s.settings)
	const statusMap = useSyncStore((s) => s.statuses)
	const syncStatuses = Array.from(statusMap.values())
	const { accentColor, darkMode } = useThemeStore()

	const pushLog = (...msgs: string[]) => setLog((l) => [...l, ...msgs])

	const toggleSection = (id: DevToolsSection) => setSections((s) => ({ ...s, [id]: !s[id] }))

	const handleReset = async () => {
		const anySelected = Object.values(resetOpts).some(Boolean)
		if (!anySelected) {
			pushLog('Nothing selected.')
			return
		}
		setBusy(true)
		pushLog('Running reset...')
		try {
			const result = await invoke<string[]>('dev_reset_data', {
				clearMessages: resetOpts.messages,
				clearEmlCache: resetOpts.emlCache,
				clearBodyCache: resetOpts.bodyCache,
				clearAttachments: resetOpts.attachments,
				clearContacts: resetOpts.contacts,
				clearSettings: resetOpts.settings,
				clearOutbox: resetOpts.outbox,
			})
			pushLog(...result, '✓ Done. Reload the app to see changes.')
		} catch (e) {
			pushLog(`✗ Error: ${String(e)}`)
		} finally {
			setBusy(false)
		}
	}

	const handleRunMaintenance = async () => {
		setBusy(true)
		pushLog('Running maintenance...')
		try {
			await invoke('run_maintenance')
			pushLog('✓ Maintenance done.')
		} catch (e) {
			pushLog(`✗ ${String(e)}`)
		} finally {
			setBusy(false)
		}
	}

	const handleStartSync = async (accountId: string) => {
		setBusy(true)
		pushLog(`Starting sync for ${accountId}...`)
		try {
			await invoke('start_sync', { accountId })
			pushLog('✓ Sync started.')
		} catch (e) {
			pushLog(`✗ ${String(e)}`)
		} finally {
			setBusy(false)
		}
	}

	const handleExportBackup = async () => {
		setBusy(true)
		pushLog('Exporting backup...')
		try {
			const path = await invoke<string>('export_backup', { passphrase: null })
			pushLog(`✓ Backup saved to: ${path}`)
		} catch (e) {
			pushLog(`✗ ${String(e)}`)
		} finally {
			setBusy(false)
		}
	}

	const storeSnapshot = JSON.stringify(
		{
			appState: currentState,
			activeAccount: activeAccount?.email ?? null,
			accountCount: accounts.length,
			settings,
			syncStatuses: syncStatuses.map((s) => ({
				email: s.accountEmail,
				status: s.status,
				lastSync: s.lastSync,
			})),
		},
		null,
		2
	)

	const CheckBox = ({
		label,
		checked,
		onChange,
		danger,
	}: {
		label: string
		checked: boolean
		onChange: (v: boolean) => void
		danger?: boolean
	}) => (
		<label className='flex cursor-pointer items-center gap-2 px-3 py-1 hover:bg-white/[0.03]'>
			<input
				type='checkbox'
				checked={checked}
				onChange={(e) => onChange(e.target.checked)}
				className='h-3 w-3 cursor-pointer accent-sky-500'
			/>
			<span className={`text-[10px] ${danger ? 'text-red-400' : 'text-white/50'}`}>
				{label}
			</span>
		</label>
	)

	return (
		<>
			{/* Trigger button */}
			<motion.button
				type='button'
				onClick={() => setOpen((o) => !o)}
				whileHover={{ scale: 1.08 }}
				whileTap={{ scale: 0.92 }}
				className='fixed right-3 bottom-9 z-[9998] flex h-7 w-7 items-center justify-center rounded-full bg-violet-600/90 shadow-lg ring-1 shadow-violet-900/50 ring-violet-400/30 backdrop-blur-sm transition-opacity'
				title='DevTools'>
				<Bug className='h-3.5 w-3.5 text-white' />
			</motion.button>

			{/* Panel */}
			<AnimatePresence>
				{open && (
					<motion.div
						initial={{ opacity: 0, x: 320 }}
						animate={{ opacity: 1, x: 0 }}
						exit={{ opacity: 0, x: 320 }}
						transition={{ type: 'spring', stiffness: 400, damping: 35 }}
						className='fixed top-0 right-0 bottom-0 z-[9999] flex w-72 flex-col overflow-hidden border-l border-white/[0.07] bg-[#0d0d0f]/95 shadow-2xl backdrop-blur-xl'>
						{/* Header */}
						<div className='flex shrink-0 items-center gap-2 border-b border-white/[0.07] px-3 py-2.5'>
							<Bug className='h-3.5 w-3.5 text-violet-400' />
							<span className='flex-1 text-[11px] font-bold tracking-widest text-white/70 uppercase'>
								DevTools
							</span>
							<Tag color='yellow'>DEV</Tag>
							<button
								type='button'
								onClick={() => setOpen(false)}
								className='ml-1 flex h-5 w-5 items-center justify-center rounded text-white/30 transition-colors hover:bg-white/10 hover:text-white/70'>
								<X className='h-3 w-3' />
							</button>
						</div>

						{/* Scrollable body */}
						<div className='min-h-0 flex-1 overflow-y-auto'>
							{/* App State */}
							<Section
								label='App State'
								icon={<Layers className='h-3 w-3' />}
								open={sections.appstate}
								onToggle={() => toggleSection('appstate')}>
								<div className='px-3 pt-0.5 pb-1'>
									<div className='mb-1.5 flex items-center gap-1.5'>
										<span className='text-[10px] text-white/30'>current:</span>
										<Tag color='blue'>{currentState}</Tag>
									</div>
									<div className='grid grid-cols-2 gap-1'>
										{APP_STATES.map((s) => (
											<button
												key={s}
												type='button'
												onClick={() => {
													setCurrentState(s)
													pushLog(`→ appState set to "${s}"`)
												}}
												className={`rounded px-2 py-1 text-left text-[10px] transition-colors ${
													s === currentState
														? 'bg-violet-600/40 text-violet-300 ring-1 ring-violet-500/40'
														: 'text-white/40 hover:bg-white/[0.06] hover:text-white/70'
												}`}>
												{s}
											</button>
										))}
									</div>
								</div>
							</Section>

							{/* Reset DB */}
							<Section
								label='Reset Data'
								icon={<Trash2 className='h-3 w-3' />}
								open={sections.reset}
								onToggle={() => toggleSection('reset')}>
								<div className='px-3 pt-0.5 pb-1'>
									<div className='mb-1 flex items-center gap-1 text-[9px] text-amber-400/70'>
										<AlertTriangle className='h-2.5 w-2.5' />
										Irreversible. Restart app after reset.
									</div>
								</div>
								<CheckBox
									label='Messages'
									checked={resetOpts.messages}
									onChange={(v) => setResetOpts((o) => ({ ...o, messages: v }))}
									danger
								/>
								<CheckBox
									label='EML file cache (raw emails)'
									checked={resetOpts.emlCache}
									onChange={(v) => setResetOpts((o) => ({ ...o, emlCache: v }))}
								/>
								<CheckBox
									label='Body JSON cache (parsed)'
									checked={resetOpts.bodyCache}
									onChange={(v) => setResetOpts((o) => ({ ...o, bodyCache: v }))}
								/>
								<CheckBox
									label='Attachments'
									checked={resetOpts.attachments}
									onChange={(v) =>
										setResetOpts((o) => ({ ...o, attachments: v }))
									}
								/>
								<CheckBox
									label='Contacts'
									checked={resetOpts.contacts}
									onChange={(v) => setResetOpts((o) => ({ ...o, contacts: v }))}
								/>
								<CheckBox
									label='Outbox'
									checked={resetOpts.outbox}
									onChange={(v) => setResetOpts((o) => ({ ...o, outbox: v }))}
								/>
								<CheckBox
									label='Settings'
									checked={resetOpts.settings}
									onChange={(v) => setResetOpts((o) => ({ ...o, settings: v }))}
									danger
								/>
								<div className='px-3 pt-2'>
									<button
										type='button'
										disabled={busy}
										onClick={handleReset}
										className='flex w-full items-center justify-center gap-1.5 rounded bg-red-600/20 px-3 py-1.5 text-[10px] font-semibold text-red-400 ring-1 ring-red-500/30 transition-colors hover:bg-red-600/30 disabled:opacity-40'>
										<Trash2 className='h-3 w-3' />
										Run Reset
									</button>
								</div>
								<DevLog entries={log} />
							</Section>

							{/* Stores snapshot */}
							<Section
								label='Store Snapshot'
								icon={<Database className='h-3 w-3' />}
								open={sections.stores}
								onToggle={() => toggleSection('stores')}>
								<div className='px-3 pt-1'>
									<Row label='accounts'>
										<Tag>{accounts.length}</Tag>
									</Row>
									<Row label='active'>
										<Tag color={activeAccount ? 'green' : 'default'}>
											{activeAccount?.email ?? 'none'}
										</Tag>
									</Row>
									<Row label='accent'>
										<span className='flex items-center gap-1.5'>
											<span
												className='h-3 w-3 rounded-full'
												style={{ backgroundColor: accentColor }}
											/>
											<Tag>{accentColor}</Tag>
										</span>
									</Row>
									<Row label='dark mode'>
										<Tag color={darkMode ? 'blue' : 'yellow'}>
											{darkMode ? 'on' : 'off'}
										</Tag>
									</Row>

									{syncStatuses.length > 0 && (
										<div className='mt-1.5 px-0'>
											<div className='mb-0.5 px-0 text-[9px] text-white/20'>
												sync status
											</div>
											{syncStatuses.map((s) => (
												<Row
													key={s.accountId}
													label={s.accountEmail.split('@')[0]}>
													<Tag
														color={
															s.status === 'syncing'
																? 'yellow'
																: s.status === 'error'
																	? 'red'
																	: 'green'
														}>
														{s.status}
													</Tag>
												</Row>
											))}
										</div>
									)}

									<div className='mt-2 mb-1'>
										<button
											type='button'
											onClick={() => copy(storeSnapshot)}
											className='flex w-full items-center justify-center gap-1.5 rounded bg-white/[0.05] px-2 py-1 text-[10px] text-white/40 transition-colors hover:bg-white/[0.09] hover:text-white/70'>
											{copied ? (
												<Check className='h-3 w-3 text-emerald-400' />
											) : (
												<Copy className='h-3 w-3' />
											)}
											{copied ? 'Copied!' : 'Copy full snapshot'}
										</button>
									</div>
								</div>
							</Section>

							{/* Commands */}
							<Section
								label='Commands'
								icon={<Terminal className='h-3 w-3' />}
								open={sections.commands}
								onToggle={() => toggleSection('commands')}>
								<div className='flex flex-col gap-1 px-3 pt-1'>
									<button
										type='button'
										disabled={busy}
										onClick={handleRunMaintenance}
										className='flex items-center gap-2 rounded bg-white/[0.05] px-3 py-1.5 text-left text-[10px] text-white/50 transition-colors hover:bg-white/[0.09] hover:text-white/70 disabled:opacity-40'>
										<Zap className='h-3 w-3 text-amber-400' />
										run_maintenance()
									</button>
									<button
										type='button'
										disabled={busy}
										onClick={handleExportBackup}
										className='flex items-center gap-2 rounded bg-white/[0.05] px-3 py-1.5 text-left text-[10px] text-white/50 transition-colors hover:bg-white/[0.09] hover:text-white/70 disabled:opacity-40'>
										<Database className='h-3 w-3 text-sky-400' />
										export_backup()
									</button>
									{accounts.map((acc) => (
										<button
											key={acc.id}
											type='button'
											disabled={busy}
											onClick={() => handleStartSync(acc.id)}
											className='flex items-center gap-2 rounded bg-white/[0.05] px-3 py-1.5 text-left text-[10px] text-white/50 transition-colors hover:bg-white/[0.09] hover:text-white/70 disabled:opacity-40'>
											<RefreshCw className='h-3 w-3 text-emerald-400' />
											start_sync({acc.email.split('@')[0]})
										</button>
									))}
									<DevLog entries={log} />
								</div>
							</Section>

							{/* Settings dump */}
							<Section
								label='Active Settings'
								icon={<Settings className='h-3 w-3' />}
								open={sections.settings}
								onToggle={() => toggleSection('settings')}>
								<div className='px-3 pt-1 pb-2'>
									{Object.entries(settings).map(([key, val]) => (
										<Row key={key} label={key}>
											<Tag
												color={
													val === true
														? 'green'
														: val === false
															? 'red'
															: 'default'
												}>
												{String(val) || '""'}
											</Tag>
										</Row>
									))}
								</div>
							</Section>
						</div>

						{/* Footer */}
						<div className='shrink-0 border-t border-white/[0.06] px-3 py-2'>
							<div className='flex items-center gap-1.5 text-[9px] text-white/20'>
								<Bug className='h-2.5 w-2.5 text-violet-500/60' />
								Only visible in{' '}
								<code className='text-violet-400/70'>import.meta.env.DEV</code>
							</div>
						</div>
					</motion.div>
				)}
			</AnimatePresence>
		</>
	)
}
