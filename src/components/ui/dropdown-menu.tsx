import * as React from 'react'
import { createPortal } from 'react-dom'
import { CheckIcon, ChevronRightIcon, CircleIcon } from 'lucide-react'
import { cn } from '@/lib/utils'

// ─── Positioning ──────────────────────────────────────────────────────────────

type Align = 'start' | 'center' | 'end'

function calcPos(refRect: DOMRect, contentEl: HTMLElement, align: Align, sideOffset: number) {
	const { height: ch, width: cw } = contentEl.getBoundingClientRect()
	const vw = window.innerWidth
	const vh = window.innerHeight
	const spaceBelow = vh - refRect.bottom
	const top =
		spaceBelow >= ch + sideOffset ? refRect.bottom + sideOffset : refRect.top - ch - sideOffset

	let left: number
	if (align === 'start') left = refRect.left
	else if (align === 'end') left = refRect.right - cw
	else left = refRect.left + refRect.width / 2 - cw / 2

	left = Math.max(8, Math.min(left, vw - cw - 8))
	return { top, left }
}

// ─── Context ──────────────────────────────────────────────────────────────────

interface DropdownCtx {
	open: boolean
	setOpen: (v: boolean) => void
	triggerRef: React.RefObject<HTMLElement | null>
}
const Ctx = React.createContext<DropdownCtx | null>(null)
const useCtx = () => {
	const ctx = React.useContext(Ctx)
	if (!ctx) throw new Error('DropdownMenu components must be inside <DropdownMenu>')
	return ctx
}

// ─── Root ─────────────────────────────────────────────────────────────────────

function DropdownMenu({
	children,
	open: controlledOpen,
	onOpenChange,
	defaultOpen = false,
}: {
	children: React.ReactNode
	open?: boolean
	onOpenChange?: (open: boolean) => void
	defaultOpen?: boolean
}) {
	const [uncontrolledOpen, setUncontrolledOpen] = React.useState(defaultOpen)
	const isControlled = controlledOpen !== undefined
	const open = isControlled ? controlledOpen! : uncontrolledOpen
	const triggerRef = React.useRef<HTMLElement | null>(null)

	const setOpen = React.useCallback(
		(v: boolean) => {
			if (!isControlled) setUncontrolledOpen(v)
			onOpenChange?.(v)
		},
		[isControlled, onOpenChange]
	)

	return <Ctx.Provider value={{ open, setOpen, triggerRef }}>{children}</Ctx.Provider>
}

function DropdownMenuTrigger({
	children,
	asChild = false,
	...props
}: React.ComponentProps<'button'> & { asChild?: boolean }) {
	const { setOpen, open, triggerRef } = useCtx()
	const toggle = () => setOpen(!open)

	if (asChild && React.isValidElement(children)) {
		const child = children as React.ReactElement<React.HTMLAttributes<HTMLElement>>
		return React.cloneElement(child, {
			onClick: (e: React.MouseEvent) => {
				toggle()
				child.props.onClick?.(e as React.MouseEvent<HTMLElement>)
			},
			ref: (el: HTMLElement | null) => {
				;(triggerRef as React.RefObject<HTMLElement | null>).current = el
			},
		} as React.HTMLAttributes<HTMLElement>)
	}

	return (
		<button
			ref={triggerRef as React.RefObject<HTMLButtonElement>}
			data-slot='dropdown-menu-trigger'
			onClick={toggle}
			{...props}>
			{children}
		</button>
	)
}

function DropdownMenuPortal({ children }: { children: React.ReactNode }) {
	return <>{children}</>
}

function DropdownMenuContent({
	children,
	className,
	align = 'start',
	sideOffset = 4,
	...props
}: React.ComponentProps<'div'> & {
	align?: Align
	sideOffset?: number
}) {
	const { open, setOpen, triggerRef } = useCtx()
	const contentRef = React.useRef<HTMLDivElement>(null)
	const [pos, setPos] = React.useState<{ top: number; left: number } | null>(null)

	React.useLayoutEffect(() => {
		if (!open) {
			setPos(null)
			return
		}
		const trigger = triggerRef.current
		const content = contentRef.current
		if (!trigger || !content) return
		setPos(calcPos(trigger.getBoundingClientRect(), content, align, sideOffset))
	}, [open, align, sideOffset])

	React.useEffect(() => {
		if (!open) return
		const handler = (e: MouseEvent) => {
			if (contentRef.current?.contains(e.target as Node)) return
			if (triggerRef.current?.contains(e.target as Node)) return
			setOpen(false)
		}
		document.addEventListener('mousedown', handler)
		return () => document.removeEventListener('mousedown', handler)
	}, [open, setOpen])

	React.useEffect(() => {
		if (!open) return
		const handler = (e: KeyboardEvent) => {
			if (e.key === 'Escape') setOpen(false)
		}
		document.addEventListener('keydown', handler)
		return () => document.removeEventListener('keydown', handler)
	}, [open, setOpen])

	if (!open) return null

	return createPortal(
		<div
			ref={contentRef}
			data-slot='dropdown-menu-content'
			className={cn(
				'bg-popover text-popover-foreground z-50 min-w-[8rem] overflow-hidden rounded-md border p-1 shadow-md',
				'transition-[opacity,transform] duration-150',
				pos ? 'scale-100 opacity-100' : 'scale-95 opacity-0',
				className
			)}
			style={
				pos
					? { position: 'fixed', top: pos.top, left: pos.left }
					: { position: 'fixed', top: -9999, left: -9999 }
			}
			{...props}>
			{children}
		</div>,
		document.body
	)
}

function DropdownMenuGroup({ className, ...props }: React.ComponentProps<'div'>) {
	return <div data-slot='dropdown-menu-group' className={className} {...props} />
}

function DropdownMenuItem({
	className,
	inset,
	variant = 'default',
	...props
}: React.ComponentProps<'div'> & {
	inset?: boolean
	variant?: 'default' | 'destructive'
}) {
	return (
		<div
			data-slot='dropdown-menu-item'
			data-inset={inset}
			data-variant={variant}
			className={cn(
				"focus:bg-accent focus:text-accent-foreground relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
				inset && 'pl-8',
				variant === 'destructive' && 'text-destructive',
				className
			)}
			tabIndex={0}
			role='menuitem'
			onKeyDown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') (e.target as HTMLElement).click()
			}}
			{...props}
		/>
	)
}

function DropdownMenuCheckboxItem({
	className,
	children,
	checked,
	...props
}: React.ComponentProps<'div'> & { checked?: boolean }) {
	return (
		<div
			data-slot='dropdown-menu-checkbox-item'
			className={cn(
				"focus:bg-accent focus:text-accent-foreground relative flex cursor-default items-center gap-2 rounded-sm py-1.5 pr-2 pl-8 text-sm outline-hidden select-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
				className
			)}
			role='menuitemcheckbox'
			aria-checked={checked}
			tabIndex={0}
			{...props}>
			<span className='pointer-events-none absolute left-2 flex size-3.5 items-center justify-center'>
				{checked && <CheckIcon className='size-4' />}
			</span>
			{children}
		</div>
	)
}

function DropdownMenuRadioGroup({ ...props }: React.ComponentProps<'div'>) {
	return <div data-slot='dropdown-menu-radio-group' role='radiogroup' {...props} />
}

function DropdownMenuRadioItem({ className, children, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='dropdown-menu-radio-item'
			className={cn(
				'focus:bg-accent focus:text-accent-foreground relative flex cursor-default items-center gap-2 rounded-sm py-1.5 pr-2 pl-8 text-sm outline-hidden select-none [&_svg]:pointer-events-none [&_svg]:shrink-0',
				className
			)}
			role='menuitemradio'
			tabIndex={0}
			{...props}>
			<span className='pointer-events-none absolute left-2 flex size-3.5 items-center justify-center'>
				<CircleIcon className='size-2 fill-current' />
			</span>
			{children}
		</div>
	)
}

function DropdownMenuLabel({
	className,
	inset,
	...props
}: React.ComponentProps<'div'> & { inset?: boolean }) {
	return (
		<div
			data-slot='dropdown-menu-label'
			className={cn('px-2 py-1.5 text-sm font-medium', inset && 'pl-8', className)}
			{...props}
		/>
	)
}

function DropdownMenuSeparator({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='dropdown-menu-separator'
			className={cn('bg-border -mx-1 my-1 h-px', className)}
			{...props}
		/>
	)
}

function DropdownMenuShortcut({ className, ...props }: React.ComponentProps<'span'>) {
	return (
		<span
			data-slot='dropdown-menu-shortcut'
			className={cn('text-muted-foreground ml-auto text-xs tracking-widest', className)}
			{...props}
		/>
	)
}

function DropdownMenuSub({ children }: { children: React.ReactNode }) {
	return <>{children}</>
}

function DropdownMenuSubTrigger({
	className,
	inset,
	children,
	...props
}: React.ComponentProps<'div'> & { inset?: boolean }) {
	return (
		<div
			data-slot='dropdown-menu-sub-trigger'
			className={cn(
				'focus:bg-accent focus:text-accent-foreground flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none [&_svg]:pointer-events-none [&_svg]:shrink-0',
				inset && 'pl-8',
				className
			)}
			tabIndex={0}
			{...props}>
			{children}
			<ChevronRightIcon className='ml-auto size-4' />
		</div>
	)
}

function DropdownMenuSubContent({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='dropdown-menu-sub-content'
			className={cn(
				'bg-popover text-popover-foreground z-50 min-w-[8rem] overflow-hidden rounded-md border p-1 shadow-lg',
				className
			)}
			{...props}
		/>
	)
}

export {
	DropdownMenu,
	DropdownMenuPortal,
	DropdownMenuTrigger,
	DropdownMenuContent,
	DropdownMenuGroup,
	DropdownMenuLabel,
	DropdownMenuItem,
	DropdownMenuCheckboxItem,
	DropdownMenuRadioGroup,
	DropdownMenuRadioItem,
	DropdownMenuSeparator,
	DropdownMenuShortcut,
	DropdownMenuSub,
	DropdownMenuSubTrigger,
	DropdownMenuSubContent,
}
