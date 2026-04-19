import { useState, useEffect, useCallback, useRef, memo } from 'react'
import {
	Bold,
	Italic,
	Underline,
	Strikethrough,
	List as ListIcon,
	ListOrdered,
} from 'lucide-react'
import { WysiwygEditor } from '@/components/Compose/Editor/Modes/WysiwygEditor'
import { LinkInsertPopover } from '@/components/Compose/Editor/LinkPopover'

interface SignatureEditorProps {
	initialHtml: string
	placeholder?: string
	onChange: (html: string) => void
}

const useEditorFormats = () => {
	const [formats, setFormats] = useState({
		bold: false,
		italic: false,
		underline: false,
		strikethrough: false,
		ordered: false,
		unordered: false,
		link: false,
	})

	const updateFormats = useCallback(() => {
		// Only update if we are in a contenteditable context
		const activeEl = document.activeElement
		const isEditable = activeEl?.getAttribute('contenteditable') === 'true' || 
						  activeEl?.closest('[contenteditable="true"]')
		
		if (!isEditable) return

		const isFormatActive = (command: string) => {
			try {
				return document.queryCommandState(command)
			} catch {
				return false
			}
		}

		const isLinkActive = () => {
			const selection = window.getSelection()
			if (!selection || !selection.anchorNode) return false
			let node: Node | null = selection.anchorNode
			if (node.nodeType === Node.TEXT_NODE) node = node.parentNode
			while (node && node instanceof Element && node.getAttribute('contenteditable') !== 'true') {
				if (node.nodeName === 'A') return true
				node = node.parentNode
			}
			return false
		}

		setFormats({
			bold: isFormatActive('bold'),
			italic: isFormatActive('italic'),
			underline: isFormatActive('underline'),
			strikethrough: isFormatActive('strikeThrough'),
			ordered: isFormatActive('insertOrderedList'),
			unordered: isFormatActive('insertUnorderedList'),
			link: isLinkActive(),
		})
	}, [])

	useEffect(() => {
		// Initial check
		updateFormats()

		document.addEventListener('selectionchange', updateFormats)
		document.addEventListener('focusin', updateFormats)
		return () => {
			document.removeEventListener('selectionchange', updateFormats)
			document.removeEventListener('focusin', updateFormats)
		}
	}, [updateFormats])

	return { formats, updateFormats }
}

function SignatureToolbar({ editorRef }: { editorRef: React.RefObject<HTMLDivElement | null> }) {
	const { formats, updateFormats } = useEditorFormats()

	const exec = (cmd: string, val: string | undefined = undefined) => {
		const el = editorRef.current
		if (el && document.activeElement !== el) {
			el.focus()
		}
		document.execCommand(cmd, false, val)
		updateFormats()
	}

	const ToolbarButton = ({
		active,
		onClick,
		children,
		title,
	}: {
		active: boolean
		onClick: () => void
		children: React.ReactNode
		title?: string
	}) => (
		<button
			type='button'
			onMouseDown={(e) => e.preventDefault()}
			onClick={onClick}
			title={title}
			className={`flex h-9 w-9 items-center justify-center rounded-md transition-colors ${
				active
					? 'bg-[var(--compose-active)] text-[var(--compose-text)] shadow-sm'
					: 'text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)]'
			}`}>
			{children}
		</button>
	)

	return (
		<div className='flex items-center gap-1 border-b border-[var(--border-faint)] px-3 py-1.5'>
			<ToolbarButton active={formats.bold} onClick={() => exec('bold')} title='Bold'>
				<Bold className='h-4 w-4' />
			</ToolbarButton>
			<ToolbarButton active={formats.italic} onClick={() => exec('italic')} title='Italic'>
				<Italic className='h-4 w-4' />
			</ToolbarButton>
			<ToolbarButton
				active={formats.underline}
				onClick={() => exec('underline')}
				title='Underline'>
				<Underline className='h-4 w-4' />
			</ToolbarButton>
			<ToolbarButton
				active={formats.strikethrough}
				onClick={() => exec('strikeThrough')}
				title='Strikethrough'>
				<Strikethrough className='h-4 w-4' />
			</ToolbarButton>

			<div className='mx-1 h-4 w-px bg-[var(--border-faint)]' />

			<ToolbarButton
				active={formats.unordered}
				onClick={() => exec('insertUnorderedList')}
				title='Bullet List'>
				<ListIcon className='h-4 w-4' />
			</ToolbarButton>
			<ToolbarButton
				active={formats.ordered}
				onClick={() => exec('insertOrderedList')}
				title='Numbered List'>
				<ListOrdered className='h-4 w-4' />
			</ToolbarButton>

			<div className='mx-1 h-4 w-px bg-[var(--border-faint)]' />

			<LinkInsertPopover onInsertLink={(url) => exec('createLink', url)} />
		</div>
	)
}

export const SignatureEditor = memo(function SignatureEditor({
	initialHtml,
	placeholder = 'Type your signature...',
	onChange,
}: SignatureEditorProps) {
	const editorRef = useRef<HTMLDivElement>(null)

	return (
		<div className='overflow-hidden rounded-lg border border-[var(--border-faint)] bg-[var(--surface-panel)]'>
			<SignatureToolbar editorRef={editorRef} />
			<div className='relative min-h-[120px]'>
				<WysiwygEditor
					editorRef={editorRef}
					value={initialHtml}
					onChange={onChange}
					placeholder={placeholder}
					className='!h-auto !min-h-[120px]'
				/>
			</div>
		</div>
	)
})
