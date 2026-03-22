import { useRef, useCallback } from 'react'
import { useAnimate } from 'framer-motion'

const DURATION = 360
const EASING = 'cubic-bezier(0.16, 1, 0.3, 1)'

async function animateHeight(shell: HTMLDivElement, fromH: number, toH: number): Promise<void> {
	if (fromH === toH) return

	await new Promise<void>((resolve) => {
		// Hint compositor to promote the layer before animating, prevents layout-driven lag on WebKit2GTK and snap on WebView2.
		shell.style.willChange = 'height'

		// Force a sync layout read so the browser commits fromH as the starting point before the transition kicks in.
		void shell.offsetHeight

		shell.style.transition = `height ${DURATION}ms ${EASING}`
		shell.style.height = toH + 'px'

		const onEnd = (e: TransitionEvent) => {
			if (e.propertyName !== 'height') return
			shell.removeEventListener('transitionend', onEnd)
			shell.style.willChange = ''
			shell.style.transition = ''
			resolve()
		}
		shell.addEventListener('transitionend', onEnd)

		// Fallback if transitionend doesn't fire
		setTimeout(() => {
			shell.removeEventListener('transitionend', onEnd)
			shell.style.willChange = ''
			shell.style.transition = ''
			resolve()
		}, DURATION + 50)
	})
}

export function useShellTransition() {
	const transitioning = useRef(false)
	const [shellScope] = useAnimate()
	const [contentScope, animateContent] = useAnimate()

	const transition = useCallback(
		async (swap: () => void | Promise<void>) => {
			if (transitioning.current) return
			transitioning.current = true

			const shell = shellScope.current as HTMLDivElement | null
			const content = contentScope.current as HTMLDivElement | null
			if (!shell || !content) {
				transitioning.current = false
				return
			}

			try {
				await animateContent(content, { opacity: 0 }, { duration: 0.15, ease: 'easeInOut' })

				const fromH = shell.offsetHeight
				shell.style.height = fromH + 'px'

				await swap()

				await new Promise<void>((r) =>
					requestAnimationFrame(() => requestAnimationFrame(() => r()))
				)

				// Measure unconstrained so scrollHeight isn't clipped
				shell.style.height = 'auto'
				const toH = content.scrollHeight
				shell.style.height = fromH + 'px'

				await animateHeight(shell, fromH, toH)

				shell.style.height = 'auto'

				await animateContent(content, { opacity: 1 }, { duration: 0.15, ease: 'easeInOut' })
			} finally {
				transitioning.current = false
			}
		},
		[animateContent, contentScope, shellScope]
	)

	const reset = useCallback(() => {
		transitioning.current = false
		const shell = shellScope.current as HTMLDivElement | null
		if (shell) {
			shell.style.willChange = ''
			shell.style.transition = ''
			shell.style.height = 'auto'
		}
	}, [shellScope])

	return { shellScope, contentScope, transition, reset }
}
