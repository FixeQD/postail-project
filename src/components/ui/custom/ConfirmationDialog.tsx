import { motion, AnimatePresence } from 'framer-motion'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import type { ConfirmationDialogProps } from '@/types/components/ui'

export function ConfirmationDialog({
	open,
	onOpenChange,
	title,
	description,
	confirmLabel,
	cancelLabel,
	onConfirm,
	children,
	confirmClassName,
}: ConfirmationDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className='border-white/[0.06] bg-zinc-950 p-0 text-zinc-100 shadow-2xl sm:max-w-[400px]'>
				<AnimatePresence>
					{open && (
						<motion.div
							key='confirmation-dialog'
							initial={{ opacity: 0, scale: 0.94, y: -10 }}
							animate={{ opacity: 1, scale: 1, y: 0 }}
							transition={{
								duration: 0.2,
								ease: 'circOut',
							}}>
							<motion.div
								initial={{ scaleX: 0, opacity: 0 }}
								animate={{ scaleX: 1, opacity: 1 }}
								transition={{ duration: 0.35, ease: 'circOut' }}
								className='h-[3px] w-full origin-left rounded-t-lg'
								style={{
									background: `linear-gradient(90deg, var(--accent-color), var(--accent-light))`,
								}}
							/>

							<div className='px-6 pt-5 pb-6'>
								<DialogHeader className='mb-4'>
									<motion.div
										initial={{ opacity: 0, y: 6 }}
										animate={{ opacity: 1, y: 0 }}
										transition={{
											duration: 0.22,
											delay: 0.06,
											ease: 'circOut',
										}}>
										<DialogTitle className='text-base font-semibold tracking-tight text-zinc-100'>
											{title}
										</DialogTitle>
									</motion.div>
									<motion.div
										initial={{ opacity: 0, y: 6 }}
										animate={{ opacity: 1, y: 0 }}
										transition={{
											duration: 0.22,
											delay: 0.12,
											ease: 'circOut',
										}}>
										<DialogDescription className='mt-1.5 text-sm leading-relaxed text-zinc-400'>
											{description}
										</DialogDescription>
									</motion.div>
								</DialogHeader>

								{children && (
									<motion.div
										initial={{ opacity: 0, y: 6 }}
										animate={{ opacity: 1, y: 0 }}
										transition={{
											duration: 0.22,
											delay: 0.18,
											ease: 'circOut',
										}}
										className='mt-4'>
										{children}
									</motion.div>
								)}

								<DialogFooter className='mt-6 flex gap-2.5 sm:gap-2.5'>
									<motion.div
										initial={{ opacity: 0, y: 6 }}
										animate={{ opacity: 1, y: 0 }}
										transition={{
											duration: 0.22,
											delay: 0.24,
											ease: 'circOut',
										}}
										className='flex-1'>
										<motion.div
											whileHover={{ scale: 1.02 }}
											whileTap={{ scale: 0.97 }}
											className='w-full'>
											<Button
												variant='ghost'
												onClick={() => onOpenChange(false)}
												className='w-full border border-white/[0.06] bg-white/[0.03] text-zinc-400 transition-colors hover:border-white/10 hover:bg-white/[0.07] hover:text-zinc-200'>
												{cancelLabel}
											</Button>
										</motion.div>
									</motion.div>

									<motion.div
										initial={{ opacity: 0, y: 6 }}
										animate={{ opacity: 1, y: 0 }}
										transition={{ duration: 0.22, delay: 0.3, ease: 'circOut' }}
										className='flex-1'>
										<motion.div
											whileHover={{ scale: 1.02 }}
											whileTap={{ scale: 0.97 }}
											className='w-full'>
											<Button
												onClick={onConfirm}
												className={
													confirmClassName ||
													'w-full border-0 font-medium text-white shadow-lg'
												}
												style={
													confirmClassName
														? undefined
														: {
																background: `linear-gradient(135deg, var(--accent-color), var(--accent-dark))`,
																boxShadow: `0 4px 20px rgba(var(--accent-rgb), 0.3)`,
																color: 'var(--accent-text)',
															}
												}>
												{confirmLabel}
											</Button>
										</motion.div>
									</motion.div>
								</DialogFooter>
							</div>
						</motion.div>
					)}
				</AnimatePresence>
			</DialogContent>
		</Dialog>
	)
}
