import { useWelcomeTranslation } from '../../hooks/useTypedTranslation'
import icon from '../../assets/icon.png'

export const WelcomeScreen = ({ onGetStarted }: { onGetStarted: () => void }) => {
	const { t } = useWelcomeTranslation()

	return (
		<div className='flex h-full flex-col items-center justify-center p-8 text-center'>
			{/* Logo Section */}
			<div className='mb-8'>
				<div className='mb-6 flex justify-center'>
					<div className='flex h-20 w-20 items-center justify-center rounded-2xl bg-slate-800 shadow-lg ring-1 ring-slate-700/50'>
						<img src={icon} alt='Mail Icon' className='h-18 w-18' />
					</div>
				</div>
				<h1 className='mb-2 text-4xl font-bold tracking-tight text-slate-100'>
					{t('welcome:title')}
				</h1>
				<p className='text-slate-400'>{t('welcome:subtitle')}</p>
			</div>

			{/* Description */}
			<div className='mb-12 max-w-md'>
				<p className='leading-relaxed text-slate-400'>{t('welcome:description')}</p>
			</div>

			{/* Get Started Button */}
			<button
				type='button'
				onClick={onGetStarted}
				className='rounded-lg bg-indigo-600 px-8 py-3 font-semibold text-white shadow-lg transition-all duration-200 hover:bg-indigo-500 hover:shadow-indigo-500/40 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-slate-900 focus:outline-none'
				title={t('welcome:getStarted')}>
				{t('welcome:getStarted')}
			</button>
		</div>
	)
}
