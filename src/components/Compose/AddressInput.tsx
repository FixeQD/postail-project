import React, { useState, useRef, KeyboardEvent } from 'react'
import { X } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { cn } from '@/lib/utils'
import type { EmailAddress } from '@/types/compose'

interface AddressInputProps {
	label: string
	recipients: EmailAddress[]
	onAdd: (recipient: EmailAddress) => void
	onRemove: (email: string) => void
	placeholder?: string
	className?: string
	rightElement?: React.ReactNode
}

export function AddressInput({
	label,
	recipients,
	onAdd,
	onRemove,
	placeholder,
	className,
	rightElement,
}: AddressInputProps) {
	const [inputValue, setInputValue] = useState('')
	const [isFocused, setIsFocused] = useState(false)
	const inputRef = useRef<HTMLInputElement>(null)

	const validateEmail = (email: string) => {
		return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
	}

	const handleAddRecipient = (value: string) => {
		const email = value.trim().replace(/,$/, '')
		if (email && validateEmail(email)) {
			onAdd({ email })
			setInputValue('')
		}
	}

	const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
		if (e.key === 'Enter' || e.key === ',' || e.key === 'Tab') {
			if (inputValue) {
				e.preventDefault()
				handleAddRecipient(inputValue)
			}
		} else if (e.key === 'Backspace' && !inputValue && recipients.length > 0) {
			onRemove(recipients[recipients.length - 1].email)
		}
	}

	const handlePaste = (e: React.ClipboardEvent) => {
		e.preventDefault()
		const pastedData = e.clipboardData.getData('text')
		const emails = pastedData.split(/[,\s]+/).filter(Boolean)
		emails.forEach((email) => handleAddRecipient(email))
	}

	return (
		<div
			className={cn(
				'flex min-h-11 w-full flex-wrap items-center gap-2 border-b border-zinc-900 bg-transparent px-0 py-1.5 transition-colors',
				isFocused && 'border-zinc-700',
				className
			)}
			onClick={() => inputRef.current?.focus()}>
			<span className='mr-1 select-none text-sm font-medium text-zinc-500'>{label}</span>

			<AnimatePresence initial={false}>
				{recipients.map((recipient) => (
					<motion.div
						key={recipient.email}
						initial={{ opacity: 0, scale: 0.8 }}
						animate={{ opacity: 1, scale: 1 }}
						exit={{ opacity: 0, scale: 0.8 }}
						transition={{ duration: 0.15 }}
						className='flex items-center gap-1.5 rounded-full bg-zinc-800 py-0.5 pl-2.5 pr-1 text-sm text-zinc-200 ring-1 ring-zinc-700 hover:bg-zinc-700'
						onClick={(e) => e.stopPropagation()}>
						<span className='max-w-[200px] truncate'>{recipient.name || recipient.email}</span>
						<button
							type='button'
							onClick={(e) => {
								e.stopPropagation()
								onRemove(recipient.email)
							}}
							className='hover:bg-zinc-600 flex h-4 w-4 items-center justify-center rounded-full text-zinc-400 hover:text-white'>
							<X className='h-3 w-3' />
						</button>
					</motion.div>
				))}
			</AnimatePresence>

			<input
				ref={inputRef}
				type='text'
				value={inputValue}
				onChange={(e) => setInputValue(e.target.value)}
				onKeyDown={handleKeyDown}
				onFocus={() => setIsFocused(true)}
				onBlur={() => {
					setIsFocused(false)
					handleAddRecipient(inputValue)
				}}
				onPaste={handlePaste}
				placeholder={recipients.length === 0 ? placeholder : ''}
				className='min-w-[120px] flex-1 bg-transparent py-1 text-sm text-zinc-100 outline-none placeholder:text-zinc-600'
			/>

			{rightElement && <div className='ml-auto flex items-center pl-2'>{rightElement}</div>}
		</div>
	)
}
