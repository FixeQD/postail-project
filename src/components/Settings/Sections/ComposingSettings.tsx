import { motion } from 'framer-motion'
import { Mail, Save, SpellCheck } from 'lucide-react'
import { ToggleSetting } from '@/components/ui/toggle-setting'
import { useSettingsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'

export function ComposingSettings() {
	const { t } = useSettingsTranslation()
	const animationsEnabled = useAnimationsEnabled()

	return (
		<div className='mx-auto flex h-full w-full max-w-3xl flex-col space-y-8 p-8'>
			<motion.div
				{...(animationsEnabled
					? { initial: { opacity: 0, y: -20 }, animate: { opacity: 1, y: 0 } }
					: {})}>
				<h1 className='text-3xl font-bold tracking-tight text-[var(--text-primary)]'>
					{t('settings:composing.title')}
				</h1>
				<p className='mt-1 text-[var(--text-secondary)]'>
					{t('settings:composing.subtitle')}
				</p>
			</motion.div>

			<div className='space-y-6'>
				<section>
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
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
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
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
					<h2 className='mb-4 ml-2 text-xs font-bold tracking-widest text-[var(--text-secondary)] uppercase'>
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
