import { memo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { PenLine, Check } from 'lucide-react'
import { useAccountStore } from '@/stores/accountStore'
import { useDraftStore } from '@/stores/draftStore'
import type { Signature } from '@/types/signatures'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Button } from '@/components/ui/button'

interface SignatureSelectorProps {
	htmlRef?: React.MutableRefObject<string>
}

export const SignatureSelector = memo(function SignatureSelector({ htmlRef }: SignatureSelectorProps) {
	const { t } = useTranslation()
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const accountId = activeAccount?.id ?? ''
	const [open, setOpen] = useState(false)
	const { currentDraft, replaceSignature } = useDraftStore()

	const { data: signatures = [] } = useQuery<Signature[]>({
		queryKey: ['signatures', accountId],
		queryFn: () => invoke<Signature[]>('list_signatures', { accountId }),
		enabled: !!accountId,
	})

	const getCurrentSignatureHtml = () => {
		if (!currentDraft?.body) return null
		const match = currentDraft.body.match(/<div class="signature">([\s\S]*?)<\/div>/i)
		return match ? match[1] : null
	}

	const currentSigHtml = getCurrentSignatureHtml()

	const isSelected = (sig: Signature | null) => {
		if (!sig && !currentSigHtml) return true
		if (!sig || !currentSigHtml) return false
		return sig.htmlContent === currentSigHtml
	}

	const handleSelect = (html: string | null) => {
		replaceSignature(html)
		setOpen(false)
		setTimeout(() => {
			const latestDraft = useDraftStore.getState().currentDraft
			if (latestDraft?.body !== undefined && htmlRef) {
				htmlRef.current = latestDraft.body
			}
		}, 50)
	}

	const hasSignature = currentSigHtml && currentSigHtml.trim().length > 0

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button
					variant='ghost'
					size='icon'
					className={`h-9 w-9 text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)] ${
						hasSignature ? 'text-blue-500' : ''
					}`}
					title={t('compose.signature.select')}>
					<PenLine className='h-4 w-4' />
				</Button>
			</PopoverTrigger>
			<PopoverContent align='end' className='w-56 p-1' sideOffset={8}>
				<div className='py-1'>
					<button
						type='button'
						onClick={() => handleSelect(null)}
						className='flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--surface-hover)]'>
						<span>{t('compose.signature.noSignature')}</span>
						{isSelected(null) && <Check className='h-4 w-4 text-blue-500' />}
					</button>

					{signatures.length === 0 && (
						<div className='px-3 py-2 text-xs text-[var(--text-tertiary)]'>
							{t('compose.signature.noSignatures')}
						</div>
					)}

					{signatures.map((sig) => (
						<button
							key={sig.id}
							type='button'
							onClick={() => handleSelect(sig.htmlContent)}
							className='flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] hover:bg-[var(--surface-hover)]'>
							<span className='truncate'>{sig.name}</span>
							{isSelected(sig) && <Check className='h-4 w-4 text-blue-500' />}
						</button>
					))}
				</div>
			</PopoverContent>
		</Popover>
	)
})
