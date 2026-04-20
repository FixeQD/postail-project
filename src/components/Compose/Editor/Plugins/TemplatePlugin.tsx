import { useEffect } from 'react'

export default function TemplatePlugin() {
	// Handle apply-template custom event
	useEffect(() => {
		const handler = (e: Event) => {
			const body = (e as CustomEvent).detail as string
			const el = document.querySelector('.wysiwyg-editable') as HTMLDivElement | null
			if (!el) return

			const sigBlockRegex = /<!-- SIGNATURE_START -->[\s\S]*?<!-- SIGNATURE_END -->/i
			const sigMatch = el.innerHTML.match(sigBlockRegex)
			const sigBlock = sigMatch ? sigMatch[0] : ''

			el.innerHTML = body + sigBlock

			// Move cursor to start so user lands at the top of the template
			const range = document.createRange()
			range.setStart(el, 0)
			range.collapse(true)
			const sel = window.getSelection()
			sel?.removeAllRanges()
			sel?.addRange(range)

			// Trigger change event
			el.dispatchEvent(new Event('compose:editor-change', { bubbles: true }))
		}
		window.addEventListener('compose:apply-template', handler)
		return () => window.removeEventListener('compose:apply-template', handler)
	}, [])

	return null
}
