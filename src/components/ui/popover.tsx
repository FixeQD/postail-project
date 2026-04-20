import * as React from 'react'
import { createPortal } from 'react-dom'
import { cn } from '@/lib/utils'

// ─── Positioning ──────────────────────────────────────────────────────────────

type Align = 'start' | 'center' | 'end'

function calcPos(
	refRect: DOMRect,
	contentEl: HTMLElement,
	align: Align,
	sideOffset: number
): { top: number; left: number } {
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

interface PopoverCtx {
	open: boolean
	setOpen: (v: boolean) => void
	triggerRef: React.RefObject<HTMLElement | null>
	anchorRef: React.RefObject<HTMLElement | null>
}
const Ctx = React.createContext<PopoverCtx | null>(null)
const useCtx = () => {
	const ctx = React.useContext(Ctx)
	if (!ctx) throw new Error('Popover components must be inside <Popover>')
	return ctx
}

// ─── Components ───────────────────────────────────────────────────────────────

function Popover({
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
	const anchorRef = React.useRef<HTMLElement | null>(null)

	const setOpen = React.useCallback(
		(v: boolean) => {
			if (!isControlled) setUncontrolledOpen(v)
			onOpenChange?.(v)
		},
		[isControlled, onOpenChange]
	)

	return <Ctx.Provider value={{ open, setOpen, triggerRef, anchorRef }}>{children}</Ctx.Provider>
}

function PopoverTrigger({
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
			data-slot='popover-trigger'
			onClick={toggle}
			{...props}>
			{children}
		</button>
	)
}

function PopoverAnchor({ className, ...props }: React.ComponentProps<'div'>) {
	const { anchorRef } = useCtx()
	return (
		<div ref={anchorRef as React.RefObject<HTMLDivElement>} className={className} {...props} />
	)
}

function PopoverContent({
	children,
	className,
	align = 'center',
	sideOffset = 4,
	...props
}: React.ComponentProps<'div'> & {
	align?: Align
	sideOffset?: number
}) {
	const { open, setOpen, triggerRef, anchorRef } = useCtx()
	const contentRef = React.useRef<HTMLDivElement>(null)
	const [pos, setPos] = React.useState<{ top: number; left: number } | null>(null)

	React.useLayoutEffect(() => {
		if (!open) {
			setPos(null)
			return
		}
		const refEl = anchorRef.current ?? triggerRef.current
		const content = contentRef.current
		if (!refEl || !content) return
		setPos(calcPos(refEl.getBoundingClientRect(), content, align, sideOffset))
	}, [open, align, sideOffset])

	// Close on outside click
	React.useEffect(() => {
		if (!open) return
		const handler = (e: MouseEvent) => {
			const trigger = triggerRef.current
			const content = contentRef.current
			if (trigger?.contains(e.target as Node) || content?.contains(e.target as Node)) return
			setOpen(false)
		}
		document.addEventListener('mousedown', handler)
		return () => document.removeEventListener('mousedown', handler)
	}, [open, setOpen])

	// Close on Escape
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
			data-slot='popover-content'
			className={cn(
				'bg-popover text-popover-foreground z-50 w-72 rounded-md border p-4 shadow-md outline-hidden',
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

function PopoverHeader({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='popover-header'
			className={cn('flex flex-col gap-1 text-sm', className)}
			{...props}
		/>
	)
}

function PopoverTitle({ className, ...props }: React.ComponentProps<'div'>) {
	return <div data-slot='popover-title' className={cn('font-medium', className)} {...props} />
}

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
