import { memo } from 'react'
import { useTranslation } from 'react-i18next'
import { Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useDraftStore } from '@/stores/draftStore'
import EditorToolbar from './Editor/EditorToolbar'
import type { ComposeFooterProps } from '@/types/components/compose'

export const ComposeFooter = memo(({ onSend, onDiscard, isValid }: ComposeFooterProps) => {
	const { t } = useTranslation()
	const { isSaving, isSending } = useDraftStore()

	return (
		<div className='mt-auto border-t border-[var(--compose-input-border)] bg-[var(--compose-footer-bg)] p-3'>
			<div className='flex items-center justify-between'>
				<div className='flex items-center gap-1'>
					<Button
						onClick={onSend}
						className='h-9 rounded-full bg-blue-600 px-6 font-semibold text-white hover:bg-blue-500'
						disabled={isSaving || isSending || !isValid}
						title={!isValid ? t('compose.validation.missingFields') : ''}>
						{isSaving ? '...' : t('actions.send')}
					</Button>
					<EditorToolbar />
				</div>

				<div className='flex items-center gap-1'>
					<Button
						variant='ghost'
						size='icon'
						className='h-9 w-9 text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)] hover:text-red-400'
						onClick={onDiscard}>
						<Trash2 className='h-4 w-4' />
					</Button>
				</div>
			</div>
		</div>
	)
})
