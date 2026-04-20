import * as React from 'react'
import { cn } from '@/lib/utils'

interface AvatarContextValue {
	imgError: boolean
	setImgError: (v: boolean) => void
}
const AvatarContext = React.createContext<AvatarContextValue>({
	imgError: false,
	setImgError: () => {},
})

function Avatar({ className, ...props }: React.ComponentProps<'div'>) {
	const [imgError, setImgError] = React.useState(false)
	return (
		<AvatarContext.Provider value={{ imgError, setImgError }}>
			<div
				data-slot='avatar'
				className={cn(
					'relative flex size-8 shrink-0 overflow-hidden rounded-full',
					className
				)}
				{...props}
			/>
		</AvatarContext.Provider>
	)
}

function AvatarImage({ className, src, alt, onError, ...props }: React.ComponentProps<'img'>) {
	const { imgError, setImgError } = React.useContext(AvatarContext)
	if (imgError || !src) return null
	return (
		<img
			data-slot='avatar-image'
			src={src}
			alt={alt}
			className={cn('aspect-square size-full', className)}
			onError={(e) => {
				setImgError(true)
				onError?.(e)
			}}
			{...props}
		/>
	)
}

function AvatarFallback({ className, ...props }: React.ComponentProps<'div'>) {
	const { imgError } = React.useContext(AvatarContext)
	if (!imgError) return null
	return (
		<div
			data-slot='avatar-fallback'
			className={cn(
				'bg-muted flex size-full items-center justify-center rounded-full',
				className
			)}
			{...props}
		/>
	)
}

export { Avatar, AvatarImage, AvatarFallback }
