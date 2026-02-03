import { useState, useEffect, useRef, useCallback } from 'react'

export interface DraggingState {
	x: number
	y: number
	width: number
	height: number
	isDragging: boolean
	isResizing: boolean
}

export const useDragging = () => {
	const [state, setState] = useState<DraggingState>({
		x: 100,
		y: 100,
		width: 672,
		height: 600,
		isDragging: false,
		isResizing: false,
	})

	const dragOffsetRef = useRef({ x: 0, y: 0 })
	const resizeStartRef = useRef({ mouseX: 0, mouseY: 0, width: 0, height: 0 })
	const isDraggingRef = useRef(false)
	const isResizingRef = useRef(false)

	useEffect(() => {
		setState((s) => ({
			...s,
			x: Math.max(50, window.innerWidth - 750),
			y: Math.max(50, window.innerHeight - 650),
		}))
	}, [])

	const widthRef = useRef(state.width)
	const heightRef = useRef(state.height)

	useEffect(() => {
		widthRef.current = state.width
		heightRef.current = state.height
	}, [state.width, state.height])

	const handleMouseMove = useCallback((e: MouseEvent) => {
		if (isDraggingRef.current) {
			const newX = Math.max(
				0,
				Math.min(e.clientX - dragOffsetRef.current.x, window.innerWidth - widthRef.current)
			)
			const newY = Math.max(
				0,
				Math.min(
					e.clientY - dragOffsetRef.current.y,
					window.innerHeight - heightRef.current
				)
			)
			setState((s) => ({ ...s, x: newX, y: newY }))
		}

		if (isResizingRef.current) {
			const newWidth = Math.max(
				450,
				resizeStartRef.current.width + (e.clientX - resizeStartRef.current.mouseX)
			)
			const newHeight = Math.max(
				400,
				resizeStartRef.current.height + (e.clientY - resizeStartRef.current.mouseY)
			)
			setState((s) => ({ ...s, width: newWidth, height: newHeight }))
		}
	}, [])

	const stopDrag = useCallback(() => {
		isDraggingRef.current = false
		isResizingRef.current = false
		setState((s) => ({ ...s, isDragging: false, isResizing: false }))
		window.removeEventListener('mousemove', handleMouseMove)
		window.removeEventListener('mouseup', stopDrag)
	}, [handleMouseMove])

	const startDrag = useCallback(
		(e: React.MouseEvent) => {
			const target = e.target as HTMLElement
			if (target.closest('button') || target.closest('[role="button"]')) {
				return
			}

			dragOffsetRef.current = {
				x: e.clientX - state.x,
				y: e.clientY - state.y,
			}
			isDraggingRef.current = true
			setState((s) => ({ ...s, isDragging: true }))
			window.addEventListener('mousemove', handleMouseMove)
			window.addEventListener('mouseup', stopDrag)
		},
		[state.x, state.y, handleMouseMove, stopDrag]
	)

	const handleResizeMouseDown = (e: React.MouseEvent) => {
		e.preventDefault()
		resizeStartRef.current = {
			mouseX: e.clientX,
			mouseY: e.clientY,
			width: state.width,
			height: state.height,
		}
		isResizingRef.current = true
		setState((s) => ({ ...s, isResizing: true }))
		window.addEventListener('mousemove', handleMouseMove)
		window.addEventListener('mouseup', stopDrag)
	}

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
	const linkTooltipRaf = useRef<number | null>(null)

	useEffect(() => {
		const container = editorRef.current || document
		const clear = () => {
			setLinkTooltipVisible(false)
			setLinkTooltipUrl('')
			setLinkTooltipRect(null)
		}

		const onPointerMove = (e: Event) => {
			const pe = e as PointerEvent
			if (linkTooltipRaf.current) cancelAnimationFrame(linkTooltipRaf.current)
			linkTooltipRaf.current = requestAnimationFrame(() => {
				const target = pe.target as HTMLElement | null
				if (!target) return clear()
				const anchor = target.closest
					? (target.closest('a') as HTMLAnchorElement | null)
					: null
				if (
					anchor &&
					(container instanceof Document || (container as HTMLElement).contains(anchor))
				) {
					const href = anchor.getAttribute('href') || anchor.href || ''
					setLinkTooltipUrl(href)
					setLinkTooltipRect(anchor.getBoundingClientRect())
					setLinkTooltipVisible(true)
				} else {
					clear()
				}
			})
		}

		container.addEventListener('pointermove', onPointerMove)

		return () => {
			if (linkTooltipRaf.current) cancelAnimationFrame(linkTooltipRaf.current)
			container.removeEventListener('pointermove', onPointerMove)
		}
	}, [editorRef])

	return {
		visible: linkTooltipVisible,
		url: linkTooltipUrl,
		rect: linkTooltipRect,
	}
}
