import { useState } from 'react'
import {
	GripVertical,
	Power,
	Trash2,
	Pencil,
	ArrowRight,
	FolderInput,
	Tag,
	Star,
	MailCheck,
} from 'lucide-react'
import { Reorder } from 'framer-motion'
import { useShellTransition } from '@/hooks/useShellTransition'
import { FilterRule, ActionType } from '@/types/filters'
import { RuleEditor } from './Editor'

const ACTION_ICONS: Record<ActionType, React.ReactNode> = {
	move_to: <FolderInput className='h-3 w-3' />,
	add_tag: <Tag className='h-3 w-3' />,
	star: <Star className='h-3 w-3' />,
	mark_read: <MailCheck className='h-3 w-3' />,
	delete: <Trash2 className='h-3 w-3' />,
}

const ACTION_COLORS: Record<ActionType, string> = {
	move_to: 'bg-blue-500/10 text-blue-400',
	add_tag: 'bg-violet-500/10 text-violet-400',
	star: 'bg-yellow-500/10 text-yellow-400',
	mark_read: 'bg-green-500/10 text-green-400',
	delete: 'bg-red-500/10 text-red-400',
}

interface RuleItemProps {
	rule: FilterRule
	onDelete: () => void
	onToggle: () => void
	disabled: boolean
}

export function RuleItem({ rule, onDelete, onToggle, disabled }: RuleItemProps) {
	const [mode, setMode] = useState<'card' | 'editor'>('card')
	const { shellScope, contentScope, transition } = useShellTransition()

	const openEditor = () => {
		if (mode === 'editor') return
		transition(() => setMode('editor'))
	}

	const closeEditor = () => {
		transition(() => setMode('card'))
	}

	const isEditing = mode === 'editor'

	return (
		<Reorder.Item
			value={rule}
			dragListener={!isEditing}
			className={`overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] transition-colors ${
				!rule.enabled && !isEditing ? 'opacity-50' : ''
			} ${disabled && !isEditing ? 'pointer-events-none opacity-40' : ''}`}>
			<div ref={shellScope} className='w-full'>
				<div ref={contentScope}>
					{isEditing ? (
						<RuleEditor
							rule={rule}
							onSave={closeEditor}
							onCancel={closeEditor}
							inline
						/>
					) : (
						<div className='group flex items-center gap-3 px-4 py-3.5 transition-colors hover:bg-[var(--surface-hover)]'>
							{/* Drag handle */}
							<div className='shrink-0 cursor-grab text-[var(--text-tertiary)] opacity-0 transition-opacity group-hover:opacity-100 active:cursor-grabbing'>
								<GripVertical className='h-4 w-4' />
							</div>

							{/* Content */}
							<div className='min-w-0 flex-1'>
								<div className='flex items-center gap-2'>
									<span className='truncate text-sm font-semibold text-[var(--text-primary)]'>
										{rule.name}
									</span>
									{!rule.enabled && (
										<span className='shrink-0 rounded-full bg-[var(--surface-active)] px-2 py-0.5 text-[10px] font-bold tracking-wider text-[var(--text-tertiary)] uppercase'>
											Off
										</span>
									)}
								</div>

								<div className='mt-1.5 flex flex-wrap items-center gap-1.5'>
									<span className='text-[10px] font-medium text-[var(--text-tertiary)]'>
										{rule.conditions.length === 1
											? `${rule.conditions[0].field} ${rule.conditions[0].operator} "${rule.conditions[0].value}"`
											: `${rule.conditions.length} conditions`}
									</span>
									<ArrowRight className='h-3 w-3 text-[var(--text-tertiary)]' />
									{rule.actions.slice(0, 3).map((action, i) => (
										<span
											key={i}
											className={`flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-semibold ${ACTION_COLORS[action.action_type]}`}>
											{ACTION_ICONS[action.action_type]}
											{action.value
												? action.value
												: action.action_type.replace('_', ' ')}
										</span>
									))}
									{rule.actions.length > 3 && (
										<span className='text-[10px] text-[var(--text-tertiary)]'>
											+{rule.actions.length - 3}
										</span>
									)}
								</div>
							</div>

							{/* Actions */}
							<div className='flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100'>
								<button
									type='button'
									onClick={onToggle}
									title={rule.enabled ? 'Disable' : 'Enable'}
									className={`rounded-lg p-1.5 transition-colors ${
										rule.enabled
											? 'text-green-400 hover:bg-green-500/10'
											: 'text-[var(--text-tertiary)] hover:bg-[var(--surface-active)]'
									}`}>
									<Power className='h-3.5 w-3.5' />
								</button>
								<button
									type='button'
									onClick={openEditor}
									className='rounded-lg p-1.5 text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-active)] hover:text-[var(--text-primary)]'>
									<Pencil className='h-3.5 w-3.5' />
								</button>
								<button
									type='button'
									onClick={onDelete}
									className='rounded-lg p-1.5 text-[var(--text-tertiary)] transition-colors hover:bg-red-500/10 hover:text-red-400'>
									<Trash2 className='h-3.5 w-3.5' />
								</button>
							</div>
						</div>
					)}
				</div>
			</div>
		</Reorder.Item>
	)
}
