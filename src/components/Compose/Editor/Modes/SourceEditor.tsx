// @ts-ignore - internal monaco paths are valid but untyped in some configs
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api'
// @ts-ignore
import 'monaco-editor/esm/vs/language/html/monaco.contribution'
// @ts-ignore
import 'monaco-editor/esm/vs/language/css/monaco.contribution'
import { loader, Editor } from '@monaco-editor/react'
import { useDraftStore } from '@/stores/draftStore'
import { memo, useCallback } from 'react'

// mass magic - manual minimal import (only HTML & CSS) to keep the bundle tiny
if (typeof window !== 'undefined') {
	loader.config({ monaco })
}

interface SourceEditorProps {
	htmlRef: React.MutableRefObject<string>
	onChange?: (value: string | undefined) => void
}

export const SourceEditor = memo(({ htmlRef, onChange }: SourceEditorProps) => {
	const { markDirty } = useDraftStore()

	const handleEditorChange = useCallback(
		(value: string | undefined) => {
			if (value !== undefined) {
				htmlRef.current = value
				markDirty()
				onChange?.(value)
			}
		},
		[htmlRef, markDirty, onChange]
	)

	return (
		<div className='flex min-h-0 flex-1 flex-col overflow-hidden bg-[#1e1e1e]'>
			<Editor
				height='100%'
				defaultLanguage='html'
				value={htmlRef.current}
				theme='vs-dark'
				options={{
					minimap: { enabled: false },
					fontSize: 14,
					wordWrap: 'on',
					lineNumbers: 'on',
					scrollBeyondLastLine: false,
					automaticLayout: true,
					padding: { top: 12, bottom: 12 },
					tabSize: 4,
					renderLineHighlight: 'all',
					fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
					fixedOverflowWidgets: true,
					roundedSelection: true,
					bracketPairColorization: { enabled: true },
				}}
				onChange={handleEditorChange}
			/>
		</div>
	)
})

export default SourceEditor
