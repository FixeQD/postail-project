import { useState, useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { platform } from '@tauri-apps/plugin-os'
import { Search, Bell, Settings } from 'lucide-react'
import icon from '../assets/icon.png'
import type { AccountMeta } from '../types/accounts'
import { useTypedTranslation } from '../hooks/useTypedTranslation'

interface TitleBarProps {
	isDashboard?: boolean
	activeAccount?: AccountMeta | null
	onSearch?: (query: string) => void
	onOpenSettings?: () => void
}

export function TitleBar({ isDashboard, activeAccount, onSearch, onOpenSettings }: TitleBarProps) {
	const { t } = useTypedTranslation()
	const [isMobile, setIsMobile] = useState<boolean | null>(null)
	const [searchQuery, setSearchQuery] = useState('')

	useEffect(() => {
		try {
			const p = platform()
			setIsMobile(p === 'android' || p === 'ios')
		} catch {
			const ua = navigator.userAgent.toLowerCase()
			setIsMobile(/android|iphone|ipad|ipod|mobile/i.test(ua))
		}
	}, [])

	if (isMobile === null || isMobile) {
		return null
	}

	const appWindow = getCurrentWindow()

	const minimize = () => appWindow.minimize()
	const toggleMaximize = () => appWindow.toggleMaximize()
	const close = () => appWindow.close()
	const startDrag = () => appWindow.startDragging()

	return (
		<div
			className='flex h-14 shrink-0 items-center justify-between border-b border-slate-900 bg-slate-950 select-none'
			onMouseDown={startDrag}>
			{/* Left: Branding */}
			<div className='flex w-64 shrink-0 items-center gap-3 px-4 pl-6'>
				<img src={icon} alt='Postail' className='h-8 w-8' />
				<span className='text-lg font-bold tracking-tight text-white'>Postail</span>
			</div>

			{/* Middle: Search Bar (Dashboard Only) */}
			<div className='flex flex-1 justify-center px-4'>
				{isDashboard && (
					<div className='relative w-full max-w-2xl'>
						<div className='pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3'>
							<Search className='h-4 w-4 text-slate-500' />
						</div>
						<input
							type='text'
							value={searchQuery}
							onChange={(e) => {
								setSearchQuery(e.target.value)
								onSearch?.(e.target.value)
							}}
							onMouseDown={(e) => e.stopPropagation()} // Allow interaction
							placeholder={t('inbox:search.placeholder')}
							className='block w-full rounded-lg bg-slate-900 py-2 pr-3 pl-10 text-sm text-slate-200 placeholder-slate-500 focus:bg-slate-800 focus:ring-1 focus:ring-slate-700 focus:outline-none'
						/>
					</div>
				)}
			</div>

			<div className='flex w-64 shrink-0 items-center justify-end gap-2 px-2'>
				{/* Dashboard Actions */}
				{isDashboard && activeAccount && (
					<div className='mr-4 flex items-center gap-3 border-r border-slate-800 pr-4'>
						<button
							className='relative text-slate-400 hover:text-slate-200'
							onMouseDown={(e) => e.stopPropagation()}>
							<Bell className='h-5 w-5' />
							<span className='absolute top-0 right-0 h-2 w-2 rounded-full bg-orange-500'></span>
						</button>
						<button
							className='text-slate-400 hover:text-slate-200'
							onClick={(e) => {
								e.stopPropagation()
								onOpenSettings?.()
							}}
							onMouseDown={(e) => e.stopPropagation()}>
							<Settings className='h-5 w-5' />
						</button>
						<div className='h-8 w-8 overflow-hidden rounded-full bg-orange-600 ring-2 ring-slate-900'>
							<div className='flex h-full w-full items-center justify-center font-bold text-white'>
								{activeAccount.name.charAt(0).toUpperCase()}
							</div>
						</div>
					</div>
				)}

				{/* Window Controls */}
				<div className='flex h-full items-center gap-1 pl-2'>
					{/* Minimize */}
					<button
						onClick={(e) => {
							e.stopPropagation()
							minimize()
						}}
						onMouseDown={(e) => e.stopPropagation()}
						className='group flex h-8 w-8 items-center justify-center rounded-full hover:bg-slate-800'>
						<div className='h-0.5 w-3 rounded-full bg-slate-400 group-hover:bg-slate-200' />
					</button>

					{/* Maximize */}
					<button
						onClick={(e) => {
							e.stopPropagation()
							toggleMaximize()
						}}
						onMouseDown={(e) => e.stopPropagation()}
						className='group flex h-8 w-8 items-center justify-center rounded-full hover:bg-slate-800'>
						<div className='h-2.5 w-2.5 rounded-[2px] border-2 border-slate-400 group-hover:border-slate-200' />
					</button>

					{/* Close */}
					<button
						onClick={(e) => {
							e.stopPropagation()
							close()
						}}
						onMouseDown={(e) => e.stopPropagation()}
						className='group flex h-8 w-8 items-center justify-center rounded-full hover:bg-red-500/10 hover:bg-red-600'>
						<svg
							className='h-4 w-4 text-slate-400 group-hover:text-white'
							fill='none'
							stroke='currentColor'
							viewBox='0 0 24 24'>
							<path
								strokeLinecap='round'
								strokeLinejoin='round'
								strokeWidth={2.5}
								d='M6 18L18 6M6 6l12 12'
							/>
						</svg>
					</button>
				</div>
			</div>
		</div>
	)
}
