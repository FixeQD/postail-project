import { useState } from 'react'
import { Plus, Filter } from 'lucide-react'
import { motion, AnimatePresence, Reorder } from 'framer-motion'
import { useFilterRules } from '@/hooks/useFilterRules'
import { useAccountStore } from '@/stores/accountStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { SettingCard } from '@/components/ui/custom/SettingCard'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { RuleEditor } from './Filters/Rule/Editor'
import { RuleItem } from './Filters/Rule/Item'
import { FilterRule } from '@/types/filters'

export function FiltersSettings() {
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const accountId = activeAccount?.id ?? ''
	const { rules, isLoading, saveRule, deleteRule, reorderRules } = useFilterRules(accountId)
	const animationsEnabled = useAnimationsEnabled()
	const { t } = useTypedTranslation(['common', 'settings'])

	const [isAdding, setIsAdding] = useState(false)
	const [ruleToDelete, setRuleToDelete] = useState<FilterRule | null>(null)

	const handleReorder = (newOrder: FilterRule[]) => {
		reorderRules(newOrder.map((r) => r.id))
	}

	if (isLoading) {
		return (
			<div className='flex items-center gap-2 p-6 text-sm text-[var(--text-tertiary)]'>
				<div className='h-4 w-4 animate-spin rounded-full border-2 border-[var(--border-subtle)] border-t-[var(--accent-primary)]' />
				{t('settings:filters.loading')}
			</div>
		)
	}

	return (
		<div className='max-w-2xl space-y-6 p-6'>
			{/* Header */}
			<div className='flex items-start justify-between'>
				<div>
					<h2 className='text-sm font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
						{t('settings:filters.title')}
					</h2>
					<p className='mt-1 text-xs text-[var(--text-tertiary)]'>
						{t('settings:filters.subtitle')}
					</p>
				</div>

				{!isAdding && (
					<button
						type='button'
						onClick={() => setIsAdding(true)}
						className='flex items-center gap-1.5 rounded-xl bg-[var(--accent-primary)] px-3.5 py-2 text-sm font-semibold text-white shadow-sm transition-all hover:opacity-90 active:scale-[0.97]'>
						<Plus className='h-4 w-4' />
						{t('settings:filters.add')}
					</button>
				)}
			</div>

			<AnimatePresence>
				{isAdding && (
					<motion.div
						key='add-editor'
						initial={animationsEnabled ? { opacity: 0, height: 0 } : {}}
						animate={animationsEnabled ? { opacity: 1, height: 'auto' } : {}}
						exit={animationsEnabled ? { opacity: 0, height: 0 } : {}}
						transition={{ duration: 0.22, ease: [0.16, 1, 0.3, 1] }}
						style={{ overflow: 'hidden' }}>
						<RuleEditor
							rule={null}
							onSave={() => setIsAdding(false)}
							onCancel={() => setIsAdding(false)}
						/>
					</motion.div>
				)}
			</AnimatePresence>

			{/* Empty state */}
			{rules.length === 0 && !isAdding ? (
				<SettingCard
					icon={Filter}
					label={t('settings:filters.noRules')}
					description={t('settings:filters.noRulesDesc')}>
					<button
						type='button'
						onClick={() => setIsAdding(true)}
						className='text-sm font-medium text-[var(--accent-primary)] hover:underline'>
						{t('settings:filters.add')}
					</button>
				</SettingCard>
			) : (
				<Reorder.Group
					axis='y'
					values={rules}
					onReorder={handleReorder}
					className='space-y-2'>
					{rules.map((rule) => (
						<RuleItem
							key={rule.id}
							rule={rule}
							disabled={false}
							onDelete={() => setRuleToDelete(rule)}
							onToggle={() => saveRule({ ...rule, enabled: !rule.enabled })}
						/>
					))}
				</Reorder.Group>
			)}

			<ConfirmationDialog
				open={!!ruleToDelete}
				onOpenChange={(open) => !open && setRuleToDelete(null)}
				title={t('settings:filters.deleteConfirm.title', { name: ruleToDelete?.name })}
				description={t('settings:filters.deleteConfirm.description')}
				confirmLabel={t('settings:filters.deleteConfirm.confirm')}
				cancelLabel={t('settings:filters.deleteConfirm.cancel')}
				onConfirm={() => {
					if (ruleToDelete) {
						deleteRule(ruleToDelete.id)
						setRuleToDelete(null)
					}
				}}
				confirmClassName='bg-red-500 text-white hover:bg-red-600'
			/>
		</div>
	)
}
