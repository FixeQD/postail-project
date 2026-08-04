import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
	Wand2,
	ArrowRight,
	FolderInput,
	Tag,
	Star,
	MailCheck,
	Trash2,
	X,
	Check,
} from 'lucide-react'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useQueryClient } from '@tanstack/react-query'
import { toast } from '@/stores/toastStore'
import type { FilterRule, ActionType } from '@/types/filters'

interface SuggestRuleDialogProps {
	rules: FilterRule[]
	accountId: string
	onClose: () => void
}

const ACTION_ICONS: Record<ActionType, React.ReactNode> = {
	move_to: <FolderInput className='h-3.5 w-3.5' />,
	add_tag: <Tag className='h-3.5 w-3.5' />,
	star: <Star className='h-3.5 w-3.5' />,
	mark_read: <MailCheck className='h-3.5 w-3.5' />,
	delete: <Trash2 className='h-3.5 w-3.5' />,
}

const ACTION_COLORS: Record<ActionType, string> = {
	move_to: 'bg-status-info/15 text-status-info border-status-info/30',
	add_tag: 'bg-status-info/15 text-status-info border-status-info/30',
	star: 'bg-status-warning/15 text-status-warning border-status-warning/30',
	mark_read: 'bg-status-success/15 text-status-success border-status-success/30',
	delete: 'bg-destructive/15 text-destructive border-destructive/30',
}

function RuleCard({ rule, onSaved }: { rule: FilterRule; onSaved: () => void }) {
	const { t } = useTypedTranslation(['common', 'settings'])
	const qc = useQueryClient()
	const [saving, setSaving] = useState(false)
	const [saved, setSaved] = useState(false)

	const handleSave = async () => {
		setSaving(true)
		try {
			await invoke('save_filter_rule', { rule })
			qc.invalidateQueries({ queryKey: ['filter-rules', rule.account_id] })
			setSaved(true)
			toast.success(t('settings:filters.suggest.saved'))
			setTimeout(onSaved, 800)
		} catch (e) {
			toast.error(String(e))
		} finally {
			setSaving(false)
		}
	}

	const firstCondition = rule.conditions[0]

	return (
		<div className='rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-4 transition-colors hover:bg-[var(--surface-hover)]'>
			<div className='flex items-start justify-between gap-3'>
				<div className='min-w-0 flex-1 space-y-2'>
					<p className='text-sm font-semibold text-[var(--text-primary)]'>{rule.name}</p>

					{/* Condition → Action summary */}
					<div className='flex flex-wrap items-center gap-1.5'>
						<span className='rounded-md bg-[var(--surface-active)] px-2 py-0.5 text-[11px] font-medium text-[var(--text-secondary)]'>
							{t(`settings:filters.fields.${firstCondition.field}`)}{' '}
							{t(`settings:filters.operators.${firstCondition.operator}`)} &ldquo;
							{firstCondition.value}&rdquo;
						</span>

						<ArrowRight className='h-3 w-3 shrink-0 text-[var(--text-tertiary)]' />

						{rule.actions.map((a, i) => (
							<span
								key={i}
								className={`flex items-center gap-1 rounded-md border px-2 py-0.5 text-[11px] font-semibold ${ACTION_COLORS[a.action_type]}`}>
								{ACTION_ICONS[a.action_type]}
								{a.value
									? a.value
									: t(`settings:filters.actionTypes.${a.action_type}`)}
							</span>
						))}
					</div>

					{/* Warning for move_to with no folder set */}
					{rule.actions.some((a) => a.action_type === 'move_to' && !a.value) && (
						<p className='text-[11px] text-status-warning'>
							{t('settings:filters.suggest.pickFolder')}
						</p>
					)}
				</div>

				<button
					type='button'
					onClick={handleSave}
					disabled={saving || saved}
					className={`flex shrink-0 items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold transition-all active:scale-95 disabled:opacity-60 ${
						saved
							? 'bg-status-success/15 text-status-success'
							: 'bg-[var(--accent-primary)] text-white hover:opacity-90'
					}`}>
					{saved ? (
						<>
							<Check className='h-3.5 w-3.5' />{' '}
							{t('settings:filters.suggest.savedShort')}
						</>
					) : (
						<>{t('settings:filters.suggest.save')}</>
					)}
				</button>
			</div>
		</div>
	)
}

export function SuggestRuleDialog({ rules, onClose }: SuggestRuleDialogProps) {
	const { t } = useTypedTranslation(['settings'])
	const [, setRemaining] = useState(rules.length)

	const handleSaved = () => {
		setRemaining((n) => {
			if (n - 1 <= 0) onClose()
			return n - 1
		})
	}

	return (
		<Dialog open onOpenChange={(open) => !open && onClose()}>
			<DialogContent className='max-w-lg border-[var(--border-subtle)] bg-[var(--surface-active)] shadow-2xl'>
				<DialogHeader>
					<div className='flex items-center gap-2'>
						<div className='flex h-8 w-8 items-center justify-center rounded-lg bg-[var(--accent-primary)]/10'>
							<Wand2 className='h-4 w-4 text-[var(--accent-primary)]' />
						</div>
						<DialogTitle className='text-sm font-bold'>
							{t('settings:filters.suggest.title')}
						</DialogTitle>
					</div>
					<p className='mt-1 text-xs text-[var(--text-tertiary)]'>
						{t('settings:filters.suggest.subtitle')}
					</p>
				</DialogHeader>

				<div className='mt-2 space-y-2'>
					{rules.map((rule, i) => (
						<RuleCard key={i} rule={rule} onSaved={handleSaved} />
					))}
				</div>

				<div className='mt-4 flex justify-end'>
					<button
						type='button'
						onClick={onClose}
						className='flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
						<X className='h-3.5 w-3.5' />
						{t('settings:filters.suggest.dismiss')}
					</button>
				</div>
			</DialogContent>
		</Dialog>
	)
}
