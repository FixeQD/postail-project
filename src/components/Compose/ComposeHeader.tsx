import { X, Minimize2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTranslation } from 'react-i18next'

interface ComposeHeaderProps {
	isDragging: boolean
	onMouseDown: (e: React.MouseEvent) => void
	onClose: () => void
}

export function ComposeHeader({ isDragging, onMouseDown, onClose }: ComposeHeaderProps) {
	const { t } = useTranslation()

	return (
		<div
			className='flex w-full items-center justify-between rounded-t-xl bg-zinc-900 px-4 py-3 select-none'
			onMouseDown={onMouseDown}
			style={{ cursor: isDragging ? 'grabbing' : 'grab' }}>
			<h2 className='text-sm font-medium text-zinc-300'>{t('compose.newMessage')}</h2>
			<div className='flex items-center gap-1'>
				<Button variant='ghost' size='icon' className='h-7 w-7 text-zinc-400'>
					<Minimize2 className='h-4 w-4' />
				</Button>
				<Button
					variant='ghost'
					size='icon'
					className='h-7 w-7 text-zinc-400 hover:text-zinc-100'
					onClick={onClose}>
					<X className='h-4 w-4' />
				</Button>
			</div>
		</div>
	)
}
