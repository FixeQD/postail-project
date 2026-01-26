import { useRef, useEffect, memo, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { LexicalEditor, EditorState } from 'lexical'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { ContentEditable } from '@lexical/react/LexicalContentEditable'
import { Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useDraftStore } from '@/stores/draftStore'
import EditorToolbar from './EditorToolbar'
import RichTextEditor from './Modes/RichTextEditor'
import SourceEditor from './Modes/SourceEditor'
import { htmlToLexical } from './utils/conversion'

interface EditorContentProps {
	onOpenChange: (open: boolean) => void
	editorRef: React.RefObject<HTMLDivElement | null>
	htmlRef: React.MutableRefObject<string>
	isHydratingRef: React.MutableRefObject<boolean>
	handleEditorChange: (editorState: EditorState, editor: LexicalEditor) => void
}

export const EditorContent = memo(
	({
		onOpenChange,
		editorRef,
		htmlRef,
		isHydratingRef,
		handleEditorChange,
	}: EditorContentProps) => {
		const { t } = useTranslation()
		const {
			currentDraft,
			isSaving,
			stopComposing,
			deleteDraft,
			markClean,
			editorMode,
		} = useDraftStore()

		const [editor] = useLexicalComposerContext()
		const lastHydratedIdRef = useRef<string | null>(null)

		const contentEditable = useMemo(
			() => (
				<ContentEditable className='h-full min-h-50 w-full text-sm text-zinc-200 outline-none focus:outline-none' />
			),
			[]
		)

		const placeholder = useMemo(
			() => (
				<div className='pointer-events-none absolute top-4 left-4 text-sm text-zinc-600'>
					{t('compose.writeSomething')}
				</div>
			),
			[t]
		)

		const errorBoundary = useMemo(() => () => <div>Error loading editor</div>, [])

		useEffect(() => {
			if (
				!currentDraft?.id ||
				!currentDraft.body ||
				currentDraft.body.trim() === '' ||
				!editor ||
				currentDraft.id === lastHydratedIdRef.current ||
				editorMode !== 'rich-text'
			)
				return

			const timeoutId = setTimeout(() => {
				try {
					isHydratingRef.current = true
					htmlToLexical(editor, currentDraft.body)
					lastHydratedIdRef.current = currentDraft!.id || null
					htmlRef.current = currentDraft!.body
					markClean()
					isHydratingRef.current = false
				} catch (error) {
					console.error('Draft hydration error:', error)
					isHydratingRef.current = false
				}
			}, 100)
			return () => clearTimeout(timeoutId)
		}, [
			currentDraft?.id,
			currentDraft?.body,
			editor,
			isHydratingRef,
			htmlRef,
			markClean,
			editorMode,
		])

		// Synchronize Source -> Rich Text when switching modes
		useEffect(() => {
			if (editorMode !== 'rich-text' || !editor) return

			const currentMonacoHtml = htmlRef.current
			if (!currentMonacoHtml) return

			htmlToLexical(editor, currentMonacoHtml)
		}, [editorMode, editor, htmlRef])

		return (
			<>
				<div
					ref={editorRef}
					className={`editor-content custom-scrollbar relative flex flex-1 flex-col ${editorMode === 'rich-text' ? 'overflow-y-auto' : 'overflow-hidden'} min-h-0 p-0`}>
					{editorMode === 'rich-text' ? (
						<RichTextEditor
							contentEditable={contentEditable}
							placeholder={placeholder}
							errorBoundary={errorBoundary}
							handleEditorChange={handleEditorChange}
						/>
					) : (
						<SourceEditor htmlRef={htmlRef} />
					)}
				</div>

				<div className='mt-auto border-t border-zinc-900 bg-zinc-950/50 p-3'>
					<div className='flex items-center justify-between'>
						<div className='flex items-center gap-1'>
							<Button
								onClick={async () => {
									if (currentDraft?.id) {
										await deleteDraft(currentDraft.id)
									}
									stopComposing()
									onOpenChange(false)
								}}
								className='h-9 rounded-full bg-blue-600 px-6 font-semibold text-white hover:bg-blue-500'
								disabled={isSaving}>
								{isSaving ? '...' : t('actions.send')}
							</Button>
							<EditorToolbar />
						</div>

						<div className='flex items-center gap-1'>
							<Button
								variant='ghost'
								size='icon'
								className='h-9 w-9 text-zinc-400 hover:bg-zinc-900 hover:text-red-400'>
								<Trash2 className='h-4 w-4' />
							</Button>
						</div>
					</div>
				</div>
			</>
		)
	}
)

export default EditorContent
