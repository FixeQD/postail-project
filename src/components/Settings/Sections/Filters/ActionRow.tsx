import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { ChevronDown, X, FolderInput, Tag, Star, MailCheck, Trash2 } from 'lucide-react'
import { RuleAction, ActionType } from '@/types/filters'
import { Mailbox } from '@/types/mail'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'

interface ActionRowProps {
	action: RuleAction
	accountId: string
	onUpdate: (updates: Partial<RuleAction>) => void
	onRemove: () => void
	isOnly: boolean
}

function StyledSelect<T extends string>({
	value,
	onChange,
	options,
	className = '',
}: {
	value: T
	onChange: (v: T) => void
	options: { value: T; label: string }[]
	className?: string
}) {
	return (
		<div className={`relative ${className}`}>
			<select
				value={value}
				onChange={(e) => onChange(e.target.value as T)}
				className='h-9 w-full cursor-pointer appearance-none rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] pr-8 pl-3 text-xs font-medium text-[var(--text-primary)] transition-colors hover:bg-[var(--surface-hover)] focus:border-[var(--accent-primary)] focus:ring-1 focus:ring-[var(--accent-primary)]/20 focus:outline-none'>
				{options.map((o) => (
					<option
						key={o.value}
						value={o.value}
						className='bg-[var(--surface-panel)] text-[var(--text-primary)]'>
						{o.label}
					</option>
				))}
			</select>
			<ChevronDown className='pointer-events-none absolute top-1/2 right-2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-tertiary)]' />
		</div>
	)
}

const ACTION_ICONS: Record<ActionType, React.ReactNode> = {
	move_to: <FolderInput className='h-3.5 w-3.5' />,
	add_tag: <Tag className='h-3.5 w-3.5' />,
	star: <Star className='h-3.5 w-3.5' />,
	mark_read: <MailCheck className='h-3.5 w-3.5' />,
	delete: <Trash2 className='h-3.5 w-3.5' />,
}

const ACTION_COLORS: Record<ActionType, string> = {
	move_to: 'text-blue-400',
	add_tag: 'text-violet-400',
	star: 'text-yellow-400',
	mark_read: 'text-green-400',
	delete: 'text-red-400',
}

export function ActionRow({ action, accountId, onUpdate, onRemove, isOnly }: ActionRowProps) {
	const { t } = useTypedTranslation(['common', 'settings'])

	const { data: mailboxes } = useQuery<Mailbox[]>({
		queryKey: ['mailboxes', accountId],
		queryFn: () => invoke<Mailbox[]>('fetch_mailboxes', { accountId }),
		enabled: !!accountId,
	})

	const { data: tagColors = {} } = useQuery<Record<string, number>>({
		queryKey: ['tag-colors', accountId],
		queryFn: () => invoke('get_tag_colors', { accountId }),
		enabled: !!accountId,
	})
	const tags = Object.keys(tagColors).sort()

	const actionTypes: ActionType[] = ['move_to', 'add_tag', 'star', 'mark_read', 'delete']

	const actionTypeOptions = actionTypes.map((type) => ({
		value: type,
		label: t(`settings:filters.actionTypes.${type}`),
	}))

	const needsValue = action.action_type === 'move_to' || action.action_type === 'add_tag'

	return (
		<div className='group flex items-center gap-2'>
			{/* Action type icon */}
			<div className={`shrink-0 ${ACTION_COLORS[action.action_type]}`}>
				{ACTION_ICONS[action.action_type]}
			</div>

			<StyledSelect
				value={action.action_type}
				onChange={(v) => onUpdate({ action_type: v, value: '' })}
				options={actionTypeOptions}
				className='w-40 shrink-0'
			/>

			{action.action_type === 'move_to' && (
				<StyledSelect
					value={action.value ?? ''}
					onChange={(v) => onUpdate({ value: v })}
					options={[
						{ value: '', label: t('common:actions.select') || 'Select folder…' },
						...(mailboxes?.map((m) => ({ value: m.name, label: m.display_name })) ??
							[]),
					]}
					className='min-w-0 flex-1'
				/>
			)}

			{action.action_type === 'add_tag' && (
				<div className='flex min-w-0 flex-1 items-center gap-2'>
					<StyledSelect
						value={action.value ?? ''}
						onChange={(v) => onUpdate({ value: v })}
						options={[
							{ value: '', label: t('common:actions.select') || 'Select tag…' },
							...tags.map((tag) => ({ value: tag, label: tag })),
							{ value: '__NEW__', label: `+ New tag` },
						]}
						className='flex-1'
					/>
					{action.value === '__NEW__' && (
						<input
							autoFocus
							type='text'
							placeholder='Tag name…'
							onChange={(e) => onUpdate({ value: e.target.value })}
							className='h-9 w-32 rounded-lg border border-[var(--accent-primary)]/40 bg-[var(--surface-panel)] px-3 text-xs font-medium text-[var(--text-primary)] placeholder:text-[var(--text-tertiary)] focus:border-[var(--accent-primary)] focus:ring-1 focus:ring-[var(--accent-primary)]/20 focus:outline-none'
						/>
					)}
				</div>
			)}

			{!needsValue && (
				<div className='flex-1'>
					{action.action_type === 'delete' && (
						<span className='text-[11px] font-medium text-red-400/70'>permanent</span>
					)}
				</div>
			)}

			<button
				type='button'
				onClick={onRemove}
				disabled={isOnly}
				className='flex h-9 w-9 shrink-0 items-center justify-center rounded-lg text-[var(--text-tertiary)] opacity-0 transition-all group-hover:opacity-100 hover:bg-red-500/10 hover:text-red-400 disabled:pointer-events-none disabled:opacity-0'>
				<X className='h-3.5 w-3.5' />
			</button>
		</div>
	)
}
