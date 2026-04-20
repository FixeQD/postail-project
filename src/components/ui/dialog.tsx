import * as React from 'react'
import { createPortal } from 'react-dom'
import { XIcon } from 'lucide-react'
import { cn } from '@/lib/utils'

// ─── Context ──────────────────────────────────────────────────────────────────

interface DialogCtx {
	open: boolean
	setOpen: (v: boolean) => void
}
const Ctx = React.createContext<DialogCtx | null>(null)
const useCtx = () => {
	const ctx = React.useContext(Ctx)
	if (!ctx) throw new Error('Dialog components must be inside <Dialog>')
	return ctx
}

// ─── Root ─────────────────────────────────────────────────────────────────────

function Dialog({
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

	const setOpen = React.useCallback(
		(v: boolean) => {
			if (!isControlled) setUncontrolledOpen(v)
			onOpenChange?.(v)
		},
		[isControlled, onOpenChange]
	)

	return <Ctx.Provider value={{ open, setOpen }}>{children}</Ctx.Provider>
}

function DialogTrigger({
	children,
	asChild = false,
	...props
}: React.ComponentProps<'button'> & { asChild?: boolean }) {
	const { setOpen, open } = useCtx()
	const toggle = () => setOpen(!open)

	if (asChild && React.isValidElement(children)) {
		const child = children as React.ReactElement<React.HTMLAttributes<HTMLElement>>
		return React.cloneElement(child, {
			onClick: (e: React.MouseEvent) => {
				toggle()
				child.props.onClick?.(e as React.MouseEvent<HTMLElement>)
			},
		} as React.HTMLAttributes<HTMLElement>)
	}

	return (
		<button data-slot='dialog-trigger' onClick={toggle} {...props}>
			{children}
		</button>
	)
}

// No-op wrappers for API compat
function DialogPortal({ children }: { children: React.ReactNode }) {
	return <>{children}</>
}
function DialogClose({
	children,
	asChild = false,
	...props
}: React.ComponentProps<'button'> & { asChild?: boolean }) {
	const { setOpen } = useCtx()
	const close = () => setOpen(false)

	if (asChild && React.isValidElement(children)) {
		const child = children as React.ReactElement<React.HTMLAttributes<HTMLElement>>
		return React.cloneElement(child, {
			onClick: (e: React.MouseEvent) => {
				close()
				child.props.onClick?.(e as React.MouseEvent<HTMLElement>)
			},
		} as React.HTMLAttributes<HTMLElement>)
	}
	return (
		<button data-slot='dialog-close' onClick={close} {...props}>
			{children}
		</button>
	)
}

// ─── Overlay ──────────────────────────────────────────────────────────────────

function DialogOverlay({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='dialog-overlay'
			className={cn('fixed inset-0 z-50 bg-black/50', className)}
			{...props}
		/>
	)
}

// ─── Content ──────────────────────────────────────────────────────────────────

function DialogContent({
	children,
	className,
	showCloseButton = true,
	...props
}: React.ComponentProps<'div'> & { showCloseButton?: boolean }) {
	const { open, setOpen } = useCtx()
	const contentRef = React.useRef<HTMLDivElement>(null)
	const [visible, setVisible] = React.useState(false)
	const [animIn, setAnimIn] = React.useState(false)

	React.useEffect(() => {
		if (open) {
			setVisible(true)
			const f = requestAnimationFrame(() => setAnimIn(true))
			return () => cancelAnimationFrame(f)
		} else {
			setAnimIn(false)
			const t = setTimeout(() => setVisible(false), 200)
			return () => clearTimeout(t)
		}
	}, [open])

	// Scroll lock
	React.useEffect(() => {
		if (!open) return
		const prev = document.body.style.overflow
		document.body.style.overflow = 'hidden'
		return () => {
			document.body.style.overflow = prev
		}
	}, [open])

	// Escape key
	React.useEffect(() => {
		if (!open) return
		const handler = (e: KeyboardEvent) => {
			if (e.key === 'Escape') setOpen(false)
		}
		document.addEventListener('keydown', handler)
		return () => document.removeEventListener('keydown', handler)
	}, [open, setOpen])

	// Focus trap
	React.useEffect(() => {
		if (!open) return
		const el = contentRef.current
		if (!el) return
		const getFocusable = () =>
			Array.from(
				el.querySelectorAll<HTMLElement>(
					'a[href], button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
				)
			)
		getFocusable()[0]?.focus()
		const trap = (e: KeyboardEvent) => {
			if (e.key !== 'Tab') return
			const els = getFocusable()
			if (!els.length) return
			const first = els[0],
				last = els[els.length - 1]
			if (e.shiftKey) {
				if (document.activeElement === first) {
					e.preventDefault()
					last?.focus()
				}
			} else {
				if (document.activeElement === last) {
					e.preventDefault()
					first?.focus()
				}
			}
		}
		document.addEventListener('keydown', trap)
		return () => document.removeEventListener('keydown', trap)
	}, [open])

	if (!visible) return null

	return createPortal(
		<>
			<div
				className='fixed inset-0 z-50 bg-black/50 transition-opacity duration-200'
				style={{ opacity: animIn ? 1 : 0 }}
				onClick={() => setOpen(false)}
			/>
			<div
				ref={contentRef}
				data-slot='dialog-content'
				className={cn(
					'fixed top-1/2 left-1/2 z-50 grid w-[calc(100%-2rem)] max-w-lg -translate-x-1/2 -translate-y-1/2 place-items-center gap-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-glass)] p-6 text-[var(--text-primary)] shadow-lg outline-none',
					'transition-[opacity,transform] duration-200',
					className
				)}
				style={{
					opacity: animIn ? 1 : 0,
				}}
				{...props}>
				{children}
				{showCloseButton && (
					<button
						data-slot='dialog-close'
						onClick={() => setOpen(false)}
						className='absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 [&_svg]:size-4'>
						<XIcon />
						<span className='sr-only'>Close</span>
					</button>
				)}
			</div>
		</>,
		document.body
	)
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function DialogHeader({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='dialog-header'
			className={cn('flex flex-col gap-2 text-center sm:text-left', className)}
			{...props}
		/>
	)
}

function DialogFooter({ className, ...props }: React.ComponentProps<'div'>) {
	return (
		<div
			data-slot='dialog-footer'
			className={cn('flex flex-col-reverse gap-2 sm:flex-row sm:justify-end', className)}
			{...props}
		/>
	)
}

function DialogTitle({ className, ...props }: React.ComponentProps<'h2'>) {
	return (
		<h2
			data-slot='dialog-title'
			className={cn('text-lg leading-none font-semibold', className)}
			{...props}
		/>
	)
}

function DialogDescription({ className, ...props }: React.ComponentProps<'p'>) {
	return (
		<p
			data-slot='dialog-description'
			className={cn('text-muted-foreground text-sm', className)}
			{...props}
		/>
	)
}

export {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogOverlay,
	DialogPortal,
	DialogTitle,
	DialogTrigger,
}
