import * as monaco from 'monaco-editor'
import { loader, Editor } from '@monaco-editor/react'
import { useDraftStore } from '@/stores/draftStore'
import { memo, useCallback } from 'react'

import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker'
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker'
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker'
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker'

// mass magic - Vite needs explicit worker factories for Monaco
if (typeof window !== 'undefined') {
	// @ts-ignore - self is window in browser context
	self.MonacoEnvironment = {
		getWorker(_: any, label: string) {
			if (label === 'json') return new jsonWorker()
			if (label === 'css' || label === 'scss' || label === 'less') return new cssWorker()
			if (label === 'html' || label === 'handlebars' || label === 'razor') return new htmlWorker()
			if (label === 'typescript' || label === 'javascript') return new tsWorker()
			return new editorWorker()
		},
	}
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
