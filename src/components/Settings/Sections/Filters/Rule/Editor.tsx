import { useState } from 'react'
import { Plus, Save, AlertCircle } from 'lucide-react'
import { v4 as uuidv4 } from 'uuid'
import { useFilterRules } from '@/hooks/useFilterRules'
import { useAccountStore } from '@/stores/accountStore'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { FilterRule, RuleCondition, RuleAction, MatchMode } from '@/types/filters'
import { toast } from '@/components/ui/custom/Toaster'
import { ConditionRow } from '../ConditionRow'
import { ActionRow } from '../ActionRow'

interface RuleEditorProps {
	rule?: FilterRule | null
	onSave: () => void
	onCancel: () => void
	inline?: boolean
}

export function RuleEditor({ rule, onSave, onCancel, inline = false }: RuleEditorProps) {
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const accountId = activeAccount?.id ?? ''
	const { saveRule, isSaving, rules } = useFilterRules(accountId)
	const { t } = useTypedTranslation(['common', 'settings'])

	const [name, setName] = useState(rule?.name ?? '')
	const [matchMode, setMatchMode] = useState<MatchMode>(rule?.match_mode ?? 'all')
	const [conditions, setConditions] = useState<RuleCondition[]>(
		rule?.conditions ?? [{ field: 'from', operator: 'contains', value: '' }]
	)
	const [actions, setActions] = useState<RuleAction[]>(
		rule?.actions ?? [{ action_type: 'mark_read', value: '' }]
	)
	const [nameError, setNameError] = useState(false)

	const handleAddCondition = () =>
		setConditions([...conditions, { field: 'from', operator: 'contains', value: '' }])

	const handleRemoveCondition = (index: number) => {
		if (conditions.length > 1) setConditions(conditions.filter((_, i) => i !== index))
	}

	const handleUpdateCondition = (index: number, updates: Partial<RuleCondition>) => {
		const next = [...conditions]
		next[index] = { ...next[index], ...updates }
		setConditions(next)
	}

	const handleAddAction = () => setActions([...actions, { action_type: 'mark_read', value: '' }])

	const handleRemoveAction = (index: number) => {
		if (actions.length > 1) setActions(actions.filter((_, i) => i !== index))
	}

	const handleUpdateAction = (index: number, updates: Partial<RuleAction>) => {
		const next = [...actions]
		next[index] = { ...next[index], ...updates }
		setActions(next)
	}

	const handleSave = async () => {
		if (!name.trim()) {
			setNameError(true)
			return
		}

		const hasInvalidActions = actions.some(
			(a) => (a.action_type === 'move_to' || a.action_type === 'add_tag') && !a.value?.trim()
		)
		if (hasInvalidActions) {
			toast.error(t('settings:filters.editor.missingActionValues'))
			return
		}

		const newRule: FilterRule = {
			id: rule?.id ?? uuidv4(),
			account_id: accountId,
			name: name.trim(),
			match_mode: matchMode,
			conditions,
			actions,
			position:
				rule?.position ??
				(rules.length > 0 ? Math.max(...rules.map((r) => r.position)) + 1 : 0),
			enabled: rule?.enabled ?? true,
		}

		try {
			await saveRule(newRule)
			onSave()
		} catch (e) {
			console.error(e)
		}
	}

	return (
		<div
			className={
				inline
					? ''
					: 'mb-4 overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--surface-active)] shadow-xl ring-1 ring-[var(--accent-primary)]/10'
			}>
			{/* Name row */}
			<div className='border-b border-[var(--border-subtle)] px-5 py-4'>
				<div className='flex items-center gap-3'>
					<input
						autoFocus
						type='text'
						value={name}
						onChange={(e) => {
							setName(e.target.value)
							setNameError(false)
						}}
						placeholder={t('settings:filters.editor.nameHint')}
						className={`flex-1 rounded-lg border bg-[var(--surface-panel)] px-3 py-2 text-sm font-medium text-[var(--text-primary)] transition-colors placeholder:text-[var(--text-tertiary)] focus:ring-1 focus:outline-none ${
							nameError
								? 'border-destructive/30 focus:border-destructive/30 focus:ring-destructive/30'
								: 'border-[var(--border-subtle)] focus:border-[var(--accent-primary)] focus:ring-[var(--accent-primary)]/20'
						}`}
					/>
					{nameError && (
						<div className='flex shrink-0 items-center gap-1 text-xs text-destructive'>
							<AlertCircle className='h-3.5 w-3.5' />
							{t('settings:filters.editor.required')}
						</div>
					)}
				</div>
			</div>

			{/* IF section */}
			<div className='border-b border-[var(--border-subtle)]'>
				<div className='flex items-center justify-between px-5 py-3'>
					<div className='flex items-center gap-3'>
						{/* IF badge */}
						<span className='rounded-md bg-[var(--accent-primary)]/10 px-2 py-0.5 text-[11px] font-bold tracking-widest text-[var(--accent-primary)] uppercase'>
							{t('settings:filters.editor.conditions')}
						</span>

						{/* Match mode toggle */}
						<div className='flex items-center gap-1 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-0.5'>
							{(['all', 'any'] as MatchMode[]).map((mode) => (
								<button
									key={mode}
									type='button'
									onClick={() => setMatchMode(mode)}
									className={`rounded-md px-3 py-1 text-[11px] font-semibold transition-all ${
										matchMode === mode
											? 'bg-[var(--accent-primary)] text-white shadow-sm'
											: 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
									}`}>
									{t(`settings:filters.editor.${mode}`)}
								</button>
							))}
						</div>

						<span className='text-xs text-[var(--text-tertiary)]'>
							{matchMode === 'all'
								? t('settings:filters.editor.matchAnd')
								: t('settings:filters.editor.matchOr')}
						</span>
					</div>

					<button
						type='button'
						onClick={handleAddCondition}
						className='flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-[11px] font-semibold text-[var(--accent-primary)] transition-all hover:bg-[var(--accent-primary)]/8 active:scale-95'>
						<Plus className='h-3 w-3' />
						{t('settings:filters.editor.addCondition')}
					</button>
				</div>

				<div className='space-y-1.5 px-5 pb-4'>
					{conditions.map((c, i) => (
						<div key={i}>
							<ConditionRow
								condition={c}
								onUpdate={(u) => handleUpdateCondition(i, u)}
								onRemove={() => handleRemoveCondition(i)}
								isOnly={conditions.length === 1}
							/>
							{/* AND/OR connector between rows */}
							{i < conditions.length - 1 && (
								<div className='flex items-center gap-2 py-1 pl-1'>
									<div className='h-px flex-1 bg-[var(--border-subtle)]' />
									<span className='text-[10px] font-bold tracking-widest text-[var(--text-tertiary)] uppercase'>
										{matchMode === 'all'
											? t('settings:filters.editor.and', 'AND')
											: t('settings:filters.editor.or', 'OR')}
									</span>
									<div className='h-px flex-1 bg-[var(--border-subtle)]' />
								</div>
							)}
						</div>
					))}
				</div>
			</div>

			{/* THEN section */}
			<div className='border-b border-[var(--border-subtle)]'>
				<div className='flex items-center justify-between px-5 py-3'>
					<div className='flex items-center gap-3'>
						<span className='rounded-md bg-status-success/15 px-2 py-0.5 text-[11px] font-bold tracking-widest text-status-success uppercase'>
							{t('settings:filters.editor.actions')}
						</span>
						<span className='text-xs text-[var(--text-tertiary)]'>
							{t('settings:filters.editor.performActions')}
						</span>
					</div>

					<button
						type='button'
						onClick={handleAddAction}
						className='flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-[11px] font-semibold text-[var(--accent-primary)] transition-all hover:bg-[var(--accent-primary)]/8 active:scale-95'>
						<Plus className='h-3 w-3' />
						{t('settings:filters.editor.addAction')}
					</button>
				</div>

				<div className='space-y-1.5 px-5 pb-4'>
					{actions.map((a, i) => (
						<ActionRow
							key={i}
							action={a}
							accountId={accountId}
							onUpdate={(u) => handleUpdateAction(i, u)}
							onRemove={() => handleRemoveAction(i)}
							isOnly={actions.length === 1}
						/>
					))}
				</div>
			</div>

			{/* Footer */}
			<div className='flex items-center justify-end gap-2 px-5 py-3'>
				<button
					type='button'
					onClick={onCancel}
					className='rounded-lg px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
					{t('settings:filters.editor.cancel')}
				</button>
				<button
					type='button'
					onClick={handleSave}
					disabled={isSaving}
					className='flex items-center gap-2 rounded-lg bg-[var(--accent-primary)] px-5 py-2 text-sm font-semibold text-white shadow-sm transition-all hover:opacity-90 active:scale-[0.98] disabled:scale-100 disabled:opacity-50'>
					<Save className='h-3.5 w-3.5' />
					{t('settings:filters.editor.save')}
				</button>
			</div>
		</div>
	)
}
