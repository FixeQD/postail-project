import { useRef, useEffect, memo, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { LexicalEditor, EditorState, $getRoot } from 'lexical'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin'
import { ContentEditable } from '@lexical/react/LexicalContentEditable'
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin'
import { ListPlugin } from '@lexical/react/LexicalListPlugin'
import { OnChangePlugin } from '@lexical/react/LexicalOnChangePlugin'
import { $generateNodesFromDOM } from '@lexical/html'
import { Trash2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { useDraftStore } from '@/stores/draftStore'
import EditorToolbar from './EditorToolbar'

interface EditorContentProps {
	onOpenChange: (open: boolean) => void
	editorRef: React.RefObject<HTMLDivElement | null>
	htmlRef: React.MutableRefObject<string>
	isHydratingRef: React.MutableRefObject<boolean>
	handleEditorChange: (editorState: EditorState, editor: LexicalEditor) => void
}

const MemoRichTextPlugin = memo(RichTextPlugin)
const MemoHistoryPlugin = memo(HistoryPlugin)
const MemoListPlugin = memo(ListPlugin)
const MemoOnChangePlugin = memo(OnChangePlugin)

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
		} = useDraftStore()

		const [editor] = useLexicalComposerContext()
		const lastHydratedIdRef = useRef<string | null>(null)

		useEffect(() => {
			if (
				!currentDraft?.id ||
				!currentDraft.body ||
				currentDraft.body.trim() === '' ||
				!editor ||
				currentDraft.id === lastHydratedIdRef.current
			)
				return

			const timeoutId = setTimeout(() => {
				try {
					isHydratingRef.current = true
					editor.update(() => {
						const root = $getRoot()
						const parser = new DOMParser()
						const dom = parser.parseFromString(currentDraft.body, 'text/html')
						const nodes = $generateNodesFromDOM(editor, dom.body)
						root.clear()
						root.append(...nodes)
						lastHydratedIdRef.current = currentDraft!.id || null
					})
					htmlRef.current = currentDraft!.body
					markClean()
					isHydratingRef.current = false
				} catch (error) {
					console.error('Draft hydration error:', error)
					isHydratingRef.current = false
				}
			}, 100)
			return () => clearTimeout(timeoutId)
		}, [currentDraft?.id, currentDraft?.body, editor, isHydratingRef, htmlRef, markClean])

		return (
			<>
				<div
					ref={editorRef}
					className='editor-content custom-scrollbar relative flex-1 overflow-y-auto p-4'>
					<MemoRichTextPlugin
						contentEditable={useMemo(
							() => (
								<ContentEditable className='h-full min-h-50 w-full text-sm text-zinc-200 outline-none focus:outline-none' />
							),
							[]
						)}
						placeholder={useMemo(
							() => (
								<div className='pointer-events-none absolute top-4 left-4 text-sm text-zinc-600'>
									{t('compose.writeSomething')}
								</div>
							),
							[t]
						)}
						ErrorBoundary={useMemo(() => () => <div>Error loading editor</div>, [])}
					/>
					<style>{`.editor-content ul ::marker, .editor-content ol ::marker { color: #e4e4e7; }`}</style>
					<MemoHistoryPlugin />
					<MemoListPlugin />
					<MemoOnChangePlugin onChange={handleEditorChange} />
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
