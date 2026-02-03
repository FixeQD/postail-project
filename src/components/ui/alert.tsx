import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'

const alertVariants = cva(
	'relative w-full rounded-lg border px-4 py-3 text-sm grid has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] grid-cols-[0_1fr] has-[>svg]:gap-x-3 gap-y-0.5 items-start [&>svg]:size-4 [&>svg]:translate-y-0.5 [&>svg]:text-current',
	{
		variants: {
			variant: {
				default: 'bg-card text-card-foreground',
				destructive:
					'text-destructive bg-card [&>svg]:text-current *:data-[slot=alert-description]:text-destructive/90',
			},
		},
		defaultVariants: {
			variant: 'default',
		},
	}
)

/**
 * Renders an alert container element with variant-driven styling.
 *
 * The rendered element has `data-slot="alert"` and `role="alert"`, and merges variant classes with any provided `className`.
 *
 * @param variant - Visual variant to apply (e.g., `"default"`, `"destructive"`). Defaults to the variant defined by `alertVariants`.
 * @param className - Additional CSS classes to merge with the variant classes.
 * @returns A `div` element configured as an alert with the selected variant classes applied.
 */
function Alert({
	className,
	variant,
	...props
}: React.ComponentProps<'div'> & VariantProps<typeof alertVariants>) {
	return (
		<div
			data-slot='alert'
			role='alert'
			className={cn(alertVariants({ variant }), className)}
			{...props}
		/>
	)
}

/**
 * Renders the alert's title slot.
 *
 * @param className - Additional class names to merge with the component's default title styles
 * @returns A `div` element used as the alert's title with `data-slot='alert-title'` and merged class names
 */
function AlertTitle({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='alert-title'
			className={cn('col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight', className)}
			{...props}
		/>
	)
}

/**
 * Renders the alert description slot with default typography and layout classes.
 *
 * Merges any provided `className` with the component's default description styles,
 * sets `data-slot="alert-description"`, and forwards remaining props to the root div.
 *
 * @param className - Additional CSS classes to merge with the default description styles
 * @param props - Additional attributes forwarded to the root div element
 * @returns The rendered alert description element
 */
function AlertDescription({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='alert-description'
			className={cn(
				'text-muted-foreground col-start-2 grid justify-items-start gap-1 text-sm [&_p]:leading-relaxed',
				className
			)}
			{...props}
		/>
	)
}

export { Alert, AlertTitle, AlertDescription }