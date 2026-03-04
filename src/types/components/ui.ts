export interface ToggleSettingProps {
	value: boolean
	onChange: (value: boolean) => void
	label: string
	description: string
	icon: import('lucide-react').LucideIcon
	disabled?: boolean
}

export interface ConfirmationDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	title: string
	description: string
	confirmLabel: string
	cancelLabel: string
	onConfirm: () => void
	children?: React.ReactNode
	confirmClassName?: string
}

export interface HSV {
	h: number
	s: number
	v: number
}

export interface CustomColorPickerProps {
	color: string
	onChange: (hex: string) => void
}
