import { AnimatePresence, motion } from 'framer-motion'
import {
	CircleCheckIcon,
	InfoIcon,
	Loader2Icon,
	OctagonXIcon,
	TriangleAlertIcon,
	XIcon,
} from 'lucide-react'
import { useEffect, useRef, useState } from 'react'

import { useToastStore, type Toast, type ToastType } from '@/stores/toastStore'
export { toast } from '@/stores/toastStore'

const icons: Record<ToastType, React.ReactNode> = {
	success: <CircleCheckIcon className='size-5 text-status-success' />,
	error: <OctagonXIcon className='size-5 text-destructive' />,
	info: <InfoIcon className='size-5 text-status-info' />,
	warning: <TriangleAlertIcon className='size-5 text-status-warning' />,
	loading: <Loader2Icon className='size-5 animate-spin text-[var(--text-secondary)]' />,
}

// ── Live ticking seconds display ──────────────────────────────────────────────

function LiveCountdown({ durationMs }: { durationMs: number }) {
	const [secs, setSecs] = useState(Math.ceil(durationMs / 1000))

	useEffect(() => {
		const id = setInterval(() => setSecs((s) => Math.max(0, s - 1)), 1000)
		return () => clearInterval(id)
	}, [])

	return (
		<>
			Sending in <span className='text-[var(--text-primary)] tabular-nums'>{secs}</span>s…
		</>
	)
}

// ── Countdown progress bar ────────────────────────────────────────────────────

function CountdownBar({ duration }: { duration: number }) {
	const barRef = useRef<HTMLDivElement>(null)

	useEffect(() => {
		const el = barRef.current
		if (!el) return
		const anim = el.animate([{ transform: 'scaleX(1)' }, { transform: 'scaleX(0)' }], {
			duration,
			easing: 'linear',
			fill: 'forwards',
		})
		return () => anim.cancel()
	}, [duration])

	return (
		<div className='absolute right-0 bottom-0 left-0 h-[3px] overflow-hidden rounded-b-2xl bg-[var(--border-faint)]'>
			<div
				ref={barRef}
				className='h-full w-full origin-left bg-gradient-to-r from-blue-500 to-cyan-400'
			/>
		</div>
	)
}

// ── Single toast item ─────────────────────────────────────────────────────────

function ToastItem({ t, onRemove }: { t: Toast; onRemove: (id: string) => void }) {
	const isCountdown = !!(t.withCountdown && t.cancelFn && t.duration)

	return (
		<motion.div
			key={t.id}
			layout='position'
			initial={{ opacity: 0, scale: 0.9, y: 20 }}
			animate={{ opacity: 1, scale: 1, y: 0 }}
			exit={{ opacity: 0, scale: 0.85, y: 10, transition: { duration: 0.15 } }}
			transition={{ type: 'spring', stiffness: 400, damping: 30, restDelta: 0.01 }}
			style={{ willChange: 'transform, opacity' }}
			className='glass pointer-events-auto relative max-w-md min-w-[320px] cursor-default overflow-hidden rounded-2xl border border-[var(--border-subtle)] p-4 shadow-2xl transition-shadow select-none'>
			<div
				className='absolute inset-0 -z-10 opacity-20'
				style={{
					background: `radial-gradient(circle at top left, var(--accent-color, #eb6226), transparent 70%)`,
				}}
			/>

			<div className='flex items-start gap-3'>
				<div className='mt-0.5 shrink-0'>{icons[t.type]}</div>
				<div className='flex-1 overflow-hidden'>
					<p className='text-[15px] leading-tight font-medium text-[var(--text-primary)]'>
						{isCountdown ? <LiveCountdown durationMs={t.duration!} /> : t.message}
					</p>
					{t.description && (
						<p className='mt-1 text-sm text-[var(--text-secondary)]'>{t.description}</p>
					)}
				</div>

				{isCountdown && (
					<button
						onClick={() => {
							t.cancelFn!()
							onRemove(t.id)
						}}
						className='ml-1 shrink-0 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-hover)] px-2.5 py-1 text-xs font-semibold text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-active)] hover:text-[var(--text-primary)]'>
						Undo
					</button>
				)}

				<button
					onClick={() => onRemove(t.id)}
					className='-mt-1 -mr-1 ml-2 rounded-full p-1 text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-secondary)]'>
					<XIcon className='size-4' />
				</button>
			</div>

			{isCountdown && <CountdownBar duration={t.duration!} />}
		</motion.div>
	)
}

// ── Toaster container ─────────────────────────────────────────────────────────

export function Toaster() {
	const { toasts, removeToast } = useToastStore()

	return (
		<div className='pointer-events-none fixed right-6 bottom-6 z-[9999] flex flex-col gap-3 overflow-visible p-4'>
			<AnimatePresence mode='popLayout'>
				{toasts.map((t) => (
					<ToastItem key={t.id} t={t} onRemove={removeToast} />
				))}
			</AnimatePresence>
		</div>
	)
}
