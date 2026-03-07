import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogDescription,
} from '@/components/ui/dialog'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { MailboxRoleStep } from './MailboxRoleStep'

interface MailboxRoleDialogProps {
	accountId: string | null
	onDone: () => void
}

export function MailboxRoleDialog({ accountId, onDone }: MailboxRoleDialogProps) {
	const { t } = useSettingsTranslation()

	return (
		<Dialog open={!!accountId} onOpenChange={(open) => !open && onDone()}>
			<DialogContent className='overflow-hidden border-[var(--border-subtle)] bg-[var(--surface-glass)] p-0 text-[var(--text-primary)] backdrop-blur-xl sm:max-w-lg'>
				<DialogHeader className='sr-only'>
					<DialogTitle>{t('settings:mailboxRoles.dialogTitle')}</DialogTitle>
					<DialogDescription>
						{t('settings:mailboxRoles.dialogDescription')}
					</DialogDescription>
				</DialogHeader>
				{accountId && (
					<div className='px-8 py-6'>
						<MailboxRoleStep accountId={accountId} onDone={onDone} />
					</div>
				)}
			</DialogContent>
		</Dialog>
	)
}
