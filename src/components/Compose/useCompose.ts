import { useState, useEffect, useRef, useCallback } from 'react'

export const useDragging = () => {
	const [isDragging, setIsDragging] = useState(false)
	const [dragStart, setDragStart] = useState({ x: 0, y: 0 })
	const [isResizing, setIsResizing] = useState(false)
	const [resizeStart, setResizeStart] = useState({ mouseX: 0, mouseY: 0, width: 0, height: 0 })

	const [position, setPosition] = useState({
		x: window.innerWidth - 720,
		y: window.innerHeight - 650,
	})
	const [size, setSize] = useState({ width: 672, height: 600 })

	const handleResizeMouseDown = (e: React.MouseEvent) => {
		e.preventDefault()
		setIsResizing(true)
		setResizeStart({
			mouseX: e.clientX,
			mouseY: e.clientY,
			width: size.width,
			height: size.height,
		})
	}

	const handleMouseMove = useCallback(
		(e: MouseEvent) => {
			if (isDragging) {
				const newX = Math.max(
					0,
					Math.min(e.clientX - dragStart.x, window.innerWidth - size.width)
				)
				const newY = Math.max(
					0,
					Math.min(e.clientY - dragStart.y, window.innerHeight - size.height)
				)
				setPosition({ x: newX, y: newY })
			}
			if (isResizing) {
				const newWidth = Math.max(450, resizeStart.width + (e.clientX - resizeStart.mouseX))
				const newHeight = Math.max(
					400,
					resizeStart.height + (e.clientY - resizeStart.mouseY)
				)
				setSize({ width: newWidth, height: newHeight })
			}
		},
		[isDragging, isResizing, dragStart, resizeStart, size.width, size.height]
	)

	const handleMouseUp = useCallback(() => {
		setIsDragging(false)
		setIsResizing(false)
	}, [])

	useEffect(() => {
		if (isDragging || isResizing) {
			window.addEventListener('mousemove', handleMouseMove)
			window.addEventListener('mouseup', handleMouseUp)
		}
		return () => {
			window.removeEventListener('mousemove', handleMouseMove)
			window.removeEventListener('mouseup', handleMouseUp)
		}
	}, [isDragging, isResizing, handleMouseMove, handleMouseUp])

	const startDrag = (e: React.MouseEvent) => {
		setIsDragging(true)
		setDragStart({ x: e.clientX - position.x, y: e.clientY - position.y })
	}

	return { position, size, isDragging, isResizing, startDrag, handleResizeMouseDown }
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
