import React, { useState, useRef, KeyboardEvent, useEffect } from 'react'
import { X, User } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { cn } from '@/lib/utils'
import type { EmailAddress } from '@/types/compose'

interface Contact {
	id: number
	email: string
	name: string | null
}

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
	const [isHovered, setIsHovered] = useState(false)
	const [suggestions, setSuggestions] = useState<Contact[]>([])
	const [selectedIndex, setSelectedIndex] = useState(0)
	const inputRef = useRef<HTMLInputElement>(null)

	useEffect(() => {
		if (inputValue.length < 2) {
			setSuggestions([])
			return
		}

		const fetchSuggestions = async () => {
			try {
				const results = await invoke<Contact[]>('search_contacts', {
					query: inputValue,
					limit: 5,
				})
				// Filter out already added recipients
				const filtered = results.filter((c) => !recipients.some((r) => r.email === c.email))
				setSuggestions(filtered)
				setSelectedIndex(0)
			} catch (err) {
				console.error('Failed to fetch suggestions:', err)
			}
		}

		const timer = setTimeout(fetchSuggestions, 150)
		return () => clearTimeout(timer)
	}, [inputValue, recipients])

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
			if (suggestions.length > 0 && isFocused) {
				e.preventDefault()
				const contact = suggestions[selectedIndex]
				handleAddRecipientWithContact(contact)
			} else if (inputValue) {
				e.preventDefault()
				handleAddRecipient(inputValue)
			}
		} else if (e.key === 'ArrowDown') {
			e.preventDefault()
			setSelectedIndex((prev) => (prev + 1) % suggestions.length)
		} else if (e.key === 'ArrowUp') {
			e.preventDefault()
			setSelectedIndex((prev) => (prev - 1 + suggestions.length) % suggestions.length)
		} else if (e.key === 'Backspace' && !inputValue && recipients.length > 0) {
			onRemove(recipients[recipients.length - 1].email)
		} else if (e.key === 'Escape') {
			setSuggestions([])
		}
	}

	const handleAddRecipientWithContact = (contact: Contact) => {
		onAdd({ email: contact.email, name: contact.name || undefined })
		setInputValue('')
		setSuggestions([])
	}

	const handlePaste = (e: React.ClipboardEvent) => {
		e.preventDefault()
		const pastedData = e.clipboardData.getData('text')
		const emails = pastedData.split(/[,\s]+/).filter(Boolean)
		emails.forEach((email) => handleAddRecipient(email))
	}

	return (
		<motion.div
			transition={{ duration: 0.2 }}
			onHoverStart={() => setIsHovered(true)}
			onHoverEnd={() => setIsHovered(false)}
			className={cn(
				'relative flex min-h-11 w-full flex-wrap items-center gap-2 border-b border-zinc-900 bg-transparent px-0 py-1.5 transition-colors',
				className
			)}
			onClick={() => inputRef.current?.focus()}>
			<span className='mr-1 text-sm font-medium text-zinc-500 select-none'>{label}</span>

			<AnimatePresence initial={false}>
				{recipients.map((recipient) => (
					<motion.div
						key={recipient.email}
						initial={{ opacity: 0, scale: 0.8 }}
						animate={{ opacity: 1, scale: 1 }}
						exit={{ opacity: 0, scale: 0.8 }}
						transition={{ duration: 0.15 }}
						className='flex items-center gap-1.5 rounded-full bg-zinc-800 py-0.5 pr-1 pl-2.5 text-sm text-zinc-200 ring-1 ring-zinc-700 hover:bg-zinc-700'
						onClick={(e) => e.stopPropagation()}>
						<span className='max-w-[200px] truncate'>
							{recipient.name || recipient.email}
						</span>
						<button
							type='button'
							onClick={(e) => {
								e.stopPropagation()
								onRemove(recipient.email)
							}}
							className='flex h-4 w-4 items-center justify-center rounded-full text-zinc-400 hover:bg-zinc-600 hover:text-white'>
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

			<AnimatePresence>
				{isFocused && suggestions.length > 0 && (
					<motion.div
						initial={{ opacity: 0, y: -10 }}
						animate={{ opacity: 1, y: 0 }}
						exit={{ opacity: 0, y: -10 }}
						className='absolute top-full left-0 z-[60] mt-1 w-full overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900 shadow-xl'>
						{suggestions.map((contact, index) => (
							<div
								key={contact.id}
								className={cn(
									'flex cursor-pointer items-center gap-3 px-4 py-3 transition-colors',
									index === selectedIndex ? 'bg-zinc-800' : 'hover:bg-zinc-800/50'
								)}
								onMouseDown={(e) => {
									e.preventDefault() // Prevents focus loss before selection
									handleAddRecipientWithContact(contact)
								}}>
								<div className='flex h-8 w-8 items-center justify-center rounded-full bg-zinc-800 text-zinc-400'>
									<User className='h-4 w-4' />
								</div>
								<div className='flex flex-col'>
									<span className='text-sm font-medium text-zinc-200'>
										{contact.name || contact.email.split('@')[0]}
									</span>
									<span className='text-xs text-zinc-500'>{contact.email}</span>
								</div>
							</div>
						))}
					</motion.div>
				)}
			</AnimatePresence>

			<motion.div
				initial={{ scaleX: 0 }}
				animate={{ scaleX: isFocused || isHovered ? 1 : 0 }}
				transition={{ duration: 0.25, ease: 'easeOut' }}
				className='pointer-events-none absolute bottom-0 left-0 h-[1px] w-full origin-center bg-zinc-500'
			/>
		</motion.div>
	)
}
