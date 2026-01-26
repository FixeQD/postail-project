import { $generateHtmlFromNodes, $generateNodesFromDOM } from '@lexical/html'
import { LexicalEditor, $getRoot } from 'lexical'

/**
 * Converts Lexical editor state to an HTML string.
 */
export const lexicalToHtml = (editor: LexicalEditor): string => {
	return $generateHtmlFromNodes(editor)
}

/**
 * Parses an HTML string and replaces the current Lexical editor content.
 */
export const htmlToLexical = (editor: LexicalEditor, html: string) => {
	if (!html) return

	editor.update(() => {
		const root = $getRoot()
		const parser = new DOMParser()
		const dom = parser.parseFromString(html, 'text/html')
		const nodes = $generateNodesFromDOM(editor, dom.body)

		root.clear()
		root.append(...nodes)
	})
}
