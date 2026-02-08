import { useThemeStore } from '@/stores/themeStore'
import type { MotionProps } from 'framer-motion'

const NOOP_MOTION: MotionProps = {
	initial: false,
	animate: undefined,
	exit: undefined,
	transition: { duration: 0 },
	whileHover: undefined,
	whileTap: undefined,
}

export function useMotion(props: MotionProps): MotionProps {
	const enabled = useThemeStore((s) => s.animationsEnabled)
	if (!enabled) return NOOP_MOTION
	return props
}

export function useAnimationsEnabled(): boolean {
	return useThemeStore((s) => s.animationsEnabled)
}
