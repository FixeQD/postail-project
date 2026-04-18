import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { FileText, Search, Settings, ChevronRight } from 'lucide-react'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { useAccountStore } from '@/stores/accountStore'
import type { Template } from '@/types/templates'

interface TemplateGalleryProps {
	onSelect: (template: Template) => void
	onManage: () => void
}

export function TemplateGallery({ onSelect, onManage }: TemplateGalleryProps) {
	const { t } = useTypedTranslation(['settings'])
	const activeAccount = useAccountStore((s) => s.activeAccount)
	const accountId = activeAccount?.id ?? ''
	const [search, setSearch] = useState('')

	const { data: templates = [], isLoading } = useQuery<Template[]>({
		queryKey: ['templates', accountId],
		queryFn: () => invoke<Template[]>('list_templates', { accountId }),
		enabled: !!accountId,
	})

	const filteredTemplates = templates.filter((tmpl) =>
		tmpl.name.toLowerCase().includes(search.toLowerCase()) ||
		tmpl.subject.toLowerCase().includes(search.toLowerCase())
	)

	// Helper to strip HTML for preview
	const stripHtml = (html: string) => {
		const doc = new DOMParser().parseFromString(html, 'text/html')
		return doc.body.textContent || ''
	}

	return (
		<div className='flex h-[400px] w-[320px] flex-col overflow-hidden'>
			<div className='p-3'>
				<div className='relative'>
					<Search className='absolute top-1/2 left-3 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-tertiary)]' />
					<input
						autoFocus
						value={search}
						onChange={(e) => setSearch(e.target.value)}
						placeholder={t('settings:templates.gallery.search')}
						className='w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-hover)] py-2 pr-3 pl-9 text-xs text-[var(--text-primary)] transition-colors outline-none focus:border-[var(--accent-color)]'
					/>
				</div>
			</div>

			<div className='flex-1 overflow-y-auto px-2 pb-2'>
				{isLoading ? (
					<div className='flex h-full items-center justify-center py-8'>
						<div className='h-4 w-4 animate-spin rounded-full border-2 border-[var(--border-subtle)] border-t-[var(--accent-primary)]' />
					</div>
				) : filteredTemplates.length === 0 ? (
					<div className='flex h-full flex-col items-center justify-center p-6 text-center'>
						<FileText className='mb-2 h-8 w-8 text-[var(--text-tertiary)] opacity-20' />
						<p className='text-xs font-medium text-[var(--text-secondary)]'>
							{t('settings:templates.gallery.empty')}
						</p>
					</div>
				) : (
					<div className='grid gap-1'>
						{filteredTemplates.map((tmpl) => (
							<button
								key={tmpl.id}
								onClick={() => onSelect(tmpl)}
								className='group relative flex flex-col items-start rounded-lg px-3 py-2 text-left transition-colors hover:bg-[var(--surface-hover)]'>
								<div className='flex w-full items-center justify-between gap-2'>
									<span className='truncate text-xs font-semibold text-[var(--text-primary)]'>
										{tmpl.name}
									</span>
									<ChevronRight className='h-3 w-3 translate-x-[-4px] opacity-0 transition-all group-hover:translate-x-0 group-hover:opacity-100' />
								</div>
								{tmpl.subject && (
									<span className='w-full truncate text-[10px] font-medium text-[var(--text-secondary)]'>
										{tmpl.subject}
									</span>
								)}
								<p className='mt-1 w-full truncate text-[10px] leading-relaxed text-[var(--text-tertiary)]'>
									{stripHtml(tmpl.htmlBody)}
								</p>
							</button>
						))}
					</div>
				)}
			</div>

			<div className='border-t border-[var(--border-faint)] p-2'>
				<button
					onClick={onManage}
					className='flex w-full items-center justify-center gap-2 rounded-lg py-2 text-[10px] font-bold tracking-widest text-[var(--text-tertiary)] uppercase transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
					<Settings className='h-3 w-3' />
					{t('settings:templates.gallery.manage')}
				</button>
			</div>
		</div>
	)
}
