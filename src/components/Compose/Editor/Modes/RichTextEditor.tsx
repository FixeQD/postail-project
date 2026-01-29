import { memo } from 'react'
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin'
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin'
import { ListPlugin } from '@lexical/react/LexicalListPlugin'
import { OnChangePlugin } from '@lexical/react/LexicalOnChangePlugin'
import { LexicalEditor, EditorState } from 'lexical'
import PastePlugin from '../Plugins/PastePlugin'

interface RichTextEditorProps {
	contentEditable: any
	placeholder: any
	errorBoundary: any
	handleEditorChange: (editorState: EditorState, editor: LexicalEditor) => void
}

const MemoRichTextPlugin = memo(RichTextPlugin)
const MemoHistoryPlugin = memo(HistoryPlugin)
const MemoListPlugin = memo(ListPlugin)
const MemoOnChangePlugin = memo(OnChangePlugin)

export const RichTextEditor = memo(
	({
		contentEditable,
		placeholder,
		errorBoundary,
		handleEditorChange,
	}: RichTextEditorProps) => {
		return (
			<div className='flex-1 p-4'>
				<MemoRichTextPlugin
					contentEditable={contentEditable}
					placeholder={placeholder}
					ErrorBoundary={errorBoundary}
				/>
				<style>{`.editor-content ul ::marker, .editor-content ol ::marker { color: #e4e4e7; }`}</style>
				<MemoHistoryPlugin />
				<MemoListPlugin />
				<MemoOnChangePlugin onChange={handleEditorChange} />
				<PastePlugin />
			</div>
		)
	}
)

export default RichTextEditor
