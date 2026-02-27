import { memo } from 'react'
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin'
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin'
import { ListPlugin } from '@lexical/react/LexicalListPlugin'
import { OnChangePlugin } from '@lexical/react/LexicalOnChangePlugin'
import PastePlugin from '../Plugins/PastePlugin'
import DragDropPlugin from '../Plugins/DragDropPlugin'
import type { RichTextEditorProps } from '@/types/components/compose'

const MemoRichTextPlugin = memo(RichTextPlugin)
const MemoHistoryPlugin = memo(HistoryPlugin)
const MemoListPlugin = memo(ListPlugin)
const MemoOnChangePlugin = memo(OnChangePlugin)

export const RichTextEditor = memo(
	({ contentEditable, placeholder, errorBoundary, handleEditorChange }: RichTextEditorProps) => {
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
				<DragDropPlugin />
			</div>
		)
	}
)

export default RichTextEditor
