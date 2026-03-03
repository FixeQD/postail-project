import { useRef, useCallback } from 'react'
import { useAnimate } from 'framer-motion'

export function useShellTransition() {
	const transitioning = useRef(false)
	const [shellScope, animateShell] = useAnimate()
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

			await animateContent(content, { opacity: 0 }, { duration: 0.15, ease: 'easeInOut' })

			shell.style.height = shell.offsetHeight + 'px'

			await swap()

			await new Promise<void>((r) =>
				requestAnimationFrame(() => requestAnimationFrame(() => r()))
			)

			const newH = content.scrollHeight
			await animateShell(shell, { height: newH }, { duration: 0.36, ease: [0.16, 1, 0.3, 1] })

			shell.style.height = 'auto'

			await animateContent(content, { opacity: 1 }, { duration: 0.15, ease: 'easeInOut' })

			transitioning.current = false
		},
		[animateContent, animateShell, contentScope, shellScope]
	)

	const reset = useCallback(() => {
		transitioning.current = false
		const shell = shellScope.current as HTMLDivElement | null
		if (shell) {
			shell.style.height = 'auto'
		}
	}, [shellScope])

	return { shellScope, contentScope, transition, reset }
}
