import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'

interface ConfirmationDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	title: string
	description: string
	confirmLabel: string
	cancelLabel: string
	onConfirm: () => void
}

/**
 * Renders a modal confirmation dialog with a title, description, and customizable confirm/cancel actions.
 *
 * @param open - Whether the dialog is currently open
 * @param onOpenChange - Callback invoked with the new open state when the dialog is opened or closed
 * @param title - Primary title text displayed in the dialog header
 * @param description - Supplemental descriptive text displayed under the title
 * @param confirmLabel - Label for the confirm action button
 * @param cancelLabel - Label for the cancel action button
 * @param onConfirm - Callback invoked when the confirm button is clicked
 * @returns A React element representing the confirmation dialog
 */
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