import { memo, useCallback } from 'react'
import WysiwygEditor from './Modes/WysiwygEditor'
import SourceEditor from './Modes/SourceEditor'
import { useDraftStore } from '@/stores/draftStore'
import { AttachmentList } from '../AttachmentList'
import type { EditorContentProps } from '@/types/components/compose'

export const EditorContent = memo(
	({
		editorRef,
		htmlRef,
		attachments,
		onRemoveAttachment,
		onSourceChange,
		autoFixKey,
		isFixing,
		onEditorMount,
	}: EditorContentProps) => {
		const { currentDraft, editorMode, markDirty } = useDraftStore()

		const handleWysiwygChange = useCallback(
			(html: string) => {
				htmlRef.current = html
				markDirty()
			},
			[htmlRef, markDirty]
		)

		return (
			<>
				<div
					ref={editorRef}
					className={`editor-content custom-scrollbar relative flex flex-1 flex-col ${editorMode === 'rich-text' ? 'overflow-y-auto' : 'overflow-hidden'} min-h-0 p-0`}>
					{editorMode === 'rich-text' ? (
						<WysiwygEditor
							value={htmlRef.current}
							onChange={handleWysiwygChange}
							placeholder='Compose message...'
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
