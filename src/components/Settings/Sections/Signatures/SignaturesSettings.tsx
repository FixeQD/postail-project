import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { PenLine, Plus, Pencil, Trash2, Check, Star } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useAccountStore } from '@/stores/accountStore'
import { useThemeStore } from '@/stores/themeStore'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { SettingCard } from '@/components/ui/custom/SettingCard'
import { SignatureEditor } from './SignatureEditor'
import type { Signature } from '@/types/signatures'

interface SignatureFormData {
	name: string
	htmlContent: string
	isDefault: boolean
}

const EMPTY_FORM: SignatureFormData = {
	name: '',
	htmlContent: '',
	isDefault: false,
}

function SignatureCard({
	signature,
	accentColor,
	animationsEnabled,
	onEdit,
	onDelete,
}: {
	signature: Signature
	accentColor: string
	animationsEnabled: boolean
	onEdit: () => void
	onDelete: () => void
}) {
	const { t } = useSettingsTranslation()

	return (
		<motion.div
			layout={animationsEnabled}
			className='flex items-center gap-3 rounded-xl border border-[var(--border-faint)] bg-[var(--surface-panel)] px-4 py-3'>
			<div
				className='flex h-8 w-8 shrink-0 items-center justify-center rounded-lg'
				style={{
					backgroundColor: `rgba(var(--accent-rgb), 0.08)`,
				}}>
				<PenLine className='h-4 w-4' style={{ color: accentColor }} />
			</div>

			<div className='min-w-0 flex-1'>
				<div className='flex items-center gap-2'>
					<span className='text-sm font-medium text-[var(--text-primary)]'>
						{signature.name}
					</span>
					{signature.isDefault && (
						<span
							className='inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-semibold tracking-wider uppercase'
							style={{
								backgroundColor: `rgba(var(--accent-rgb), 0.1)`,
								color: accentColor,
							}}>
							<Star className='h-2.5 w-2.5' />
							{t('settings:signatures.defaultBadge')}
						</span>
					)}
				</div>
			</div>

			<div className='flex items-center gap-1'>
				<button
					type='button'
					onClick={onEdit}
					className='flex h-7 w-7 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
					<Pencil className='h-3.5 w-3.5' />
				</button>
				<button
					type='button'
					onClick={onDelete}
					className='flex h-7 w-7 items-center justify-center rounded-lg text-[var(--text-tertiary)] transition-colors hover:bg-red-500/10 hover:text-red-400'>
					<Trash2 className='h-3.5 w-3.5' />
				</button>
			</div>
		</motion.div>
	)
}

function SignatureForm({
	initialData,
	onSave,
	onCancel,
}: {
	initialData: SignatureFormData & { id?: string }
	onSave: (data: SignatureFormData & { id?: string }) => void
	onCancel: () => void
}) {
	const { t } = useSettingsTranslation()
	const [form, setForm] = useState<SignatureFormData>({
		name: initialData.name,
		htmlContent: initialData.htmlContent,
		isDefault: initialData.isDefault,
	})
	const accentColor = useThemeStore((s) => s.accentColor)

	const handleSubmit = () => {
		if (!form.name.trim()) return
		onSave({ ...form, id: initialData.id })
	}

	return (
		<motion.div
			initial={{ opacity: 0, height: 0 }}
			animate={{ opacity: 1, height: 'auto' }}
			exit={{ opacity: 0, height: 0 }}
			transition={{ duration: 0.22, ease: [0.16, 1, 0.3, 1] }}
			style={{ overflow: 'hidden' }}
			className='rounded-xl border border-[var(--border-faint)] bg-[var(--surface-panel)] p-4'>
			<div className='space-y-4'>
				<div>
					<label className='mb-1.5 block text-xs font-semibold tracking-wider text-[var(--text-secondary)] uppercase'>
						{t('settings:signatures.name')}
					</label>
					<input
						autoFocus
						value={form.name}
						onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
						placeholder='My Signature'
						className='w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-hover)] px-3 py-2 text-sm text-[var(--text-primary)] transition-colors outline-none placeholder:text-[var(--text-tertiary)] focus:border-[var(--accent-color)] focus:ring-1 focus:ring-[var(--accent-color)]'
					/>
				</div>

				<div>
					<label className='mb-1.5 block text-xs font-semibold tracking-wider text-[var(--text-secondary)] uppercase'>
						{t('settings:signatures.title')}
					</label>
					<SignatureEditor
						initialHtml={form.htmlContent}
						placeholder={t('settings:signatures.placeholder')}
						onChange={(html) => setForm((f) => ({ ...f, htmlContent: html }))}
					/>
				</div>

				<div className='flex items-center gap-2'>
					<button
						type='button'
						role='checkbox'
						aria-checked={form.isDefault}
						onClick={() => setForm((f) => ({ ...f, isDefault: !f.isDefault }))}
						className='flex h-4 w-4 shrink-0 items-center justify-center rounded border transition-colors'
						style={{
							borderColor: form.isDefault ? accentColor : 'var(--border-subtle)',
							backgroundColor: form.isDefault ? accentColor : 'transparent',
						}}>
						{form.isDefault && <Check className='h-3 w-3 text-white' />}
					</button>
					<span className='text-sm text-[var(--text-primary)]'>
						{t('settings:signatures.isDefault')}
					</span>
				</div>

				<div className='flex items-center justify-end gap-2 border-t border-[var(--border-faint)] pt-4'>
					<button
						type='button'
						onClick={onCancel}
						className='rounded-xl px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
						{t('common:actions.cancel')}
					</button>
					<button
						type='button'
						onClick={handleSubmit}
						disabled={!form.name.trim()}
						className='rounded-xl px-4 py-2 text-sm font-semibold text-white shadow-sm transition-all hover:opacity-90 active:scale-[0.97] disabled:opacity-40'
						style={{ backgroundColor: accentColor }}>
						{t('common:actions.save')}
					</button>
				</div>
			</div>
		</motion.div>
	)
}

export function SignaturesSettings() {
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const accountId = activeAccount?.id ?? ''
	const { t } = useSettingsTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const qc = useQueryClient()

	const [isAdding, setIsAdding] = useState(false)
	const [editingId, setEditingId] = useState<string | null>(null)
	const [deleteTarget, setDeleteTarget] = useState<Signature | null>(null)

	const { data: signatures = [], isLoading } = useQuery<Signature[]>({
		queryKey: ['signatures', accountId],
		queryFn: () => invoke<Signature[]>('list_signatures', { accountId }),
		enabled: !!accountId,
	})

	const saveMutation = useMutation({
		mutationFn: (sig: Signature) => invoke('save_signature', { sig }),
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: ['signatures', accountId] })
		},
	})

	const deleteMutation = useMutation({
		mutationFn: ({ id, accountId: aid }: { id: string; accountId: string }) =>
			invoke('delete_signature', { id, accountId: aid }),
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: ['signatures', accountId] })
		},
	})

	const handleSave = (data: SignatureFormData & { id?: string }) => {
		const now = Math.floor(Date.now() / 1000)
		const sig: Signature = {
			id: data.id ?? '',
			accountId,
			name: data.name.trim(),
			htmlContent: data.htmlContent,
			isDefault: data.isDefault,
			createdAt: now,
			updatedAt: now,
		}
		saveMutation.mutate(sig, {
			onSuccess: () => {
				setIsAdding(false)
				setEditingId(null)
			},
		})
	}

	const handleDelete = () => {
		if (!deleteTarget) return
		deleteMutation.mutate(
			{ id: deleteTarget.id, accountId },
			{ onSuccess: () => setDeleteTarget(null) }
		)
	}

	if (isLoading) {
		return (
			<div className='flex items-center gap-2 p-6 text-sm text-[var(--text-tertiary)]'>
				<div className='h-4 w-4 animate-spin rounded-full border-2 border-[var(--border-subtle)] border-t-[var(--accent-primary)]' />
				{t('settings:tags.loading')}
			</div>
		)
	}

	return (
		<div className='space-y-6 p-6'>
			<div className='flex items-start justify-between'>
				<div>
					<h2 className='text-sm font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
						{t('settings:signatures.title')}
					</h2>
					<p className='mt-1 text-xs text-[var(--text-tertiary)]'>
						{t('settings:signatures.addSignature')}
					</p>
				</div>

				{!isAdding && !editingId && (
					<button
						type='button'
						onClick={() => setIsAdding(true)}
						className='flex items-center gap-1.5 rounded-xl bg-[var(--accent-primary)] px-3.5 py-2 text-sm font-semibold text-white shadow-sm transition-all hover:opacity-90 active:scale-[0.97]'
						style={{ backgroundColor: accentColor }}>
						<Plus className='h-4 w-4' />
						{t('settings:signatures.addSignature')}
					</button>
				)}
			</div>

			<AnimatePresence mode='popLayout'>
				{isAdding && (
					<SignatureForm
						initialData={EMPTY_FORM}
						onSave={handleSave}
						onCancel={() => setIsAdding(false)}
					/>
				)}

				{signatures.length === 0 && !isAdding ? (
					<SettingCard
						icon={PenLine}
						label={t('settings:signatures.noSignatures')}
						description={t('settings:signatures.addSignature')}>
						<button
							type='button'
							onClick={() => setIsAdding(true)}
							className='text-sm font-medium hover:underline'
							style={{ color: accentColor }}>
							{t('settings:signatures.addSignature')}
						</button>
					</SettingCard>
				) : (
					signatures.map((sig) =>
						editingId === sig.id ? (
							<SignatureForm
								key={sig.id}
								initialData={{
									id: sig.id,
									name: sig.name,
									htmlContent: sig.htmlContent,
									isDefault: sig.isDefault,
								}}
								onSave={handleSave}
								onCancel={() => setEditingId(null)}
							/>
						) : (
							<SignatureCard
								key={sig.id}
								signature={sig}
								accentColor={accentColor}
								animationsEnabled={animationsEnabled}
								onEdit={() => setEditingId(sig.id)}
								onDelete={() => setDeleteTarget(sig)}
							/>
						)
					)
				)}
			</AnimatePresence>

			<ConfirmationDialog
				open={!!deleteTarget}
				onOpenChange={(open) => !open && setDeleteTarget(null)}
				title={t('settings:signatures.deleteConfirm.title', {
					name: deleteTarget?.name,
				})}
				description={t('settings:signatures.deleteConfirm.description')}
				confirmLabel={t('settings:signatures.deleteConfirm.confirm')}
				cancelLabel={t('settings:signatures.deleteConfirm.cancel')}
				onConfirm={handleDelete}
				confirmClassName='bg-red-500 text-white hover:bg-red-600'
			/>
		</div>
	)
}
