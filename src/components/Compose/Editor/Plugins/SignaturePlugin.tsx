import { useEffect } from 'react'

/**
 * Plugin that handles signature injection and updates via direct DOM manipulation
 */
export default function SignaturePlugin() {
	useEffect(() => {
		const handler = (e: Event) => {
			const html = (e as CustomEvent).detail as string | null
			const el = document.querySelector('.wysiwyg-editable') as HTMLDivElement | null
			if (!el) return

			const sigBlockRegex = /<!-- SIGNATURE_START -->[\s\S]*?<!-- SIGNATURE_END -->/i

			if (html === null) {
				el.innerHTML = el.innerHTML.replace(sigBlockRegex, '')
			} else {
				const sigHtml = `<!-- SIGNATURE_START --><br><br><div class="signature-wrapper"><div class="signature">${html}</div></div><!-- SIGNATURE_END -->`
				
				if (sigBlockRegex.test(el.innerHTML)) {
					el.innerHTML = el.innerHTML.replace(sigBlockRegex, sigHtml)
				} else {
					el.insertAdjacentHTML('beforeend', sigHtml)
				}
			}
		}

		window.addEventListener('compose:update-signature', handler)
		return () => window.removeEventListener('compose:update-signature', handler)
	}, [])

	return null
}
