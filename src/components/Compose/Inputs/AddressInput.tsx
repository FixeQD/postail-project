import React, { useState, useRef, KeyboardEvent, useEffect } from 'react'
import { X, User, Users } from 'lucide-react'
import { motion } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { cn } from '@/lib/utils'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import type { AddressInputProps, Contact } from '@/types/components/compose'

export function AddressInput({
	label,
	recipients,
	onAdd,
	onRemove,
	placeholder,
	className,
	rightElement,
}: AddressInputProps) {
	const animationsEnabled = useAnimationsEnabled()
	const [inputValue, setInputValue] = useState('')
	const [isFocused, setIsFocused] = useState(false)
	const [isHovered, setIsHovered] = useState(false)
	const [suggestions, setSuggestions] = useState<Contact[]>([])
	const [selectedIndex, setSelectedIndex] = useState(0)
	const inputRef = useRef<HTMLInputElement>(null)

	useEffect(() => {
		if (inputValue.length < 1) {
			setSuggestions([])
			return
		}

		const fetchSuggestions = async () => {
			try {
				const { contacts, groups } = await invoke<{ contacts: Contact[], groups: any[] }>('search_contacts_and_groups', {
					query: inputValue,
					limit: 10,
				})

				// Map groups to a common suggestion format
				const groupSuggestions = groups.map(g => ({
					id: `group-${g.id}`,
					email: `group:${g.id}`, // Internal marker
					name: g.name,
					isGroup: true,
					memberCount: g.member_count,
					color: g.color
				}))

				// Filter out already added recipients (contacts only)
				const filteredContacts = contacts.filter((c) => !recipients.some((r) => r.email === c.email))
				
				setSuggestions([...groupSuggestions, ...filteredContacts] as any[])
				setSelectedIndex(0)
			} catch (err) {
				console.error('Failed to fetch suggestions:', err)
			}
		}

		const timer = setTimeout(fetchSuggestions, 100)
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

	const handleAddRecipientWithContact = async (suggestion: any) => {
		if (suggestion.isGroup) {
			try {
				const groupId = parseInt(suggestion.email.split(':')[1])
				const members = await invoke<Contact[]>('get_contacts_in_group', { groupId })
				
				// Add each member if not already present
				members.forEach(member => {
					if (!recipients.some(r => r.email === member.email)) {
						onAdd({ email: member.email, name: member.name || undefined })
					}
				})
			} catch (err) {
				console.error('Failed to expand group:', err)
			}
		} else {
			onAdd({ email: suggestion.email, name: suggestion.name || undefined })
		}
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
			{...(animationsEnabled
				? {
						transition: { duration: 0.2 },
						onHoverStart: () => setIsHovered(true),
						onHoverEnd: () => setIsHovered(false),
					}
				: {})}
			onMouseEnter={animationsEnabled ? undefined : () => setIsHovered(true)}
			onMouseLeave={animationsEnabled ? undefined : () => setIsHovered(false)}
			className={cn(
				'relative flex min-h-11 w-full flex-wrap items-center gap-2 border-b border-[var(--compose-input-border)] bg-transparent px-0 py-1.5 transition-colors',
				className
			)}
			onClick={() => inputRef.current?.focus()}>
			<span className='mr-1 text-sm font-medium text-[var(--compose-text-muted)] select-none'>
				{label}
			</span>

			{recipients.map((recipient) => (
				<motion.div
					key={recipient.email}
					{...(animationsEnabled
						? {
								initial: { opacity: 0, scale: 0.8 },
								animate: { opacity: 1, scale: 1 },
								exit: { opacity: 0, scale: 0.8 },
								transition: { duration: 0.15 },
							}
						: {})}
					className='flex items-center gap-1.5 rounded-full bg-[var(--compose-chip-bg)] py-0.5 pr-1 pl-2.5 text-sm text-[var(--compose-text)] ring-1 ring-[var(--compose-chip-border)] hover:bg-[var(--compose-active)]'
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
						className='flex h-4 w-4 items-center justify-center rounded-full text-[var(--compose-text-muted)] hover:bg-[var(--compose-hover)] hover:text-[var(--compose-text)]'>
						<X className='h-3 w-3' />
					</button>
				</motion.div>
			))}

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
				className='min-w-[120px] flex-1 bg-transparent py-1 text-sm text-[var(--compose-text)] outline-none placeholder:text-[var(--compose-placeholder)]'
			/>

			{rightElement && <div className='ml-auto flex items-center pl-2'>{rightElement}</div>}

			{isFocused && suggestions.length > 0 && (
				<motion.div
					{...(animationsEnabled
						? {
								initial: { opacity: 0, y: -10 },
								animate: { opacity: 1, y: 0 },
								exit: { opacity: 0, y: -10 },
							}
						: {})}
					className='absolute top-full left-0 z-[60] mt-1 w-full overflow-hidden rounded-lg border border-[var(--compose-ring)] bg-[var(--compose-suggestions-bg)] shadow-xl'>
					{suggestions.map((suggestion: any, index) => (
						<div
							key={suggestion.id}
							className={cn(
								'flex cursor-pointer items-center gap-3 px-4 py-3 transition-colors',
								index === selectedIndex
									? 'bg-[var(--compose-active)]'
									: 'hover:bg-[var(--compose-hover)]'
							)}
							onMouseDown={(e) => {
								e.preventDefault()
								handleAddRecipientWithContact(suggestion)
							}}>
							<div className={cn(
								'flex h-8 w-8 items-center justify-center rounded-full text-white font-bold text-[10px]',
								suggestion.isGroup ? 'bg-indigo-500' : 'bg-[var(--compose-chip-bg)] text-[var(--compose-text-muted)]'
							)}
							style={suggestion.isGroup && suggestion.color ? { backgroundColor: suggestion.color } : undefined}
							>
								{suggestion.isGroup ? (
									<Users className='h-4 w-4' />
								) : (
									<User className='h-4 w-4' />
								)}
							</div>
							<div className='flex flex-col'>
								<span className='text-sm font-medium text-[var(--compose-text)]'>
									{suggestion.name || suggestion.email.split('@')[0]}
								</span>
								<span className='text-xs text-[var(--compose-text-muted)]'>
									{suggestion.isGroup ? `${suggestion.memberCount} members` : suggestion.email}
								</span>
							</div>
						</div>
					))}
				</motion.div>
			)}

			<motion.div
				{...(animationsEnabled
					? {
							initial: { scaleX: 0 },
							animate: { scaleX: isFocused || isHovered ? 1 : 0 },
							transition: { duration: 0.25, ease: 'easeOut' },
						}
					: {
							style: { scaleX: isFocused || isHovered ? 1 : 0 },
						})}
				className='pointer-events-none absolute bottom-0 left-0 h-[1px] w-full origin-center bg-[var(--compose-focus-line)]'
			/>
		</motion.div>
	)
}
