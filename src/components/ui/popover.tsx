import * as React from 'react'
import * as PopoverPrimitive from '@radix-ui/react-popover'

import { cn } from '@/lib/utils'

/**
 * Render a Popover root element with a data-slot of 'popover'.
 *
 * @param props - Props to apply to the underlying Popover root; all props are forwarded unchanged.
 * @returns A React element representing the Popover root with the given props applied.
 */
function Popover({ ...props }: React.ComponentProps<typeof PopoverPrimitive.Root>) {
	return <PopoverPrimitive.Root data-slot='popover' {...props} />
}

/**
 * Renders a Radix popover trigger element with a data-slot of 'popover-trigger'.
 *
 * @param props - Props forwarded to the underlying PopoverPrimitive.Trigger
 * @returns The rendered popover trigger element
 */
function PopoverTrigger({ ...props }: React.ComponentProps<typeof PopoverPrimitive.Trigger>) {
	return <PopoverPrimitive.Trigger data-slot='popover-trigger' {...props} />
}

/**
 * Renders popover content inside a Portal with default alignment, offset, and composed styling.
 *
 * @param className - Additional CSS classes to merge with the component's default styles
 * @param align - Content alignment relative to the trigger (defaults to `center`)
 * @param sideOffset - Distance in pixels between the trigger and the content (defaults to `4`)
 * @returns The configured PopoverPrimitive.Content element rendered inside a Portal
 */
function PopoverContent({
	className,
	align = 'center',
	sideOffset = 4,
	...props
}: React.ComponentProps<typeof PopoverPrimitive.Content>) {
	return (
		<PopoverPrimitive.Portal>
			<PopoverPrimitive.Content
				data-slot='popover-content'
				align={align}
				sideOffset={sideOffset}
				className={cn(
					'bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 w-72 origin-(--radix-popover-content-transform-origin) rounded-md border p-4 shadow-md outline-hidden',
					className
				)}
				{...props}
			/>
		</PopoverPrimitive.Portal>
	)
}

/**
 * Renders a popover anchor element and forwards all received props to it.
 *
 * The rendered element includes a `data-slot="popover-anchor"` attribute to identify the slot.
 *
 * @param props - Props passed to the underlying popover anchor element
 * @returns The rendered popover anchor element
 */
function PopoverAnchor({ ...props }: React.ComponentProps<typeof PopoverPrimitive.Anchor>) {
	return <PopoverPrimitive.Anchor data-slot='popover-anchor' {...props} />
}

/**
 * Renders the header container for a popover.
 *
 * @param className - Additional CSS classes merged with the default header styles.
 * @param props - Additional props spread onto the underlying `div`.
 * @returns A `div` element with `data-slot="popover-header"` and header-specific styling.
 */
function PopoverHeader({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='popover-header'
			className={cn('flex flex-col gap-1 text-sm', className)}
			{...props}
		/>
	)
}

/**
 * Renders the popover's title element with base title styling and optional additional classes.
 *
 * @param className - Optional additional CSS class names to merge with the base title styles
 * @param props - Additional props are forwarded to the underlying element
 * @returns The rendered popover title element
 */
function PopoverTitle({ className, ...props }: React.ComponentProps<'h2'>) {
	return <div data-slot='popover-title' className={cn('font-medium', className)} {...props} />
}

/**
 * Renders the popover description element.
 *
 * @param className - Optional additional class names merged with the base `text-muted-foreground`
 * @param props - Additional props forwarded to the underlying `<p>` element
 * @returns The paragraph element used as the popover description
 */
function PopoverDescription({ className, ...props }: React.ComponentProps<'p'>) {
	return (
		<p
			data-slot='popover-description'
			className={cn('text-muted-foreground', className)}
			{...props}
		/>
	)
}

export {
	Popover,
	PopoverTrigger,
	PopoverContent,
	PopoverAnchor,
	PopoverHeader,
	PopoverTitle,
	PopoverDescription,
}