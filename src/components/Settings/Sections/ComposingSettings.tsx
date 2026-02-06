import { motion } from 'framer-motion'
import { Mail, Save, SpellCheck } from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'

export function ComposingSettings() {
	const { t } = useSettingsTranslation()

	return (
		<div className='flex h-full flex-col p-8 max-w-3xl mx-auto w-full space-y-8'>
			<motion.div
				initial={{ opacity: 0, y: -20 }}
				animate={{ opacity: 1, y: 0 }}>
				<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
					{t('settings:composing.title')}
				</h1>
				<p className='mt-1 text-slate-400'>{t('settings:composing.subtitle')}</p>
			</motion.div>

			<div className='space-y-6'>
				<section>
					<h2 className='text-xs font-bold uppercase tracking-widest text-slate-500 mb-4 ml-2'>
						{t('settings:composing.sending.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Mail}
							label={t('settings:composing.sending.readReceipts.label')}
							description={t('settings:composing.sending.readReceipts.description')}
							value={false}
							onChange={() => {}}
						/>
					</div>
				</section>

				<section>
					<h2 className='text-xs font-bold uppercase tracking-widest text-slate-500 mb-4 ml-2'>
						{t('settings:composing.drafts.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={Save}
							label={t('settings:composing.drafts.autoSave.label')}
							description={t('settings:composing.drafts.autoSave.description')}
							value={true}
							onChange={() => {}}
						/>
					</div>
				</section>

				<section>
					<h2 className='text-xs font-bold uppercase tracking-widest text-slate-500 mb-4 ml-2'>
						{t('settings:composing.editor.title')}
					</h2>
					<div className='space-y-3'>
						<ToggleSetting
							icon={SpellCheck}
							label={t('settings:composing.editor.spellCheck.label')}
							description={t('settings:composing.editor.spellCheck.description')}
							value={true}
							onChange={() => {}}
						/>
					</div>
				</section>
			</div>
		</div>
	)
}
