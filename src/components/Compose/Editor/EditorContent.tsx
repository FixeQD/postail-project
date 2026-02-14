import React, { useRef, useEffect, memo, useMemo } from 'react'
import { LexicalEditor, EditorState } from 'lexical'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { ContentEditable } from '@lexical/react/LexicalContentEditable'
import RichTextEditor from './Modes/RichTextEditor'
import SourceEditor from './Modes/SourceEditor'
import { useDraftStore } from '@/stores/draftStore'
import { htmlToLexical } from './utils/conversion'
import { AttachmentList } from '../AttachmentList'

interface EditorContentProps {
	editorRef: React.RefObject<HTMLDivElement | null>
	htmlRef: React.MutableRefObject<string>
	isHydratingRef: React.MutableRefObject<boolean>
	handleEditorChange: (editorState: EditorState, editor: LexicalEditor) => void
	attachments: any[]
	onRemoveAttachment: (id: string) => void
	onSourceChange?: () => void
	autoFixKey?: number
	isFixing?: boolean
	onEditorMount?: () => void
}

export const EditorContent = memo(
	({
		editorRef,
		htmlRef,
		isHydratingRef,
		handleEditorChange,
		attachments,
		onRemoveAttachment,
		onSourceChange,
		autoFixKey,
		isFixing,
		onEditorMount,
	}: EditorContentProps) => {
		const { currentDraft, markClean, editorMode } = useDraftStore()
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
					Compose message...
				</div>
			),
			[]
		)

		const errorBoundary = useMemo(() => {
			return class ErrorBoundary extends React.Component<
				{ children: React.ReactNode },
				{ hasError: boolean }
			> {
				constructor(props: { children: React.ReactNode }) {
					super(props)
					this.state = { hasError: false }
				}

				static getDerivedStateFromError() {
					return { hasError: true }
				}

				render() {
					if (this.state.hasError) {
						return <div className='p-4 text-red-500'>Editor crashed.</div>
					}
					return this.props.children
				}
			}
		}, [])

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
						<SourceEditor
							htmlRef={htmlRef}
							onChange={onSourceChange}
							key={autoFixKey}
							isFixing={isFixing}
							onMount={onEditorMount}
						/>
					)}
				</div>

				<AttachmentList attachments={attachments} onRemove={onRemoveAttachment} />
			</>
		)
	}
)

export default EditorContent
