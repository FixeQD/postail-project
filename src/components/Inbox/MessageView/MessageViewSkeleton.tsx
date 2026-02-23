
export const MessageViewSkeleton = () => {
	const skeletonStyle = {
		backgroundImage:
			'linear-gradient(90deg, rgba(var(--accent-rgb), 0.05) 25%, rgba(var(--accent-rgb), 0.1) 50%, rgba(var(--accent-rgb), 0.05) 75%)',
		backgroundSize: '200% 100%',
		animation: 'shimmer 1.5s ease-in-out infinite',
	}

	return (
		<div className='flex h-full flex-col bg-slate-900'>
			{/* Header imitation */}
			<div className='flex h-14 items-center gap-2 border-b border-white/[0.06] bg-slate-900/50 px-4'>
				<div className='h-8 w-8 rounded-lg' style={skeletonStyle} />
				<div className='mx-2 h-6 w-px bg-white/[0.08]' />
				<div className='h-8 w-8 rounded-lg' style={skeletonStyle} />
				<div className='h-8 w-8 rounded-lg' style={skeletonStyle} />
				<div className='h-8 w-8 rounded-lg' style={skeletonStyle} />
				<div className='ml-auto h-8 w-24 rounded-lg' style={skeletonStyle} />
			</div>

			<div className='flex-1 overflow-y-auto px-6 py-4'>
				{/* Meta imitation */}
				<div className='space-y-4'>
					<div className='h-7 w-3/4 rounded' style={skeletonStyle} />
					<div className='flex items-center gap-3'>
						<div className='h-5 w-1/3 rounded' style={skeletonStyle} />
						<div className='h-5 w-1/4 rounded' style={skeletonStyle} />
					</div>
					<div className='h-4 w-48 rounded' style={skeletonStyle} />
				</div>

				{/* Body imitation */}
				<div className='mt-8 space-y-3'>
					<div className='h-4 w-full rounded' style={skeletonStyle} />
					<div className='h-4 w-[98%] rounded' style={skeletonStyle} />
					<div className='h-4 w-[95%] rounded' style={skeletonStyle} />
					<div className='h-4 w-[90%] rounded' style={skeletonStyle} />
					<div className='h-4 w-[92%] rounded' style={skeletonStyle} />
					<div className='h-4 w-[60%] rounded' style={skeletonStyle} />
				</div>

				<div className='mt-6 space-y-3'>
					<div className='h-4 w-full rounded' style={skeletonStyle} />
					<div className='h-4 w-[96%] rounded' style={skeletonStyle} />
					<div className='h-4 w-[93%] rounded' style={skeletonStyle} />
					<div className='h-4 w-[40%] rounded' style={skeletonStyle} />
				</div>
			</div>
		</div>
	)
}
