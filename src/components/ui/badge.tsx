import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const badgeVariants = cva(
	'inline-flex items-center justify-center rounded-full border border-transparent px-2 py-0.5 text-xs font-medium w-fit whitespace-nowrap shrink-0 [&>svg]:size-3 gap-1 [&>svg]:pointer-events-none transition-[color,box-shadow] overflow-hidden',
	{
		variants: {
			variant: {
				default: 'bg-primary text-primary-foreground',
				secondary: 'bg-secondary text-secondary-foreground',
				destructive: 'bg-destructive text-white',
				outline: 'border-border text-foreground',
				ghost: '',
				link: 'text-primary underline-offset-4',
			},
		},
		defaultVariants: { variant: 'default' },
	}
)

interface BadgeProps extends React.ComponentProps<'span'>, VariantProps<typeof badgeVariants> {
	asChild?: boolean
}

function Badge({
	className,
	variant = 'default',
	asChild = false,
	children,
	...props
}: BadgeProps) {
	if (asChild && React.isValidElement(children)) {
		const child = children as React.ReactElement<React.HTMLAttributes<HTMLElement>>
		return React.cloneElement(child, {
			...props,
			className: cn(badgeVariants({ variant }), child.props.className, className),
		} as React.HTMLAttributes<HTMLElement>)
	}
	return (
		<span
			data-slot='badge'
			data-variant={variant}
			className={cn(badgeVariants({ variant }), className)}
			{...props}>
			{children}
		</span>
	)
}

export { Badge, badgeVariants }
