import { useState } from 'react'
import { Plus, Filter, Play, Loader2, ChevronDown } from 'lucide-react'
import { motion, AnimatePresence, Reorder } from 'framer-motion'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { useFilterRules } from '@/hooks/useFilterRules'
import { useAccountStore } from '@/stores/accountStore'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { SettingCard } from '@/components/ui/custom/SettingCard'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { RuleEditor } from './Filters/Rule/Editor'
import { RuleItem } from './Filters/Rule/Item'
import { FilterRule } from '@/types/filters'
import type { Mailbox } from '@/types/mail'

export function FiltersSettings() {
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const accountId = activeAccount?.id ?? ''
	const {
		rules,
		isLoading,
		saveRule,
		deleteRule,
		reorderRules,
		applyRulesToMailbox,
		isApplying,
	} = useFilterRules(accountId)
	const animationsEnabled = useAnimationsEnabled()
	const { t } = useTypedTranslation(['common', 'settings'])

	const [isAdding, setIsAdding] = useState(false)
	const [ruleToDelete, setRuleToDelete] = useState<FilterRule | null>(null)

	const [applyMailbox, setApplyMailbox] = useState<string>('INBOX')

	const { data: mailboxes } = useQuery<Mailbox[]>({
		queryKey: ['mailboxes', accountId],
		queryFn: () => invoke<Mailbox[]>('fetch_mailboxes', { accountId }),
		enabled: !!accountId,
	})

	const handleApply = async () => {
		try {
			await applyRulesToMailbox(applyMailbox)
		} catch (e) {
			console.error(e)
		}
	}

	const hasEnabledRules = rules.some((r) => r.enabled)

	const handleReorder = (newOrder: FilterRule[]) => {
		reorderRules(newOrder.map((r) => r.id)).catch(console.error)
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
							onToggle={() =>
								saveRule({ ...rule, enabled: !rule.enabled }).catch(console.error)
							}
						/>
					))}
				</Reorder.Group>
			)}

			{rules.length > 0 && hasEnabledRules && (
				<div className='mt-8 flex flex-col items-start gap-4 border-t border-[var(--border-faint)] pt-6 xl:flex-row xl:items-center xl:justify-between'>
					<div className='flex-1'>
						<h3 className='text-sm font-semibold text-[var(--text-primary)]'>
							{t('settings:filters.applyNow')}
						</h3>
						<p className='mt-1 text-xs text-[var(--text-tertiary)]'>
							{t('settings:filters.applyNowDesc')}
						</p>
					</div>

					<div className='flex shrink-0 items-center gap-3'>
						<div className='relative min-w-[160px]'>
							<select
								value={applyMailbox}
								onChange={(e) => setApplyMailbox(e.target.value)}
								disabled={isApplying}
								className='h-9 w-full cursor-pointer appearance-none rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] pr-8 pl-3 text-sm font-medium text-[var(--text-primary)] transition-colors hover:bg-[var(--surface-hover)] focus:border-[var(--accent-primary)] focus:ring-1 focus:ring-[var(--accent-primary)]/20 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50'>
								{mailboxes?.map((mb) => (
									<option
										key={mb.name}
										value={mb.name}
										className='bg-[var(--surface-panel)] text-[var(--text-primary)]'>
										{mb.display_name || mb.name}
									</option>
								))}
							</select>
							<ChevronDown className='pointer-events-none absolute top-1/2 right-2.5 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-tertiary)]' />
						</div>

						<button
							type='button'
							onClick={handleApply}
							disabled={isApplying}
							className='flex h-9 items-center gap-2 rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] px-4 text-sm font-medium whitespace-nowrap text-[var(--text-primary)] shadow-sm transition-all hover:bg-[var(--surface-hover)] active:scale-[0.97] disabled:pointer-events-none disabled:opacity-50'>
							{isApplying ? (
								<>
									<Loader2 className='h-3.5 w-3.5 animate-spin' />
									{t('settings:filters.applying')}
								</>
							) : (
								<>
									<Play className='h-3.5 w-3.5 fill-current' />
									{t('settings:filters.applyNow')}
								</>
							)}
						</button>
					</div>
				</div>
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
						deleteRule(ruleToDelete.id).catch(console.error)
						setRuleToDelete(null)
					}
				}}
				confirmClassName='bg-destructive text-white hover:bg-destructive'
			/>
		</div>
	)
}
