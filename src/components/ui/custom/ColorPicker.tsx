import { useState, useRef, useEffect, useCallback } from 'react'
import type { HSV, CustomColorPickerProps } from '@/types/components/ui'

const hexToHsv = (hex: string): HSV => {
	let r = 0,
		g = 0,
		b = 0
	if (hex.length === 7) {
		r = parseInt(hex.slice(1, 3), 16) / 255
		g = parseInt(hex.slice(3, 5), 16) / 255
		b = parseInt(hex.slice(5, 7), 16) / 255
	}
	const max = Math.max(r, g, b),
		min = Math.min(r, g, b)
	const d = max - min
	let h = 0
	const s = max === 0 ? 0 : d / max
	const v = max

	if (max !== min) {
		switch (max) {
			case r:
				h = (g - b) / d + (g < b ? 6 : 0)
				break
			case g:
				h = (b - r) / d + 2
				break
			case b:
				h = (r - g) / d + 4
				break
		}
		h /= 6
	}
	return { h: h * 360, s: s * 100, v: v * 100 }
}

const hsvToHex = ({ h, s, v }: HSV): string => {
	s /= 100
	v /= 100
	const i = Math.floor(h / 60)
	const f = h / 60 - i
	const p = v * (1 - s)
	const q = v * (1 - f * s)
	const t = v * (1 - (1 - f) * s)
	let r = 0,
		g = 0,
		b = 0
	switch (i % 6) {
		case 0:
			r = v
			g = t
			b = p
			break
		case 1:
			r = q
			g = v
			b = p
			break
		case 2:
			r = p
			g = v
			b = t
			break
		case 3:
			r = p
			g = q
			b = v
			break
		case 4:
			r = t
			g = p
			b = v
			break
		case 5:
			r = v
			g = p
			b = q
			break
	}
	const toHex = (c: number) =>
		Math.round(c * 255)
			.toString(16)
			.padStart(2, '0')
	return `#${toHex(r)}${toHex(g)}${toHex(b)}`
}

export const ColorPicker = ({ color, onChange }: CustomColorPickerProps) => {
	const [hsv, setHsv] = useState(() => hexToHsv(color))
	const svRef = useRef<HTMLDivElement>(null)
	const hRef = useRef<HTMLDivElement>(null)

	useEffect(() => {
		const currentHex = hsvToHex(hsv)
		if (color.toLowerCase() !== currentHex.toLowerCase()) {
			setHsv(hexToHsv(color))
		}
	}, [color])

	const handleSvPointer = useCallback(
		(e: React.PointerEvent | PointerEvent) => {
			if (!svRef.current) return
			const rect = svRef.current.getBoundingClientRect()
			const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
			const y = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height))

			const newHsv = { ...hsv, s: x * 100, v: (1 - y) * 100 }
			setHsv(newHsv)
			onChange(hsvToHex(newHsv))
		},
		[hsv, onChange]
	)

	const handleHPointer = useCallback(
		(e: React.PointerEvent | PointerEvent) => {
			if (!hRef.current) return
			const rect = hRef.current.getBoundingClientRect()
			const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))

			const newHsv = { ...hsv, h: x * 360 }
			setHsv(newHsv)
			onChange(hsvToHex(newHsv))
		},
		[hsv, onChange]
	)

	const onSvDown = (e: React.PointerEvent) => {
		;(e.target as HTMLElement).setPointerCapture(e.pointerId)
		handleSvPointer(e)
	}

	const onHDown = (e: React.PointerEvent) => {
		;(e.target as HTMLElement).setPointerCapture(e.pointerId)
		handleHPointer(e)
	}

	return (
		<div className='flex w-[200px] flex-col gap-4 select-none'>
			{/* SV Square */}
			<div
				ref={svRef}
				onPointerDown={onSvDown}
				onPointerMove={(e) => e.buttons > 0 && handleSvPointer(e)}
				className='relative h-[180px] w-full cursor-crosshair overflow-hidden rounded-lg border border-[var(--border-faint)]'
				style={{ backgroundColor: hsvToHex({ h: hsv.h, s: 100, v: 100 }) }}>
				<div className='absolute inset-0 bg-gradient-to-r from-white to-transparent' />
				<div className='absolute inset-0 bg-gradient-to-t from-black to-transparent' />

				{/* Pointer */}
				<div
					className='pointer-events-none absolute h-3 w-3 -translate-x-1/2 translate-y-1/2 rounded-full border-2 border-white shadow-md'
					style={{
						left: `${hsv.s}%`,
						bottom: `${hsv.v}%`,
						backgroundColor: hsvToHex(hsv),
					}}
				/>
			</div>

			{/* Hue Slider */}
			<div
				ref={hRef}
				onPointerDown={onHDown}
				onPointerMove={(e) => e.buttons > 0 && handleHPointer(e)}
				className='relative h-3 w-full cursor-ew-resize rounded-full border border-[var(--border-faint)]'
				style={{
					background:
						'linear-gradient(to right, #f00 0%, #ff0 17%, #0f0 33%, #0ff 50%, #00f 67%, #f0f 83%, #f00 100%)',
				}}>
				{/* Thumb */}
				<div
					className='pointer-events-none absolute top-1/2 h-4 w-4 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-slate-900 bg-white shadow-md'
					style={{ left: `${(hsv.h / 360) * 100}%` }}
				/>
			</div>
		</div>
	)
}
