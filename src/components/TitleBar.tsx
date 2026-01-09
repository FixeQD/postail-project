import { getCurrentWindow } from '@tauri-apps/api/window'

export function TitleBar() {
	const appWindow = getCurrentWindow()

	const minimize = () => appWindow.minimize()
	const toggleMaximize = () => appWindow.toggleMaximize()
	const close = () => appWindow.close()
	const startDrag = () => appWindow.startDragging()

	return (
		<div
			className='flex h-10 shrink-0 items-center justify-between border-b border-gray-800 bg-gray-950 select-none'
			onMouseDown={startDrag}>
			<div className='flex items-center gap-2 px-4'>
				<span className='text-sm font-medium tracking-wide text-gray-200'>Postail</span>
			</div>

			<div className='flex h-full'>
				{/* Minimize */}
				<button
					onClick={(e) => {
						e.stopPropagation()
						minimize()
					}}
					onMouseDown={(e) => e.stopPropagation()}
					className='flex h-full w-12 items-center justify-center text-gray-400 transition-colors duration-150 hover:bg-gray-800 hover:text-gray-200'>
					<svg className='h-4 w-4' fill='none' stroke='currentColor' viewBox='0 0 24 24'>
						<path
							strokeLinecap='round'
							strokeLinejoin='round'
							strokeWidth={2}
							d='M20 12H4'
						/>
					</svg>
				</button>

				{/* Maximize */}
				<button
					onClick={(e) => {
						e.stopPropagation()
						toggleMaximize()
					}}
					onMouseDown={(e) => e.stopPropagation()}
					className='flex h-full w-12 items-center justify-center text-gray-400 transition-colors duration-150 hover:bg-gray-800 hover:text-gray-200'>
					<svg
						className='h-3.5 w-3.5'
						fill='none'
						stroke='currentColor'
						viewBox='0 0 24 24'>
						<rect x='4' y='4' width='16' height='16' rx='1' strokeWidth={2} />
					</svg>
				</button>

				{/* Close */}
				<button
					onClick={(e) => {
						e.stopPropagation()
						close()
					}}
					onMouseDown={(e) => e.stopPropagation()}
					className='flex h-full w-12 items-center justify-center text-gray-400 transition-colors duration-150 hover:bg-red-600 hover:text-white'>
					<svg className='h-4 w-4' fill='none' stroke='currentColor' viewBox='0 0 24 24'>
						<path
							strokeLinecap='round'
							strokeLinejoin='round'
							strokeWidth={2}
							d='M6 18L18 6M6 6l12 12'
						/>
					</svg>
				</button>
			</div>
		</div>
	)
}
