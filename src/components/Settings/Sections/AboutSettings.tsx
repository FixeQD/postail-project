import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { GitBranch, GitCommit, Clock, Cpu, Package, Terminal } from 'lucide-react'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { motion } from 'framer-motion'
import { useTranslation } from 'react-i18next'
import type { BuildInfo } from '@/types/components/shared'

const Row = ({
	icon: Icon,
	label,
	value,
	mono = false,
	index = 0,
	animationsEnabled = false,
}: {
	icon: React.ElementType
	label: string
	value: string
	mono?: boolean
	index?: number
	animationsEnabled?: boolean
}) => (
	<motion.div
		{...(animationsEnabled
			? {
					initial: { opacity: 0, y: 8 },
					animate: { opacity: 1, y: 0 },
					transition: { duration: 0.25, delay: index * 0.06, ease: [0.16, 1, 0.3, 1] },
				}
			: {})}
		className='flex items-center justify-between rounded-2xl border border-white/5 bg-white/5 px-5 py-3.5 transition-colors hover:bg-white/[0.07]'>
		<div className='flex items-center gap-3.5'>
			<div className='flex h-9 w-9 items-center justify-center rounded-xl bg-slate-900 ring-1 ring-white/10'>
				<Icon className='h-4 w-4 text-slate-400' />
			</div>
			<span className='text-sm font-medium text-slate-300'>{label}</span>
		</div>
		<span
			className={`max-w-[340px] truncate text-sm text-slate-400 ${mono ? 'font-mono tracking-tight' : ''}`}>
			{value}
		</span>
	</motion.div>
)

export function AboutSettings() {
	const animationsEnabled = useAnimationsEnabled()
	const { i18n } = useTranslation('common')
	const locale =
		(i18n.getResourceBundle(i18n.language, 'common') as { app?: { languageCode?: string } })
			?.app?.languageCode ?? i18n.language
	const [info, setInfo] = useState<BuildInfo | null>(null)

	useEffect(() => {
		invoke<BuildInfo>('get_build_info')
			.then(setInfo)
			.catch((e) => console.error('Failed to fetch build info:', e))
	}, [])

	const rows = info
		? [
				{ icon: Package, label: 'Version', value: `v${info.version}`, mono: false },
				{
					icon: Clock,
					label: 'Build date',
					value: new Date(parseInt(info.build_timestamp) * 1000).toLocaleString(locale, {
						year: 'numeric',
						month: 'short',
						day: 'numeric',
						hour: '2-digit',
						minute: '2-digit',
						second: '2-digit',
					}),
					mono: false,
				},
				{ icon: GitCommit, label: 'Commit', value: info.git_hash, mono: true },
				{ icon: GitBranch, label: 'Branch', value: info.git_branch, mono: true },
				{
					icon: Cpu,
					label: 'Profile',
					value: info.profile.charAt(0).toUpperCase() + info.profile.slice(1),
					mono: false,
				},
				{ icon: Terminal, label: 'Compiler', value: info.rustc, mono: true },
			]
		: []

	return (
		<div className='flex h-full flex-col overflow-y-auto p-8'>
			{/* Header */}
			<motion.div
				{...(animationsEnabled
					? {
							initial: { opacity: 0, y: -8 },
							animate: { opacity: 1, y: 0 },
							transition: { duration: 0.3 },
						}
					: {})}
				className='mb-8 flex items-center gap-4'>
				<div className='flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-900 ring-1 ring-white/10'>
					<Package className='h-7 w-7 text-slate-400' />
				</div>
				<div>
					<h1 className='text-xl font-bold text-slate-100'>Postail</h1>
					<p className='text-sm text-slate-500'>{info ? `v${info.version}` : '—'}</p>
				</div>
			</motion.div>

			{/* Rows */}
			<div className='space-y-2'>
				{info ? (
					rows.map((row, i) => (
						<Row
							key={row.label}
							icon={row.icon}
							label={row.label}
							value={row.value}
							mono={row.mono}
							index={i}
							animationsEnabled={animationsEnabled}
						/>
					))
				) : (
					<div className='flex flex-col gap-2'>
						{[...Array(6)].map((_, i) => (
							<div key={i} className='skeleton h-[52px] rounded-2xl' />
						))}
					</div>
				)}
			</div>
		</div>
	)
}
