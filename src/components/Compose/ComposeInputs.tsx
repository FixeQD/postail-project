import { useCallback } from 'react'
import { X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { AddressInput } from './Inputs/AddressInput'
import { SubjectInput } from './Inputs/SubjectInput'
import { useDraftStore } from '@/stores/draftStore'
import type { EmailAddress, ComposeDraft } from '@/types/compose'

interface ComposeInputsProps {
	to: EmailAddress[]
	cc: EmailAddress[]
	bcc: EmailAddress[]
	subject: string
	showCc: boolean
	showBcc: boolean
	setShowCc: (show: boolean) => void
	setShowBcc: (show: boolean) => void
	onUpdate: (updates: Partial<ComposeDraft>) => void
	onAddRecipient: (type: 'to' | 'cc' | 'bcc', recipient: EmailAddress) => void
	onRemoveRecipient: (type: 'to' | 'cc' | 'bcc', email: string) => void
}

export function ComposeInputs({
	to,
	cc,
	bcc,
	subject,
	showCc,
	showBcc,
	setShowCc,
	setShowBcc,
	onUpdate,
	onAddRecipient,
	onRemoveRecipient,
}: ComposeInputsProps) {
	const { t } = useTranslation()

	const handleToggleCc = useCallback(() => setShowCc(!showCc), [showCc, setShowCc])
	const handleToggleBcc = useCallback(() => setShowBcc(!showBcc), [showBcc, setShowBcc])

	const { setSubject } = useDraftStore()

	return (
		<div className='flex flex-col px-4 pt-1'>
			<AddressInput
				label={t('compose.to')}
				recipients={to}
				onAdd={(recipient) => onAddRecipient('to', recipient)}
				onRemove={(email) => onRemoveRecipient('to', email)}
				placeholder={t('compose.recipients')}
				rightElement={
					<div className='mr-2 flex gap-2'>
						{!showCc && (
							<button
								type='button'
								onClick={handleToggleCc}
								className='text-xs font-medium text-zinc-500 transition-colors hover:text-zinc-300'>
								{t('compose.cc')}
							</button>
						)}
						{!showBcc && (
							<button
								type='button'
								onClick={handleToggleBcc}
								className='text-xs font-medium text-zinc-500 transition-colors hover:text-zinc-300'>
								{t('compose.bcc')}
							</button>
						)}
					</div>
				}
			/>
			{showCc && (
				<AddressInput
					label={t('compose.cc')}
					recipients={cc}
					onAdd={(recipient) => onAddRecipient('cc', recipient)}
					onRemove={(email) => onRemoveRecipient('cc', email)}
					rightElement={
						<button
							type='button'
							onClick={() => {
								setShowCc(false)
								onUpdate({ cc: [] })
							}}
							className='mr-2 text-zinc-500 transition-colors hover:text-zinc-300'>
							<X className='h-3.5 w-3.5' />
						</button>
					}
				/>
			)}
			{showBcc && (
				<AddressInput
					label={t('compose.bcc')}
					recipients={bcc}
					onAdd={(recipient) => onAddRecipient('bcc', recipient)}
					onRemove={(email) => onRemoveRecipient('bcc', email)}
					rightElement={
						<button
							type='button'
							onClick={() => {
								setShowBcc(false)
								onUpdate({ bcc: [] })
							}}
							className='mr-2 text-zinc-500 transition-colors hover:text-zinc-300'>
							<X className='h-3.5 w-3.5' />
						</button>
					}
				/>
			)}
			<SubjectInput
				placeholder={t('compose.subject')}
				value={subject}
				onChange={setSubject}
			/>
		</div>
	)
}
