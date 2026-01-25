import { useState } from 'react'
import {
	X,
	Send,
	Paperclip,
	Bold,
	Italic,
	Underline,
	Link,
	List,
	ListOrdered,
	AlignLeft,
	MoreVertical,
	Trash2,
	Minimize2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

interface ComposeScreenProps {
	open: boolean
	onOpenChange: (open: boolean) => void
}

export function ComposeScreen({ open, onOpenChange }: ComposeScreenProps) {
	const [to, setTo] = useState('')
	const [subject, setSubject] = useState('')

	if (!open) return null

	return (
		<div className='fixed inset-0 z-50 flex items-end justify-end p-0 sm:items-center sm:justify-center sm:bg-black/60 sm:p-4 sm:backdrop-blur-sm'>
			{/* Modal Container */}
			<div className='flex h-[600px] w-full max-w-2xl flex-col overflow-hidden rounded-t-xl bg-zinc-950 text-zinc-100 shadow-2xl ring-1 ring-zinc-800 sm:rounded-xl'>
				{/* Header */}
				<div className='flex items-center justify-between bg-zinc-900 px-4 py-3'>
					<h2 className='text-sm font-medium text-zinc-300'>Nowa wiadomość</h2>
					<div className='flex items-center gap-1'>
						<Button
							variant='ghost'
							size='icon'
							className='h-6 w-6 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'>
							<Minimize2 className='h-4 w-4' />
						</Button>
						<Button
							variant='ghost'
							size='icon'
							onClick={() => onOpenChange(false)}
							className='h-6 w-6 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100'>
							<X className='h-4 w-4' />
						</Button>
					</div>
				</div>

				{/* Form Fields */}
				<div className='flex flex-col gap-1 px-4 pt-2'>
					<div className='relative'>
						<Input
							placeholder='Odbiorcy'
							value={to}
							onChange={(e) => setTo(e.target.value)}
							className='h-auto border-0 border-b border-zinc-800 bg-transparent px-0 py-3 text-sm placeholder:text-zinc-500 focus-visible:ring-0'
						/>
					</div>
					<div>
						<Input
							placeholder='Temat'
							value={subject}
							onChange={(e) => setSubject(e.target.value)}
							className='h-auto border-0 border-b border-zinc-800 bg-transparent px-0 py-3 text-sm font-medium placeholder:text-zinc-500 focus-visible:ring-0'
						/>
					</div>
				</div>

				{/* Editor Area */}
				<div className='flex-1 overflow-y-auto p-4'>
					<div
						className='h-full min-h-[200px] w-full resize-none border-0 bg-transparent text-sm text-zinc-200 outline-none'
						contentEditable
						suppressContentEditableWarning
						data-placeholder='Napisz coś...'
					/>
				</div>

				{/* Footer / Toolbar - Gmail Style */}
				<div className='mt-auto flex flex-col gap-2 border-t border-zinc-800 p-3'>
					{/* Formatting Icons Row */}
					<div className='flex items-center gap-1 overflow-x-auto pb-2 sm:pb-0'>
						<Button
							variant='ghost'
							size='icon'
							className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
							<Bold className='h-4 w-4' />
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
							<Italic className='h-4 w-4' />
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
							<Underline className='h-4 w-4' />
						</Button>
						<div className='mx-1 h-4 w-[1px] bg-zinc-800' />
						<Button
							variant='ghost'
							size='icon'
							className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
							<AlignLeft className='h-4 w-4' />
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
							<List className='h-4 w-4' />
						</Button>
						<Button
							variant='ghost'
							size='icon'
							className='h-8 w-8 text-zinc-400 hover:bg-zinc-800'>
							<ListOrdered className='h-4 w-4' />
						</Button>
					</div>

					{/* Action Row */}
					<div className='flex items-center justify-between pt-1'>
						<div className='flex items-center gap-2'>
							<Button className='rounded-full bg-blue-600 px-6 font-medium text-white hover:bg-blue-700'>
								Wyślij
							</Button>
							<Button
								variant='ghost'
								size='icon'
								className='h-9 w-9 text-zinc-400 hover:bg-zinc-800'>
								<Paperclip className='h-5 w-5' />
							</Button>
							<Button
								variant='ghost'
								size='icon'
								className='h-9 w-9 text-zinc-400 hover:bg-zinc-800'>
								<Link className='h-5 w-5' />
							</Button>
						</div>

						<div className='flex items-center gap-1'>
							<Button
								variant='ghost'
								size='icon'
								className='h-9 w-9 text-zinc-400 hover:bg-zinc-800 hover:text-red-400'>
								<Trash2 className='h-4 w-4' />
							</Button>
							<Button
								variant='ghost'
								size='icon'
								className='h-9 w-9 text-zinc-400 hover:bg-zinc-800'>
								<MoreVertical className='h-4 w-4' />
							</Button>
						</div>
					</div>
				</div>
			</div>
		</div>
	)
}
