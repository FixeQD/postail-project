import { useRef, useCallback } from 'react'
import { useAnimate } from 'framer-motion'

const DURATION = 360

// Cubic bezier approximation of (0.16, 1, 0.3, 1) — spring-like ease out
function easeOutSpring(t: number): number {
	return 1 - Math.pow(1 - t, 3) * Math.cos(t * Math.PI * 2.5)
}

function animateHeightRaf(
	shell: HTMLDivElement,
	fromH: number,
	toH: number,
	signal: AbortSignal
): Promise<void> {
	if (fromH === toH) return Promise.resolve()

	return new Promise<void>((resolve) => {
		let start: number | null = null
		let rafId: number

		const tick = (now: number) => {
			if (signal.aborted) {
				shell.style.height = 'auto'
				resolve()
				return
			}

			if (start === null) start = now
			const elapsed = now - start
			const t = Math.min(elapsed / DURATION, 1)
			const eased = easeOutSpring(t)
			shell.style.height = fromH + (toH - fromH) * eased + 'px'

			if (t < 1) {
				rafId = requestAnimationFrame(tick)
			} else {
				shell.style.height = 'auto'
				resolve()
			}
		}

		rafId = requestAnimationFrame(tick)

		signal.addEventListener(
			'abort',
			() => {
				cancelAnimationFrame(rafId)
				shell.style.height = 'auto'
				resolve()
			},
			{ once: true }
		)
	})
}

export function useShellTransition() {
	const transitioning = useRef(false)
	const abortRef = useRef<AbortController | null>(null)
	const [shellScope] = useAnimate()
	const [contentScope, animateContent] = useAnimate()

	const reset = useCallback(() => {
		abortRef.current?.abort()
		abortRef.current = null
		transitioning.current = false

		const shell = shellScope.current as HTMLDivElement | null
		const content = contentScope.current as HTMLDivElement | null
		if (shell) {
			shell.style.height = 'auto'
		}
		if (content) {
			content.style.opacity = '0'
		}
	}, [shellScope, contentScope])

	const transition = useCallback(
		async (swap: () => void | Promise<void>) => {
			if (transitioning.current) return
			transitioning.current = true

			const ac = new AbortController()
			abortRef.current = ac

			const shell = shellScope.current as HTMLDivElement | null
			const content = contentScope.current as HTMLDivElement | null
			if (!shell || !content) {
				transitioning.current = false
				return
			}

			try {
				await animateContent(content, { opacity: 0 }, { duration: 0.15, ease: 'easeInOut' })
				if (ac.signal.aborted) return

				const fromH = shell.offsetHeight
				shell.style.height = fromH + 'px'

				await swap()
				if (ac.signal.aborted) return

				await new Promise<void>((r) =>
					requestAnimationFrame(() => requestAnimationFrame(() => r()))
				)
				if (ac.signal.aborted) return

				shell.style.height = 'auto'
				const toH = content.scrollHeight
				shell.style.height = fromH + 'px'

				await animateHeightRaf(shell, fromH, toH, ac.signal)
				if (ac.signal.aborted) return

				await animateContent(content, { opacity: 1 }, { duration: 0.15, ease: 'easeInOut' })
			} finally {
				if (!ac.signal.aborted) {
					transitioning.current = false
					abortRef.current = null
				}
			}
		},
		[animateContent, contentScope, shellScope]
	)

	return { shellScope, contentScope, transition, reset }
}
