import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { FileText, Plus, Pencil, Trash2 } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { useAnimationsEnabled } from '@/hooks/useMotion'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useAccountStore } from '@/stores/accountStore'
import { useThemeStore } from '@/stores/themeStore'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { SettingCard } from '@/components/ui/custom/SettingCard'
import { SignatureEditor } from './Signatures/SignatureEditor'
import type { Template } from '@/types/templates'

interface TemplateFormData {
	name: string
	subject: string
	htmlBody: string
}

const EMPTY_FORM: TemplateFormData = {
	name: '',
	subject: '',
	htmlBody: '',
}

function TemplateCard({
	template,
	accentColor,
	animationsEnabled,
	onEdit,
	onDelete,
}: {
	template: Template
	accentColor: string
	animationsEnabled: boolean
	onEdit: () => void
	onDelete: () => void
}) {
	return (
		<motion.div
			layout={animationsEnabled}
			className='flex items-center gap-3 rounded-xl border border-[var(--border-faint)] bg-[var(--surface-panel)] px-4 py-3'>
			<div
				className='flex h-8 w-8 shrink-0 items-center justify-center rounded-lg'
				style={{
					backgroundColor: `rgba(var(--accent-rgb), 0.08)`,
				}}>
				<FileText className='h-4 w-4' style={{ color: accentColor }} />
			</div>

			<div className='min-w-0 flex-1'>
				<div className='flex flex-col'>
					<span className='text-sm font-medium text-[var(--text-primary)]'>
						{template.name}
					</span>
					{template.subject && (
						<span className='truncate text-xs text-[var(--text-tertiary)]'>
							{template.subject}
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

function TemplateForm({
	initialData,
	onSave,
	onCancel,
}: {
	initialData: TemplateFormData & { id?: string }
	onSave: (data: TemplateFormData & { id?: string }) => void
	onCancel: () => void
}) {
	const { t } = useSettingsTranslation()
	const [form, setForm] = useState<TemplateFormData>({
		name: initialData.name,
		subject: initialData.subject,
		htmlBody: initialData.htmlBody,
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
				<div className='grid grid-cols-2 gap-4'>
					<div>
						<label className='mb-1.5 block text-xs font-semibold tracking-wider text-[var(--text-secondary)] uppercase'>
							{t('settings:templates.name')}
						</label>
						<input
							autoFocus
							value={form.name}
							onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
							placeholder='Support Reply'
							className='w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-hover)] px-3 py-2 text-sm text-[var(--text-primary)] transition-colors outline-none placeholder:text-[var(--text-tertiary)] focus:border-[var(--accent-color)] focus:ring-1 focus:ring-[var(--accent-color)]'
						/>
					</div>
					<div>
						<label className='mb-1.5 block text-xs font-semibold tracking-wider text-[var(--text-secondary)] uppercase'>
							{t('settings:templates.subject')}
						</label>
						<input
							value={form.subject}
							onChange={(e) => setForm((f) => ({ ...f, subject: e.target.value }))}
							placeholder='Re: {{subject}}'
							className='w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-hover)] px-3 py-2 text-sm text-[var(--text-primary)] transition-colors outline-none placeholder:text-[var(--text-tertiary)] focus:border-[var(--accent-color)] focus:ring-1 focus:ring-[var(--accent-color)]'
						/>
					</div>
				</div>

				<div>
					<label className='mb-1.5 block text-xs font-semibold tracking-wider text-[var(--text-secondary)] uppercase'>
						{t('settings:templates.body')}
					</label>
					<SignatureEditor
						initialHtml={form.htmlBody}
						placeholder='Type your template content...'
						onChange={(html) => setForm((f) => ({ ...f, htmlBody: html }))}
					/>
					<p className='mt-2 text-[10px] text-[var(--text-tertiary)]'>
						Available variables: <code className='rounded bg-[var(--surface-hover)] px-1'>{'{{name}}'}</code>, <code className='rounded bg-[var(--surface-hover)] px-1'>{'{{email}}'}</code>, <code className='rounded bg-[var(--surface-hover)] px-1'>{'{{date}}'}</code>
					</p>
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

export function TemplatesSettings() {
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const accountId = activeAccount?.id ?? ''
	const { t } = useSettingsTranslation()
	const accentColor = useThemeStore((s) => s.accentColor)
	const animationsEnabled = useAnimationsEnabled()
	const qc = useQueryClient()

	const [isAdding, setIsAdding] = useState(false)
	const [editingId, setEditingId] = useState<string | null>(null)
	const [deleteTarget, setDeleteTarget] = useState<Template | null>(null)

	const { data: templates = [], isLoading } = useQuery<Template[]>({
		queryKey: ['templates', accountId],
		queryFn: () => invoke<Template[]>('list_templates', { accountId }),
		enabled: !!accountId,
	})

	const saveMutation = useMutation({
		mutationFn: (tmpl: Template) => invoke('save_template', { tmpl }),
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: ['templates', accountId] })
		},
	})

	const deleteMutation = useMutation({
		mutationFn: ({ id, accountId: aid }: { id: string; accountId: string }) =>
			invoke('delete_template', { id, accountId: aid }),
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: ['templates', accountId] })
		},
	})

	const handleSave = (data: TemplateFormData & { id?: string }) => {
		const now = Math.floor(Date.now() / 1000)
		const tmpl: Template = {
			id: data.id ?? '',
			accountId,
			name: data.name.trim(),
			subject: data.subject,
			htmlBody: data.htmlBody,
			createdAt: now,
			updatedAt: now,
		}
		saveMutation.mutate(tmpl, {
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
				{t('status:loading')}
			</div>
		)
	}

	return (
		<div className='space-y-6 p-6'>
			<div className='flex items-start justify-between'>
				<div>
					<h2 className='text-sm font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
						{t('settings:templates.title')}
					</h2>
					<p className='mt-1 text-xs text-[var(--text-tertiary)]'>
						{t('settings:templates.subtitle')}
					</p>
				</div>

				{!isAdding && !editingId && (
					<button
						type='button'
						onClick={() => setIsAdding(true)}
						className='flex items-center gap-1.5 rounded-xl bg-[var(--accent-primary)] px-3.5 py-2 text-sm font-semibold text-white shadow-sm transition-all hover:opacity-90 active:scale-[0.97]'
						style={{ backgroundColor: accentColor }}>
						<Plus className='h-4 w-4' />
						{t('settings:templates.addTemplate')}
					</button>
				)}
			</div>

			<AnimatePresence mode='popLayout'>
				{isAdding && (
					<TemplateForm
						initialData={EMPTY_FORM}
						onSave={handleSave}
						onCancel={() => setIsAdding(false)}
					/>
				)}

				{templates.length === 0 && !isAdding ? (
					<SettingCard
						icon={FileText}
						label={t('settings:templates.noTemplates')}
						description={t('settings:templates.noTemplatesDesc')}>
						<button
							type='button'
							onClick={() => setIsAdding(true)}
							className='text-sm font-medium hover:underline'
							style={{ color: accentColor }}>
							{t('settings:templates.addTemplate')}
						</button>
					</SettingCard>
				) : (
					<div className='grid gap-3'>
						{templates.map((tmpl) =>
							editingId === tmpl.id ? (
								<TemplateForm
									key={tmpl.id}
									initialData={{
										id: tmpl.id,
										name: tmpl.name,
										subject: tmpl.subject,
										htmlBody: tmpl.htmlBody,
									}}
									onSave={handleSave}
									onCancel={() => setEditingId(null)}
								/>
							) : (
								<TemplateCard
									key={tmpl.id}
									template={tmpl}
									accentColor={accentColor}
									animationsEnabled={animationsEnabled}
									onEdit={() => setEditingId(tmpl.id)}
									onDelete={() => setDeleteTarget(tmpl)}
								/>
							)
						)}
					</div>
				)}
			</AnimatePresence>

			<ConfirmationDialog
				open={!!deleteTarget}
				onOpenChange={(open) => !open && setDeleteTarget(null)}
				title={t('settings:templates.deleteConfirm.title', {
					name: deleteTarget?.name,
				})}
				description={t('settings:templates.deleteConfirm.description')}
				confirmLabel={t('settings:templates.deleteConfirm.confirm')}
				cancelLabel={t('settings:templates.deleteConfirm.cancel')}
				onConfirm={handleDelete}
				confirmClassName='bg-red-500 text-white hover:bg-red-600'
			/>
		</div>
	)
}
