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
			<DialogContent className='border-slate-800 bg-slate-900/95 p-6 text-slate-100 backdrop-blur-xl'>
				<DialogHeader className='sr-only'>
					<DialogTitle>{t('settings:mailboxRoles.dialogTitle')}</DialogTitle>
					<DialogDescription>
						{t('settings:mailboxRoles.dialogDescription')}
					</DialogDescription>
				</DialogHeader>
				{accountId && <MailboxRoleStep accountId={accountId} onDone={onDone} />}
			</DialogContent>
		</Dialog>
	)
}
