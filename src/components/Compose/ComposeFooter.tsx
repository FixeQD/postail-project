import { memo } from 'react'
import { useTranslation } from 'react-i18next'
import { Trash2, FileText } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useDraftStore } from '@/stores/draftStore'
import EditorToolbar from './Editor/EditorToolbar'
import { SignatureSelector } from './SignatureSelector'
import { TemplateGallery } from './TemplateGallery'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { htmlToLexical } from './Editor/utils/conversion'
import type { ComposeFooterProps } from '@/types/components/compose'
import type { Template } from '@/types/templates'

export const ComposeFooter = memo(({ onSend, onDiscard, isValid }: ComposeFooterProps) => {
	const { t } = useTranslation()
	const [editor] = useLexicalComposerContext()
	const { isSaving, isSending, applyTemplate } = useDraftStore()

	const handleTemplateSelect = (template: Template) => {
		applyTemplate(template)
		// Re-hydrate editor after a short delay to ensure store update is processed
		setTimeout(() => {
			htmlToLexical(editor, template.htmlBody)
		}, 50)
	}

	const handleManageTemplates = () => {
		// This should ideally trigger a navigation to settings
		// For now, we'll assume there's a way to open settings templates
		window.dispatchEvent(new CustomEvent('app:open-settings', { detail: { section: 'templates' } }))
	}

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
					<Popover>
						<PopoverTrigger asChild>
							<Button
								variant='ghost'
								size='icon'
								className='h-9 w-9 text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'
								title={t('settings:templates.gallery.title')}>
								<FileText className='h-4 w-4' />
							</Button>
						</PopoverTrigger>
						<PopoverContent align='end' className='w-auto p-0' sideOffset={8}>
							<TemplateGallery
								onSelect={handleTemplateSelect}
								onManage={handleManageTemplates}
							/>
						</PopoverContent>
					</Popover>
					<SignatureSelector />
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
