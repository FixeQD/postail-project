import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { Tag, Pencil, Trash2, Check, X } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useAccountStore } from '@/stores/accountStore'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { SettingCard } from '@/components/ui/custom/SettingCard'

// ── Hue slider ────────────────────────────────────────────────────────────────
const HueSlider = ({ hue, onChange }: { hue: number; onChange: (h: number) => void }) => (
	<div className='flex items-center gap-2'>
		<div
			className='h-4 w-4 shrink-0 rounded-full ring-1 ring-black/20'
			style={{ background: `hsl(${hue} 70% 55%)` }}
		/>
		<input
			type='range'
			min={0}
			max={359}
			value={hue}
			onChange={(e) => onChange(Number(e.target.value))}
			className='h-1.5 w-28 cursor-pointer appearance-none rounded-full outline-none'
			style={{
				background: `linear-gradient(to right, hsl(0 70% 55%), hsl(60 70% 55%), hsl(120 70% 55%), hsl(180 70% 55%), hsl(240 70% 55%), hsl(300 70% 55%), hsl(360 70% 55%))`,
			}}
		/>
	</div>
)

// ── TagRow ────────────────────────────────────────────────────────────────────
const TagRow = ({
	tag,
	hue,
	accountId,
	onDeleted,
}: {
	tag: string
	hue: number
	accountId: string
	onDeleted: () => void
}) => {
	const qc = useQueryClient()
	const animationsEnabled = useAnimationsEnabled()
	const { t } = useTypedTranslation(['common', 'settings'])
	const [editing, setEditing] = useState(false)
	const [draft, setDraft] = useState(tag)
	const [localHue, setLocalHue] = useState(hue)
	const [confirmDelete, setConfirmDelete] = useState(false)

	const invalidate = () => {
		qc.invalidateQueries({ queryKey: ['tag-colors', accountId] })
		qc.invalidateQueries({ queryKey: ['account-tags', accountId] })
	}

	const saveColor = useMutation({
		mutationFn: (h: number) => invoke('set_tag_color', { tag, hue: h }),
		onSuccess: invalidate,
	})

	const renameTag = useMutation({
		mutationFn: (newTag: string) => invoke('rename_tag', { oldTag: tag, newTag, accountId }),
		onSuccess: () => {
			invalidate()
			setEditing(false)
		},
	})

	const deleteTag = useMutation({
		mutationFn: () => invoke('delete_tag', { tag, accountId }),
		onSuccess: () => {
			invalidate()
			onDeleted()
		},
	})

	const commitRename = () => {
		let t_str = draft.trim()
		if (!t_str || t_str === tag) {
			setEditing(false)
			setDraft(tag)
			return
		}

		if (t_str.includes(' ')) {
			t_str = t_str.replace(/ /g, '_')
			const { toast } = require('@/stores/toastStore')
			toast.info(t('settings:tags.formatToast'), {
				description: t('settings:tags.formatDesc'),
			})
		}

		renameTag.mutate(t_str)
	}

	const commitColor = () => saveColor.mutate(localHue)

	return (
		<>
			<motion.div
				layout={animationsEnabled}
				className='flex items-center gap-3 rounded-xl border border-[var(--border-faint)] bg-[var(--surface-panel)] px-4 py-3'>
				{/* Colour swatch */}
				<div
					className='flex h-6 w-6 shrink-0 items-center justify-center rounded-full'
					style={{
						background: `hsl(${localHue} 65% 20%)`,
						color: `hsl(${localHue} 80% 70%)`,
					}}>
					<Tag className='h-3 w-3' />
				</div>

				{/* Name — editable */}
				<div className='min-w-0 flex-1'>
					{editing ? (
						<div className='flex items-center gap-2'>
							<input
								autoFocus
								value={draft}
								maxLength={40}
								onChange={(e) => setDraft(e.target.value)}
								onKeyDown={(e) => {
									if (e.key === 'Enter') commitRename()
									if (e.key === 'Escape') {
										setEditing(false)
										setDraft(tag)
									}
								}}
								className='min-w-0 flex-1 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-active)] px-2 py-0.5 text-sm text-[var(--text-primary)] focus:outline-none'
							/>
							<button
								type='button'
								onClick={commitRename}
								className='text-status-success '>
								<Check className='h-4 w-4' />
							</button>
							<button
								type='button'
								onClick={() => {
									setEditing(false)
									setDraft(tag)
								}}
								className='text-[var(--text-tertiary)] hover:text-[var(--text-primary)]'>
								<X className='h-4 w-4' />
							</button>
						</div>
					) : (
						<span
							className='text-sm font-medium'
							style={{ color: `hsl(${localHue} 75% 70%)` }}>
							{tag}
						</span>
					)}
				</div>

				{/* Hue slider */}
				{!editing && (
					<HueSlider
						hue={localHue}
						onChange={(h) => {
							setLocalHue(h)
						}}
					/>
				)}

				{/* Actions */}
				{!editing && (
					<div className='flex items-center gap-1'>
						<button
							type='button'
							onClick={commitColor}
							disabled={localHue === hue || saveColor.isPending}
							className='rounded-lg px-2 py-1 text-xs font-medium text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)] disabled:opacity-30'>
							{t('common:actions.save')}
						</button>
						<button
							type='button'
							onClick={() => setEditing(true)}
							className='flex h-7 w-7 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
							<Pencil className='h-3.5 w-3.5' />
						</button>
						<button
							type='button'
							onClick={() => setConfirmDelete(true)}
							className='flex h-7 w-7 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-colors hover:bg-destructive/15 hover:text-destructive'>
							<Trash2 className='h-3.5 w-3.5' />
						</button>
					</div>
				)}
			</motion.div>

			<ConfirmationDialog
				open={confirmDelete}
				onOpenChange={setConfirmDelete}
				title={t('settings:tags.deleteConfirm.title', { tag })}
				description={t('settings:tags.deleteConfirm.description')}
				confirmLabel={t('settings:tags.deleteConfirm.confirm')}
				cancelLabel={t('settings:tags.deleteConfirm.cancel')}
				onConfirm={() => deleteTag.mutate()}
				confirmClassName='bg-destructive text-white hover:bg-destructive'
			/>
		</>
	)
}

// ── TagsSettings ──────────────────────────────────────────────────────────────
export const TagsSettings = () => {
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const accountId = activeAccount?.id ?? ''

	const { data: tagColors = {}, isLoading } = useQuery<Record<string, number>>({
		queryKey: ['tag-colors', accountId],
		queryFn: () => invoke('get_tag_colors', { accountId }),
		enabled: !!accountId,
	})

	const tags = Object.keys(tagColors).sort()
	const { t } = useTypedTranslation(['common', 'settings'])

	return (
		<div className='space-y-6 p-6'>
			<div>
				<h2 className='text-sm font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
					{t('settings:tags.title')}
				</h2>
				<p className='mt-1 text-xs text-[var(--text-tertiary)]'>
					{t('settings:tags.subtitle')}
				</p>
			</div>

			{isLoading && <p className='text-sm text-[var(--text-tertiary)]'>{t('settings:tags.loading')}</p>}

			{!isLoading && tags.length === 0 && (
				<SettingCard
					icon={Tag}
					label={t('settings:tags.noTags')}
					description={t('settings:tags.noTagsDesc')}>
					{null}
				</SettingCard>
			)}

			<AnimatePresence mode='popLayout'>
				{tags.map((tag) => (
					<TagRow
						key={tag}
						tag={tag}
						hue={tagColors[tag] ?? 200}
						accountId={accountId}
						onDeleted={() => {}}
					/>
				))}
			</AnimatePresence>
		</div>
	)
}
