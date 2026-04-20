import * as React from 'react'
import { createPortal } from 'react-dom'
import { cn } from '@/lib/utils'

// ─── Positioning ──────────────────────────────────────────────────────────────

function calcPos(
	triggerRect: DOMRect,
	contentEl: HTMLElement,
	sideOffset: number
): { top: number; left: number } {
	const { height: ch, width: cw } = contentEl.getBoundingClientRect()
	const vw = window.innerWidth
	const vh = window.innerHeight
	const spaceBelow = vh - triggerRect.bottom
	const top =
		spaceBelow >= ch + sideOffset
			? triggerRect.bottom + sideOffset
			: triggerRect.top - ch - sideOffset
	let left = triggerRect.left + triggerRect.width / 2 - cw / 2
	left = Math.max(8, Math.min(left, vw - cw - 8))
	return { top, left }
}

// ─── Context ──────────────────────────────────────────────────────────────────

interface TooltipCtx {
	open: boolean
	setOpen: (v: boolean) => void
	triggerRef: React.RefObject<HTMLElement | null>
}
const Ctx = React.createContext<TooltipCtx | null>(null)
const useCtx = () => {
	const ctx = React.useContext(Ctx)
	if (!ctx) throw new Error('Tooltip components must be used inside <Tooltip>')
	return ctx
}

// ─── Components ───────────────────────────────────────────────────────────────

function TooltipProvider({ children }: { children: React.ReactNode; delayDuration?: number }) {
	return <>{children}</>
}

function Tooltip({
	children,
	open: controlledOpen,
	defaultOpen = false,
	onOpenChange,
}: {
	children: React.ReactNode
	open?: boolean
	defaultOpen?: boolean
	onOpenChange?: (open: boolean) => void
}) {
	const [uncontrolledOpen, setUncontrolledOpen] = React.useState(defaultOpen)
	const isControlled = controlledOpen !== undefined
	const open = isControlled ? controlledOpen : uncontrolledOpen
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

function TooltipTrigger({
	children,
	asChild = false,
	...props
}: React.ComponentProps<'button'> & { asChild?: boolean }) {
	const { setOpen, triggerRef } = useCtx()
	let delay: ReturnType<typeof setTimeout> | null = null

	const handlers = {
		onMouseEnter: () => {
			delay = setTimeout(() => setOpen(true), 500)
		},
		onMouseLeave: () => {
			if (delay) clearTimeout(delay)
			setOpen(false)
		},
		onFocus: () => setOpen(true),
		onBlur: () => setOpen(false),
	}

	if (asChild && React.isValidElement(children)) {
		const child = children as React.ReactElement<React.HTMLAttributes<HTMLElement>>
		return React.cloneElement(child, {
			...handlers,
			ref: (el: HTMLElement | null) => {
				;(triggerRef as React.RefObject<HTMLElement | null>).current = el
			},
		} as React.HTMLAttributes<HTMLElement>)
	}

	return (
		<button
			ref={triggerRef as React.RefObject<HTMLButtonElement>}
			data-slot='tooltip-trigger'
			{...handlers}
			{...props}>
			{children}
		</button>
	)
}

function TooltipContent({
	children,
	className,
	sideOffset = 0,
	...props
}: React.ComponentProps<'div'> & { sideOffset?: number }) {
	const { open, triggerRef } = useCtx()
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
		setPos(calcPos(trigger.getBoundingClientRect(), content, sideOffset))
	}, [open, sideOffset])

	if (!open) return null

	return createPortal(
		<div
			ref={contentRef}
			data-slot='tooltip-content'
			className={cn(
				'bg-foreground text-background z-50 w-fit rounded-md px-3 py-1.5 text-xs text-balance',
				'transition-opacity duration-100',
				pos ? 'opacity-100' : 'opacity-0',
				className
			)}
			style={
				pos
					? { position: 'fixed', top: pos.top, left: pos.left, pointerEvents: 'none' }
					: { position: 'fixed', top: -9999, left: -9999, pointerEvents: 'none' }
			}
			{...props}>
			{children}
		</div>,
		document.body
	)
}

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider }
