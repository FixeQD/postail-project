// @ts-ignore - internal monaco paths are valid but untyped in some configs
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api'
// @ts-ignore
import 'monaco-editor/esm/vs/language/html/monaco.contribution'
// @ts-ignore
import 'monaco-editor/esm/vs/language/css/monaco.contribution'
// @ts-ignore
import 'monaco-editor/esm/vs/basic-languages/html/html.contribution'
import { loader, Editor } from '@monaco-editor/react'
import { useDraftStore } from '@/stores/draftStore'
import { memo, useCallback, useEffect } from 'react'
import { html_beautify } from 'js-beautify'
import { motion } from 'framer-motion'

// @ts-ignore
import htmlWorkerUrl from 'monaco-editor/esm/vs/language/html/html.worker?url'
// @ts-ignore
import cssWorkerUrl from 'monaco-editor/esm/vs/language/css/css.worker?url'
// @ts-ignore
import editorWorkerUrl from 'monaco-editor/esm/vs/editor/editor.worker?url'

// mass magic - manual minimal import (only HTML & CSS) to keep the bundle tiny
if (typeof window !== 'undefined') {
	;(window as any).MonacoEnvironment = {
		getWorkerUrl: (_moduleId: string, label: string) => {
			switch (label) {
				case 'html':
					return htmlWorkerUrl
				case 'css':
					return cssWorkerUrl
				default:
					return editorWorkerUrl
			}
		},
	}
	loader.config({ monaco })
}

interface SourceEditorProps {
	htmlRef: React.MutableRefObject<string>
	onChange?: (value: string | undefined) => void
	isFixing?: boolean
	onMount?: () => void
}

const formatOptions: import('js-beautify').HTMLBeautifyOptions = {
	indent_size: 1,
	indent_char: '\t',
	max_preserve_newlines: 1,
	preserve_newlines: false,
	wrap_line_length: 0,
	wrap_attributes: 'auto',
	wrap_attributes_indent_size: 1,
	end_with_newline: false,
	indent_inner_html: true,
	extra_liners: [],
}

export const SourceEditor = memo(({ htmlRef, onChange, isFixing, onMount }: SourceEditorProps) => {
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

	useEffect(() => {
		const currentHtml = htmlRef.current
		if (!currentHtml || !onChange) return

		const needsFormatting = !currentHtml.includes('\n') || currentHtml.length > 200
		if (!needsFormatting) return

		const formatted = html_beautify(currentHtml, formatOptions)

		htmlRef.current = formatted
		onChange(formatted)
		markDirty()
	}, [htmlRef, onChange, markDirty])

	return (
		<motion.div
			className='relative flex min-h-0 flex-1 flex-col overflow-hidden bg-[#1e1e1e]'
			initial={{
				filter: isFixing ? 'blur(6px) brightness(0.6)' : 'blur(0px) brightness(1)',
			}}
			animate={{
				filter: isFixing ? 'blur(6px) brightness(0.6)' : 'blur(0px) brightness(1)',
			}}
			transition={{
				duration: 0.3,
				ease: 'easeInOut',
			}}>
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
				onMount={onMount}
			/>
		</motion.div>
	)
})

export default SourceEditor
