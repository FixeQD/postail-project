import { AnimatePresence, motion } from 'framer-motion'
import {
	CircleCheckIcon,
	InfoIcon,
	Loader2Icon,
	OctagonXIcon,
	TriangleAlertIcon,
	XIcon,
} from 'lucide-react'

import { useToastStore, type ToastType } from '../../../stores/toastStore'
export { toast } from '../../../stores/toastStore'

const icons: Record<ToastType, React.ReactNode> = {
	success: <CircleCheckIcon className='size-5 text-emerald-400' />,
	error: <OctagonXIcon className='size-5 text-rose-400' />,
	info: <InfoIcon className='size-5 text-blue-400' />,
	warning: <TriangleAlertIcon className='size-5 text-amber-400' />,
	loading: <Loader2Icon className='size-5 animate-spin text-slate-400' />,
}

export function Toaster() {
	const { toasts, removeToast } = useToastStore()

	return (
		<div className='pointer-events-none fixed bottom-6 right-6 z-[9999] flex flex-col gap-3 overflow-visible p-4'>
			<AnimatePresence mode='popLayout'>
				{toasts.map((t) => (
					<motion.div
						key={t.id}
						layout='position'
						initial={{ opacity: 0, scale: 0.9, y: 20, filter: 'blur(8px)' }}
						animate={{ opacity: 1, scale: 1, y: 0, filter: 'blur(0px)' }}
						exit={{ opacity: 0, scale: 0.85, y: 10, transition: { duration: 0.15 } }}
						transition={{
							type: 'spring',
							stiffness: 400,
							damping: 30,
							restDelta: 0.01,
						}}
						style={{ willChange: 'transform, opacity, filter' }}
						className='pointer-events-auto relative min-w-[320px] max-w-md cursor-default select-none overflow-hidden rounded-2xl border border-white/[0.08] bg-slate-900/80 p-4 shadow-2xl backdrop-blur-2xl transition-shadow hover:shadow-white/[0.02]'>
						<div
							className='absolute inset-0 -z-10 opacity-20'
							style={{
								background: `radial-gradient(circle at top left, var(--accent-color, #f97316), transparent 70%)`,
							}}
						/>

						<div className='flex items-start gap-3'>
							<div className='mt-0.5 shrink-0'>{icons[t.type]}</div>
							<div className='flex-1 overflow-hidden'>
								<p className='text-[15px] font-medium leading-tight text-white/95'>
									{t.message}
								</p>
								{t.description && (
									<p className='mt-1 text-sm text-white/60'>{t.description}</p>
								)}
							</div>
							<button
								onClick={() => removeToast(t.id)}
								className='-mr-1 -mt-1 ml-2 rounded-full p-1 text-white/20 transition-colors hover:bg-white/5 hover:text-white/40'>
								<XIcon className='size-4' />
							</button>
						</div>
					</motion.div>
				))}
			</AnimatePresence>
		</div>
	)
}
