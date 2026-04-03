import { ChevronDown, X } from 'lucide-react'
import { RuleCondition, ConditionField, ConditionOperator } from '@/types/filters'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'

interface ConditionRowProps {
	condition: RuleCondition
	onUpdate: (updates: Partial<RuleCondition>) => void
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
				className='h-9 w-full cursor-pointer appearance-none rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] pr-8 pl-3 text-xs font-medium text-[var(--text-primary)] transition-colors hover:border-[var(--border-subtle)] hover:bg-[var(--surface-hover)] focus:border-[var(--accent-primary)] focus:ring-1 focus:ring-[var(--accent-primary)]/20 focus:outline-none'>
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

export function ConditionRow({ condition, onUpdate, onRemove, isOnly }: ConditionRowProps) {
	const { t } = useTypedTranslation(['settings'])

	const fieldOptions: { value: ConditionField; label: string }[] = [
		{ value: 'from', label: t('settings:filters.fields.from') },
		{ value: 'to', label: t('settings:filters.fields.to') },
		{ value: 'subject', label: t('settings:filters.fields.subject') },
		{ value: 'body', label: t('settings:filters.fields.body') },
	]

	const operatorOptions: { value: ConditionOperator; label: string }[] = [
		{ value: 'contains', label: t('settings:filters.operators.contains') },
		{ value: 'not_contains', label: t('settings:filters.operators.not_contains') },
		{ value: 'equals', label: t('settings:filters.operators.equals') },
		{ value: 'not_equals', label: t('settings:filters.operators.not_equals') },
		{ value: 'starts_with', label: t('settings:filters.operators.starts_with') },
		{ value: 'ends_with', label: t('settings:filters.operators.ends_with') },
	]

	return (
		<div className='group flex items-center gap-2'>
			<StyledSelect
				value={condition.field}
				onChange={(v) => onUpdate({ field: v })}
				options={fieldOptions}
				className='w-28 shrink-0'
			/>

			<StyledSelect
				value={condition.operator}
				onChange={(v) => onUpdate({ operator: v })}
				options={operatorOptions}
				className='w-40 shrink-0'
			/>

			<input
				type='text'
				value={condition.value}
				onChange={(e) => onUpdate({ value: e.target.value })}
				placeholder={t('settings:filters.valuePlaceholder')}
				className='h-9 min-w-0 flex-1 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] px-3 text-xs font-medium text-[var(--text-primary)] transition-colors placeholder:text-[var(--text-tertiary)] focus:border-[var(--accent-primary)] focus:ring-1 focus:ring-[var(--accent-primary)]/20 focus:outline-none'
			/>

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
