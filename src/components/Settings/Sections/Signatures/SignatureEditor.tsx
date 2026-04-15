import React, {
	useCallback,
	useEffect,
	useRef,
	memo,
	useMemo,
	useState,
	type ReactElement,
} from 'react'
import { LexicalComposer } from '@lexical/react/LexicalComposer'
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin'
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin'
import { ListPlugin } from '@lexical/react/LexicalListPlugin'
import { OnChangePlugin } from '@lexical/react/LexicalOnChangePlugin'
import { ContentEditable } from '@lexical/react/LexicalContentEditable'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { HeadingNode, QuoteNode } from '@lexical/rich-text'
import { ListNode, ListItemNode } from '@lexical/list'
import { LinkNode } from '@lexical/link'
import { FORMAT_TEXT_COMMAND, EditorState, LexicalEditor, type LexicalCommand } from 'lexical'
import { Bold, Italic, Underline } from 'lucide-react'
import { lexicalToHtml, htmlToLexical } from '@/components/Compose/Editor/utils/conversion'

interface SignatureEditorProps {
	initialHtml: string
	onChange: (html: string) => void
}

const theme = {
	text: {
		bold: 'font-bold',
		italic: 'italic',
		underline: 'underline',
		strikethrough: 'line-through',
	},
	list: {
		listitem: '!ml-4',
		nested: { listitem: '!ml-8' },
		ol: '!list-decimal !ml-4',
		ul: '!list-disc !ml-4',
	},
	link: 'underline text-cyan-400',
}

const nodes = [HeadingNode, QuoteNode, ListNode, ListItemNode, LinkNode]

function useErrorBoundary() {
	return useMemo(() => {
		class ErrorBoundary extends React.Component<
			{ children: ReactElement; onError: (error: Error) => void },
			{ hasError: boolean }
		> {
			constructor(props: { children: ReactElement; onError: (error: Error) => void }) {
				super(props)
				this.state = { hasError: false }
			}

			static getDerivedStateFromError() {
				return { hasError: true }
			}

			componentDidCatch(error: Error) {
				this.props.onError(error)
			}

			render() {
				if (this.state.hasError) {
					return <div className='p-4 text-red-500'>Editor crashed.</div>
				}
				return this.props.children
			}
		}
		return ErrorBoundary
	}, [])
}

function HydrationPlugin({ initialHtml }: { initialHtml: string }) {
	const [editor] = useLexicalComposerContext()
	const lastHtmlRef = useRef(initialHtml)
	const isInitializedRef = useRef(false)

	useEffect(() => {
		if (!isInitializedRef.current && initialHtml) {
			isInitializedRef.current = true
			lastHtmlRef.current = initialHtml
			htmlToLexical(editor, initialHtml)
		}
	}, [initialHtml, editor])

	return null
}

function SignatureToolbar() {
	const [editor] = useLexicalComposerContext()
	const [formats, setFormats] = useState({
		bold: false,
		italic: false,
		underline: false,
	})

	useEffect(() => {
		return editor.registerUpdateListener(({ editorState }) => {
			editorState.read(() => {
				const selection = editorState._selection
				if (selection && selection.hasFormat) {
					setFormats({
						bold: selection.hasFormat('bold'),
						italic: selection.hasFormat('italic'),
						underline: selection.hasFormat('underline'),
					})
				} else {
					setFormats({ bold: false, italic: false, underline: false })
				}
			})
		})
	}, [editor])

	const exec = (cmd: string) => {
		editor.dispatchCommand(FORMAT_TEXT_COMMAND as LexicalCommand<string>, cmd)
		editor.focus()
	}

	return (
		<div className='flex items-center gap-1 border-b border-[var(--border-faint)] px-3 py-1.5'>
			<button
				type='button'
				onMouseDown={(e) => e.preventDefault()}
				onClick={() => exec('bold')}
				className={`flex h-7 w-7 items-center justify-center rounded-md transition-colors ${
					formats.bold
						? 'bg-[var(--accent-primary)] text-white'
						: 'text-[var(--text-secondary)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
				}`}>
				<Bold className='h-3.5 w-3.5' />
			</button>
			<button
				type='button'
				onMouseDown={(e) => e.preventDefault()}
				onClick={() => exec('italic')}
				className={`flex h-7 w-7 items-center justify-center rounded-md transition-colors ${
					formats.italic
						? 'bg-[var(--accent-primary)] text-white'
						: 'text-[var(--text-secondary)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
				}`}>
				<Italic className='h-3.5 w-3.5' />
			</button>
			<button
				type='button'
				onMouseDown={(e) => e.preventDefault()}
				onClick={() => exec('underline')}
				className={`flex h-7 w-7 items-center justify-center rounded-md transition-colors ${
					formats.underline
						? 'bg-[var(--accent-primary)] text-white'
						: 'text-[var(--text-secondary)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'
				}`}>
				<Underline className='h-3.5 w-3.5' />
			</button>
		</div>
	)
}

export const SignatureEditor = memo(function SignatureEditor({
	initialHtml,
	onChange,
}: SignatureEditorProps) {
	const handleChange = useCallback(
		(editorState: EditorState, editor: LexicalEditor) => {
			editorState.read(() => {
				const html = lexicalToHtml(editor)
				onChange(html)
			})
		},
		[onChange]
	)

	const ErrorBoundary = useErrorBoundary()

	const initialConfig = {
		namespace: 'SignatureEditor',
		theme,
		nodes,
		onError: (err: Error) => console.error(err),
	}

	return (
		<div className='overflow-hidden rounded-lg border border-[var(--border-faint)] bg-[var(--surface-panel)]'>
			<LexicalComposer initialConfig={initialConfig}>
				<SignatureToolbar />
				<div className='relative min-h-[120px] p-3'>
					<RichTextPlugin
						contentEditable={
							<ContentEditable className='min-h-[100px] w-full text-sm text-[var(--text-primary)] outline-none' />
						}
						placeholder={
							<div className='pointer-events-none absolute top-3 left-3 text-sm text-[var(--text-tertiary)]'>
								Type your signature...
							</div>
						}
						ErrorBoundary={ErrorBoundary}
					/>
				</div>
				<HistoryPlugin />
				<ListPlugin />
				<OnChangePlugin onChange={handleChange} />
				<HydrationPlugin initialHtml={initialHtml} />
			</LexicalComposer>
		</div>
	)
})
