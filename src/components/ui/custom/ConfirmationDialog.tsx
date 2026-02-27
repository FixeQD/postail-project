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
}: ConfirmationDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className='border-zinc-800 bg-zinc-900 text-zinc-100 sm:max-w-[425px]'>
				<DialogHeader>
					<DialogTitle>{title}</DialogTitle>
					<DialogDescription className='text-zinc-400'>{description}</DialogDescription>
				</DialogHeader>
				<DialogFooter className='gap-2 sm:gap-0'>
					<Button
						variant='ghost'
						onClick={() => onOpenChange(false)}
						className='text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'>
						{cancelLabel}
					</Button>
					<Button onClick={onConfirm} className='bg-blue-600 hover:bg-blue-500'>
						{confirmLabel}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
