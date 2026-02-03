import * as React from 'react'
import * as TooltipPrimitive from '@radix-ui/react-tooltip'

import { cn } from '@/lib/utils'

/**
 * Wraps Radix's TooltipProvider to apply a default `delayDuration` and a `data-slot` attribute.
 *
 * @param delayDuration - Milliseconds to wait before showing the tooltip; defaults to `0`.
 * @returns A React element rendering a TooltipProvider with `data-slot="tooltip-provider"` and forwarded props.
 */
function TooltipProvider({
	delayDuration = 0,
	...props
}: React.ComponentProps<typeof TooltipPrimitive.Provider>) {
	return (
		<TooltipPrimitive.Provider
			data-slot='tooltip-provider'
			delayDuration={delayDuration}
			{...props}
		/>
	)
}

/**
 * Wraps Radix TooltipRoot with the module's TooltipProvider and forwards all props.
 *
 * @param props - Props forwarded to Radix TooltipPrimitive.Root
 * @returns A TooltipRoot element wrapped with the module's TooltipProvider
 */
function Tooltip({ ...props }: React.ComponentProps<typeof TooltipPrimitive.Root>) {
	return (
		<TooltipProvider>
			<TooltipPrimitive.Root data-slot='tooltip' {...props} />
		</TooltipProvider>
	)
}

/**
 * Renders a Radix Tooltip Trigger and forwards all received props to the underlying trigger.
 *
 * @param props - Props to pass through to `TooltipPrimitive.Trigger`. A `data-slot="tooltip-trigger"` attribute is applied.
 * @returns The rendered tooltip trigger element with forwarded props.
 */
function TooltipTrigger({ ...props }: React.ComponentProps<typeof TooltipPrimitive.Trigger>) {
	return <TooltipPrimitive.Trigger data-slot='tooltip-trigger' {...props} />
}

/**
 * Renders styled tooltip content inside a portal with entrance/exit animations and a matching arrow.
 *
 * @param className - Additional CSS classes merged with the component's default styling
 * @param sideOffset - Distance in pixels between the trigger and the tooltip; defaults to `0`
 * @returns A React element containing the tooltip content and arrow
 */
function TooltipContent({
	className,
	sideOffset = 0,
	children,
	...props
}: React.ComponentProps<typeof TooltipPrimitive.Content>) {
	return (
		<TooltipPrimitive.Portal>
			<TooltipPrimitive.Content
				data-slot='tooltip-content'
				sideOffset={sideOffset}
				className={cn(
					'bg-foreground text-background animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 w-fit origin-(--radix-tooltip-content-transform-origin) rounded-md px-3 py-1.5 text-xs text-balance',
					className
				)}
				{...props}>
				{children}
				<TooltipPrimitive.Arrow className='bg-foreground fill-foreground z-50 size-2.5 translate-y-[calc(-50%_-_2px)] rotate-45 rounded-[2px]' />
			</TooltipPrimitive.Content>
		</TooltipPrimitive.Portal>
	)
}

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider }