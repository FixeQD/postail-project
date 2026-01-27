import { $generateHtmlFromNodes, $generateNodesFromDOM } from '@lexical/html'
import { LexicalEditor, $getRoot, RangeSelection, NodeSelection } from 'lexical'

/**
 * Converts Lexical editor state to an HTML string.
 */
export const lexicalToHtml = (
	editor: LexicalEditor,
	selection: RangeSelection | NodeSelection | null = null
): string => {
	return $generateHtmlFromNodes(editor, selection)
}

/**
 * Parses an HTML string and replaces the current Lexical editor content.
 */
export const htmlToLexical = (editor: LexicalEditor, html: string) => {
	editor.update(() => {
		const root = $getRoot()

		if (!html || html.trim() === '') {
			root.clear()
			return
		}

		try {
			const parser = new DOMParser()
			const dom = parser.parseFromString(html, 'text/html')

			if (dom.querySelector('parsererror')) {
				console.error('Lexical: HTML translation failed - parser error')
				return
			}

			const nodes = $generateNodesFromDOM(editor, dom)

			root.clear()
			root.append(...nodes)
		} catch (e) {
			console.error('Lexical: Critical error during HTML -> Lexical translation:', e)
		}
	})
}
