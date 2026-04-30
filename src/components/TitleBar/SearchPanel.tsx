import { motion, AnimatePresence } from 'framer-motion'
import {
	User,
	AtSign,
	Type,
	AlignLeft,
	Calendar,
	FolderOpen,
	Paperclip,
	ChevronDown,
	Search,
} from 'lucide-react'
import type { AdvancedSearchQuery } from '@/types/search'
import type { Mailbox } from '@/types/mail'
import { SearchField } from './SearchField'

const EASE_OUT_EXPO: [number, number, number, number] = [0.16, 1, 0.3, 1]

interface SearchPanelProps {
	open: boolean
	query: AdvancedSearchQuery
	accentColor: string
	animationsEnabled: boolean
	mailboxes?: Mailbox[]
	updateField: <K extends keyof AdvancedSearchQuery>(field: K, value: AdvancedSearchQuery[K]) => void
	onClear: () => void
	onSubmit: () => void
	t: (key: string) => string
}

export function SearchPanel({
	open,
	query,
	accentColor,
	animationsEnabled,
	mailboxes,
	updateField,
	onClear,
	onSubmit,
	t,
}: SearchPanelProps) {
	const motionProps = animationsEnabled
		? { whileHover: { scale: 1.05 }, whileTap: { scale: 0.95 } }
		: {}

	return (
		<AnimatePresence>
			{open && (
				<motion.div
					key='advanced-panel'
					initial={
						animationsEnabled
							? { opacity: 0, y: -8, scale: 0.97, filter: 'blur(4px)' }
							: {}
					}
					animate={
						animationsEnabled
							? { opacity: 1, y: 0, scale: 1, filter: 'blur(0px)' }
							: {}
					}
					exit={
						animationsEnabled
							? { opacity: 0, y: -8, scale: 0.97, filter: 'blur(4px)' }
							: {}
					}
					transition={{ duration: 0.22, ease: EASE_OUT_EXPO }}
					className='glass absolute top-[calc(100%+6px)] right-0 left-0 z-50 rounded-2xl border border-[var(--border-subtle)] p-4 shadow-2xl backdrop-blur-xl'
					style={{
						boxShadow: `0 20px 60px rgba(0,0,0,0.5), 0 0 0 1px var(--border-subtle)`,
						backgroundImage: `linear-gradient(to bottom, ${accentColor}0A, ${accentColor}1A)`,
					}}
					onMouseDown={(e) => e.stopPropagation()}>
					{/* Accent top bar */}
					<div
						className='absolute inset-x-0 top-0 h-[2px] rounded-t-2xl'
						style={{
							background: `linear-gradient(90deg, transparent, ${accentColor}, transparent)`,
						}}
					/>

					<div className='grid grid-cols-2 gap-3'>
						<SearchField
							icon={<User className='h-3.5 w-3.5' />}
							label={t('inbox:search.fields.from')}
							value={query.from ?? ''}
							onChange={(v) => updateField('from', v || undefined)}
							placeholder='sender@example.com'
							accentColor={accentColor}
						/>
						<SearchField
							icon={<AtSign className='h-3.5 w-3.5' />}
							label={t('inbox:search.fields.to')}
							value={query.to ?? ''}
							onChange={(v) => updateField('to', v || undefined)}
							placeholder='recipient@example.com'
							accentColor={accentColor}
						/>
						<SearchField
							icon={<Type className='h-3.5 w-3.5' />}
							label={t('inbox:search.fields.subject')}
							value={query.subject ?? ''}
							onChange={(v) => updateField('subject', v || undefined)}
							placeholder={t('inbox:search.fields.subject')}
							accentColor={accentColor}
							className='col-span-2'
						/>
						<SearchField
							icon={<AlignLeft className='h-3.5 w-3.5' />}
							label={t('inbox:search.fields.body')}
							value={query.body ?? ''}
							onChange={(v) => updateField('body', v || undefined)}
							placeholder={t('inbox:search.fields.body')}
							accentColor={accentColor}
							className='col-span-2'
						/>
						<SearchField
							icon={<Calendar className='h-3.5 w-3.5' />}
							label={t('inbox:search.fields.dateFrom')}
							value={query.dateFrom ?? ''}
							onChange={(v) => updateField('dateFrom', v || undefined)}
							type='date'
							accentColor={accentColor}
						/>
						<SearchField
							icon={<Calendar className='h-3.5 w-3.5' />}
							label={t('inbox:search.fields.dateTo')}
							value={query.dateTo ?? ''}
							onChange={(v) => updateField('dateTo', v || undefined)}
							type='date'
							accentColor={accentColor}
						/>

						{/* Folder select */}
						<div className='flex flex-col gap-1'>
							<label className='flex items-center gap-1.5 text-[11px] font-medium text-[var(--text-secondary)]'>
								<FolderOpen className='h-3.5 w-3.5' />
								{t('inbox:search.fields.folder')}
							</label>
							<div className='relative flex items-center'>
								<select
									value={query.folder ?? ''}
									onChange={(e) => updateField('folder', e.target.value || undefined)}
									onFocus={(e) => {
										e.currentTarget.style.borderColor = accentColor
										e.currentTarget.style.boxShadow = `0 0 0 1px ${accentColor}`
									}}
									onBlur={(e) => {
										e.currentTarget.style.borderColor = 'var(--border-subtle)'
										e.currentTarget.style.boxShadow = 'none'
									}}
									className='h-8 w-full appearance-none rounded-lg border bg-[var(--surface-secondary)] px-3 pr-8 text-xs text-[var(--text-primary)] transition-all focus:outline-none'
									style={{
										borderColor: 'var(--border-subtle)',
										backgroundColor: 'var(--surface-secondary)',
										color: 'var(--text-primary)',
									}}>
									<option value=''>{t('inbox:search.fields.allFolders')}</option>
									{mailboxes?.map((mb) => (
										<option key={mb.name} value={mb.name}>
											{mb.display_name || mb.name}
										</option>
									))}
								</select>
								<ChevronDown className='pointer-events-none absolute right-2.5 h-3.5 w-3.5 text-[var(--text-tertiary)]' />
							</div>
						</div>

						{/* Has attachment */}
						<div className='flex flex-col gap-1'>
							<label className='flex items-center gap-1.5 text-[11px] font-medium text-[var(--text-secondary)]'>
								<Paperclip className='h-3.5 w-3.5' />
								{t('inbox:search.fields.hasAttachment')}
							</label>
							<button
								type='button'
								onClick={() =>
									updateField('hasAttachment', query.hasAttachment ? undefined : true)
								}
								className='flex h-8 items-center gap-2 rounded-lg border px-3 text-xs transition-all'
								style={{
									borderColor: query.hasAttachment ? accentColor : 'var(--border-subtle)',
									backgroundColor: query.hasAttachment
										? `${accentColor}18`
										: 'var(--surface-secondary)',
									color: query.hasAttachment ? accentColor : 'var(--text-secondary)',
								}}>
								<div
									className='flex h-3.5 w-3.5 items-center justify-center rounded-full border-2 transition-all'
									style={{
										borderColor: query.hasAttachment
											? accentColor
											: 'var(--border-strong)',
									}}>
									{query.hasAttachment && (
										<div
											className='h-2 w-2 rounded-full'
											style={{ backgroundColor: accentColor }}
										/>
									)}
								</div>
								{t('inbox:search.fields.hasAttachment')}
							</button>
						</div>
					</div>

					{/* Panel footer */}
					<div className='mt-3 flex items-center justify-between border-t border-[var(--border-subtle)] pt-3'>
						<button
							type='button'
							onClick={onClear}
							className='text-xs text-[var(--text-tertiary)] transition-colors hover:text-[var(--text-secondary)]'>
							{t('inbox:search.actions.clear')}
						</button>

						<motion.button
							type='button'
							onClick={onSubmit}
							{...motionProps}
							className='flex h-8 items-center gap-2 rounded-xl px-4 text-xs font-semibold text-white transition-all'
							style={{ backgroundColor: accentColor }}>
							<Search className='h-3.5 w-3.5' />
							{t('inbox:search.actions.search')}
						</motion.button>
					</div>
				</motion.div>
			)}
		</AnimatePresence>
	)
}
