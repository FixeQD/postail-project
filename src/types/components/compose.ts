import type { LexicalEditor, EditorState } from 'lexical'
import type { EmailAttachment } from '../compose'

export interface ComposeScreenProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	accountId?: string
}

export interface EditorContentProps {
	editorRef: React.RefObject<HTMLDivElement | null>
	htmlRef: React.MutableRefObject<string>
	isHydratingRef: React.MutableRefObject<boolean>
	handleEditorChange: (editorState: EditorState, editor: LexicalEditor) => void
	attachments: EmailAttachment[]
	onRemoveAttachment: (id: string) => void
	onSourceChange?: () => void
	autoFixKey?: number
	isFixing?: boolean
	onEditorMount?: () => void
}

export interface EditorToolbarProps {
	editor: LexicalEditor
	onAttach?: () => void
}

export interface LinkPopoverProps {
	editor: LexicalEditor
	formats: {
		bold: boolean
		italic: boolean
		underline: boolean
		strikethrough: boolean
		ordered: boolean
		unordered: boolean
		link: boolean
	}
	linkData: { url: string; text: string }
}

export interface RichTextEditorProps {
	contentEditable: React.ReactElement
	placeholder: React.ReactElement
	errorBoundary: React.ComponentType<any>
	handleEditorChange: (editorState: EditorState, editor: LexicalEditor) => void
}

export interface MonacoEnvironment {
	getWorkerUrl: (moduleId: string, label: string) => string
}

export interface SourceEditorProps {
	htmlRef: React.MutableRefObject<string>
	onChange?: (value: string | undefined) => void
	isFixing?: boolean
	onMount?: () => void
}

export interface AttachmentListProps {
	attachments: EmailAttachment[]
	onRemove: (id: string) => void
}

export interface CompatibilityButtonProps {
	isOpen: boolean
	onClick: () => void
	issues: import('../compose').SanitizeIssue[]
	isLoading?: boolean
}

export interface CompatibilityPanelProps {
	isOpen: boolean
	onClose: () => void
	width: number
	onWidthChange: (width: number) => void
	issues: import('../compose').SanitizeIssue[]
	isLoading: boolean
	onCheckAgain: () => void
}

export interface ComposeInputsProps {
	to: import('../compose').EmailAddress[]
	cc: import('../compose').EmailAddress[]
	bcc: import('../compose').EmailAddress[]
	subject: string
	showCc: boolean
	showBcc: boolean
	setShowCc: (show: boolean) => void
	setShowBcc: (show: boolean) => void
	onUpdate: (updates: Partial<import('../compose').ComposeDraft>) => void
	onAddRecipient: (
		type: 'to' | 'cc' | 'bcc',
		recipient: import('../compose').EmailAddress
	) => void
	onRemoveRecipient: (type: 'to' | 'cc' | 'bcc', email: string) => void
}

export interface ComposeHeaderProps {
	isDragging: boolean
	onMouseDown: (e: React.MouseEvent) => void
	onClose: () => void
}

export interface ComposeFooterProps {
	onSend: () => void
	onDiscard: () => void
	isValid: boolean
}

export interface SubjectInputProps {
	value: string
	onChange: (value: string) => void
	placeholder?: string
	className?: string
	autoFocus?: boolean
}

export interface Contact {
	id: number
	email: string
	name: string | null
}

export interface AddressInputProps {
	label: string
	recipients: import('../compose').EmailAddress[]
	onAdd: (recipient: import('../compose').EmailAddress) => void
	onRemove: (email: string) => void
	placeholder?: string
	className?: string
	rightElement?: React.ReactNode
}
