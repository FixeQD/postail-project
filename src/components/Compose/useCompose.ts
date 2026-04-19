import { useState, useEffect, useRef, useCallback } from 'react'

export interface DraggingState {
	x: number
	y: number
	width: number
	height: number
	isDragging: boolean
	isResizing: boolean
}

// Keep in sync with ComposeHeader
const COMPOSE_HEADER_H = 44

export const useDragging = () => {
	const [state, setState] = useState<DraggingState>({
		x: 100,
		y: 100,
		width: 672,
		height: 600,
		isDragging: false,
		isResizing: false,
	})

	// All mutable state in refs
	const posRef = useRef({ x: state.x, y: state.y })
	const sizeRef = useRef({ width: state.width, height: state.height })
	const isDraggingRef = useRef(false)
	const isResizingRef = useRef(false)
	const rafRef = useRef<number | null>(null)
	const dragOffsetRef = useRef({ x: 0, y: 0 })
	const resizeOriginRef = useRef({ mouseX: 0, mouseY: 0, width: 0, height: 0 })
	const onMoveRef = useRef<((e: MouseEvent) => void) | null>(null)
	const onUpRef = useRef<((e: MouseEvent) => void) | null>(null)

	useEffect(() => {
		const x = Math.max(0, window.innerWidth - 750)
		const y = Math.max(0, window.innerHeight - 650)
		posRef.current = { x, y }
		setState((s) => ({ ...s, x, y }))
	}, [])

	const flush = useCallback(() => {
		setState((s) => ({
			...s,
			x: posRef.current.x,
			y: posRef.current.y,
			width: sizeRef.current.width,
			height: sizeRef.current.height,
		}))
	}, [])

	const scheduleFlush = useCallback(() => {
		if (rafRef.current !== null) return
		rafRef.current = requestAnimationFrame(() => {
			rafRef.current = null
			flush()
		})
	}, [flush])

	const onMouseMove = useCallback(
		(e: MouseEvent) => {
			if (isDraggingRef.current) {
				const w = sizeRef.current.width
				posRef.current = {
					// window can't go off left/right edges
					x: Math.max(
						0,
						Math.min(e.clientX - dragOffsetRef.current.x, window.innerWidth - w)
					),
					// "New message" header must always stay inside viewport top/bottom
					y: Math.max(
						0,
						Math.min(
							e.clientY - dragOffsetRef.current.y,
							window.innerHeight - COMPOSE_HEADER_H
						)
					),
				}
				scheduleFlush()
			}

			if (isResizingRef.current) {
				sizeRef.current = {
					width: Math.max(
						450,
						resizeOriginRef.current.width + (e.clientX - resizeOriginRef.current.mouseX)
					),
					height: Math.max(
						400,
						resizeOriginRef.current.height +
							(e.clientY - resizeOriginRef.current.mouseY)
					),
				}
				scheduleFlush()
			}
		},
		[scheduleFlush]
	)

	const onMouseUp = useCallback(() => {
		isDraggingRef.current = false
		isResizingRef.current = false

		if (rafRef.current !== null) {
			cancelAnimationFrame(rafRef.current)
			rafRef.current = null
		}
		flush()
		setState((s) => ({ ...s, isDragging: false, isResizing: false }))

		if (onMoveRef.current) document.removeEventListener('mousemove', onMoveRef.current)
		if (onUpRef.current) document.removeEventListener('mouseup', onUpRef.current)
	}, [flush])

	// Keep stable refs in sync with latest callbacks
	useEffect(() => {
		onMoveRef.current = onMouseMove
		onUpRef.current = onMouseUp
	}, [onMouseMove, onMouseUp])

	const startDrag = useCallback((e: React.MouseEvent) => {
		const target = e.target as HTMLElement
		if (
			target.closest('button') ||
			target.closest('[role="button"]') ||
			target.closest('input') ||
			target.closest('textarea') ||
			target.closest('[contenteditable]')
		)
			return

		e.preventDefault()
		dragOffsetRef.current = {
			x: e.clientX - posRef.current.x,
			y: e.clientY - posRef.current.y,
		}
		isDraggingRef.current = true
		setState((s) => ({ ...s, isDragging: true }))

		document.addEventListener('mousemove', onMoveRef.current!)
		document.addEventListener('mouseup', onUpRef.current!)
	}, [])

	const handleResizeMouseDown = useCallback((e: React.MouseEvent) => {
		e.preventDefault()
		e.stopPropagation()
		resizeOriginRef.current = {
			mouseX: e.clientX,
			mouseY: e.clientY,
			width: sizeRef.current.width,
			height: sizeRef.current.height,
		}
		isResizingRef.current = true
		setState((s) => ({ ...s, isResizing: true }))

		document.addEventListener('mousemove', onMoveRef.current!)
		document.addEventListener('mouseup', onUpRef.current!)
	}, [])

	// Block text selection + all hover states app-wide during drag/resize
	useEffect(() => {
		if (state.isDragging || state.isResizing) {
			document.body.style.userSelect = 'none'
			document.body.classList.add('is-dragging')
		} else {
			document.body.style.userSelect = ''
			document.body.classList.remove('is-dragging')
		}
		return () => {
			document.body.style.userSelect = ''
			document.body.classList.remove('is-dragging')
		}
	}, [state.isDragging, state.isResizing])

	// Cleanup on unmount
	useEffect(() => {
		return () => {
			if (rafRef.current !== null) cancelAnimationFrame(rafRef.current)
			if (onMoveRef.current) document.removeEventListener('mousemove', onMoveRef.current)
			if (onUpRef.current) document.removeEventListener('mouseup', onUpRef.current)
			document.body.style.userSelect = ''
			document.body.classList.remove('is-dragging')
		}
	}, [])

	return {
		position: { x: state.x, y: state.y },
		size: { width: state.width, height: state.height },
		isDragging: state.isDragging,
		isResizing: state.isResizing,
		startDrag,
		handleResizeMouseDown,
	}
}

export const useLinkTooltip = (editorRef: React.RefObject<HTMLDivElement | null>) => {
	const [linkTooltipVisible, setLinkTooltipVisible] = useState(false)
	const [linkTooltipUrl, setLinkTooltipUrl] = useState('')
	const [linkTooltipRect, setLinkTooltipRect] = useState<DOMRect | null>(null)
	const [linkTooltipNode, setLinkTooltipNode] = useState<HTMLAnchorElement | null>(null)
	const linkTooltipRaf = useRef<number | null>(null)
	const hideTimeout = useRef<ReturnType<typeof setTimeout> | null>(null)
	const showTimeout = useRef<ReturnType<typeof setTimeout> | null>(null)
	const visibleRef = useRef(false)

	useEffect(() => {
		const clear = () => {
			if (showTimeout.current) {
				clearTimeout(showTimeout.current)
				showTimeout.current = null
			}
			setLinkTooltipVisible(false)
			visibleRef.current = false
			setLinkTooltipUrl('')
			setLinkTooltipRect(null)
			setLinkTooltipNode(null)
		}

		const onMouseMove = (e: Event) => {
			const me = e as MouseEvent
			if (linkTooltipRaf.current) cancelAnimationFrame(linkTooltipRaf.current)

			linkTooltipRaf.current = requestAnimationFrame(() => {
				let target = me.target as Node | null
				if (!target) return

				if (target.nodeType === 3) target = target.parentElement
				const el = target as HTMLElement

				if (!el || typeof el.closest !== 'function') return

				if (el.closest('.link-edit-tooltip')) {
					if (hideTimeout.current) {
						clearTimeout(hideTimeout.current)
						hideTimeout.current = null
					}
					return
				}

				const anchor = el.closest('a') as HTMLAnchorElement | null
				const container = editorRef.current

				if (
					anchor &&
					container &&
					(container instanceof Document || container.contains(anchor))
				) {
					if (hideTimeout.current) {
						clearTimeout(hideTimeout.current)
						hideTimeout.current = null
					}

					const href = anchor.getAttribute('href') || anchor.href || ''

					const rect = anchor.getBoundingClientRect()
					if (!visibleRef.current || linkTooltipUrl !== href) {
						if (!showTimeout.current) {
							showTimeout.current = setTimeout(() => {
								setLinkTooltipUrl(href)
								setLinkTooltipRect(rect)
								setLinkTooltipNode(anchor)
								setLinkTooltipVisible(true)
								visibleRef.current = true
								showTimeout.current = null
							}, 300)
						}
					}
				} else {
					if (showTimeout.current) {
						clearTimeout(showTimeout.current)
						showTimeout.current = null
					}
					if (!hideTimeout.current && visibleRef.current) {
						hideTimeout.current = setTimeout(() => {
							clear()
							hideTimeout.current = null
						}, 300)
					}
				}
			})
		}

		document.addEventListener('mousemove', onMouseMove)

		return () => {
			if (linkTooltipRaf.current) cancelAnimationFrame(linkTooltipRaf.current)
			if (hideTimeout.current) clearTimeout(hideTimeout.current)
			document.removeEventListener('mousemove', onMouseMove)
		}
	}, [editorRef])

	return {
		visible: linkTooltipVisible,
		url: linkTooltipUrl,
		rect: linkTooltipRect,
		node: linkTooltipNode,
	}
}
