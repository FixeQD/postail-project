import { useState, useCallback, memo, useEffect, useRef } from 'react'
import { Virtuoso, type VirtuosoHandle } from 'react-virtuoso'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { Search } from 'lucide-react'
import { useThemeStore } from '@/stores/themeStore'

import { useContactsTranslation } from '@/hooks/useTypedTranslation'
import type { Contact } from '@/types/components/compose'

interface ContactListProps {
	selectedContact: Contact | null
	onSelect: (contact: Contact) => void
	selectedGroupId: number | null
}

const generateInitials = (name: string | null, email: string) => {
	if (!name) return email.slice(0, 2).toUpperCase()
	const parts = name.trim().split(/\s+/)
	if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase()
	return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase()
}

export const ContactList = memo(function ContactList({
	selectedContact,
	onSelect,
	selectedGroupId,
}: ContactListProps) {
	const { t } = useContactsTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const virtuosoRef = useRef<VirtuosoHandle>(null)
	const [searchQuery, setSearchQuery] = useState('')

	// Fetch all contacts (list) or search results
	const {
		data: contacts,
		isLoading,
		error,
	} = useQuery({
		queryKey: ['contacts-list', searchQuery, selectedGroupId],
		queryFn: async () => {
			if (selectedGroupId) {
				return await invoke<Contact[]>('get_contacts_in_group', {
					groupId: selectedGroupId,
				})
			}
			if (searchQuery.trim().length === 0) {
				return await invoke<Contact[]>('list_contacts')
			}
			return await invoke<Contact[]>('search_contacts_full', {
				query: searchQuery.trim(),
				limit: 50,
			})
		},
		enabled: true,
	})

	// Handle search input debouncing could be added if needed
	const handleSearchChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
		setSearchQuery(e.target.value)
	}, [])

	// Reset scroll on new data
	useEffect(() => {
		if (virtuosoRef.current) {
			virtuosoRef.current.scrollToIndex({ index: 0 })
		}
	}, [searchQuery])

	const handleContactClick = useCallback(
		(contact: Contact) => {
			onSelect(contact)
		},
		[onSelect]
	)

	const renderItem = useCallback(
		(_index: number, contact: Contact) => {
			const isSelected = selectedContact?.id === contact.id
			const initials = generateInitials(contact.name, contact.email)

			const itemBase = `group flex items-center gap-3 px-3 py-2.5 cursor-pointer transition-colors ${
				isSelected ? 'bg-[var(--surface-active)]' : 'hover:bg-[var(--surface-hover)]'
			}`

			const avatarStyle: React.CSSProperties = {
				backgroundColor: 'rgba(var(--accent-rgb), 0.12)',
				color: 'rgb(var(--accent-rgb))',
			}

			const selectedIndicatorStyle: React.CSSProperties = {
				backgroundColor: accentColor,
				boxShadow: `0 0 8px ${accentColor}80`,
			}

			return (
				<div className={itemBase} onClick={() => handleContactClick(contact)}>
					<div
						className='flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-[11px] font-semibold'
						style={avatarStyle}>
						{initials}
					</div>
					<div className='min-w-0 flex-1'>
						<p className='truncate text-[13px] font-medium text-[var(--text-primary)]'>
							{contact.name || contact.email}
						</p>
						<p className='truncate text-[11px] text-[var(--text-tertiary)]'>
							{contact.email}
						</p>
						{contact.company && (
							<p className='truncate text-[10px] text-[var(--text-tertiary)]'>
								{contact.company}
							</p>
						)}
					</div>
					{isSelected && (
						<div
							className='h-3 w-1 shrink-0 rounded-full'
							style={selectedIndicatorStyle}
						/>
					)}
				</div>
			)
		},
		[selectedContact, accentColor, handleContactClick]
	)

	if (error) {
		return (
			<div className='flex flex-1 items-center justify-center p-4'>
				<p className='text-[12px] text-red-500'>{t('common:errors.loadFailed')}</p>
			</div>
		)
	}

	return (
		<div className='flex h-full flex-col'>
			{/* Search */}
			<div className='p-2'>
				<div className='relative'>
					<Search className='absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-tertiary)]' />
					<input
						type='text'
						placeholder={t('contacts:search.placeholder')}
						value={searchQuery}
						onChange={handleSearchChange}
						className='w-full rounded-lg bg-[var(--surface-active)] py-1.5 pr-3 pl-8 text-[13px] text-[var(--text-primary)] ring-1 ring-transparent transition-all outline-none placeholder:text-[var(--text-tertiary)] focus:ring-[rgba(var(--accent-rgb),0.4)]'
					/>
				</div>
			</div>

			{/* Contact list */}
			<div className='flex-1 overflow-hidden'>
				{isLoading ? (
					<div className='flex items-center justify-center py-8'>
						<div
							className='h-5 w-5 animate-spin rounded-full border-2 border-transparent'
							style={{ borderTopColor: 'rgb(var(--accent-rgb))' }}
						/>
					</div>
				) : contacts && contacts.length > 0 ? (
					<Virtuoso
						ref={virtuosoRef}
						data={contacts}
						itemContent={renderItem}
						overscan={100}
						components={{ Footer: () => <div className='h-1' /> }}
					/>
				) : (
					<div className='flex flex-col items-center justify-center gap-3 p-6 text-center'>
						<div
							className='flex h-10 w-10 items-center justify-center rounded-xl'
							style={{ backgroundColor: 'rgba(var(--accent-rgb), 0.06)' }}>
							<Search
								className='h-4 w-4'
								style={{ color: 'rgba(var(--accent-rgb), 0.5)' }}
							/>
						</div>
						<div className='flex flex-col gap-1'>
							<p className='text-[13px] font-medium text-[var(--text-primary)]'>
								{searchQuery.trim().length > 0
									? t('contacts:empty.noResults.title')
									: t('contacts:empty.noContacts.title')}
							</p>
							<p className='text-[12px] text-[var(--text-tertiary)]'>
								{searchQuery.trim().length > 0
									? t('contacts:empty.noResults.description')
									: t('contacts:empty.noContacts.description')}
							</p>
						</div>
					</div>
				)}
			</div>
		</div>
	)
})
