import { motion } from 'framer-motion'
import { Plus, Mail } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useAccountsTranslation } from '@/hooks/useTypedTranslation'
import { useAnimationsEnabled } from '@/hooks/useMotion'

import { AccountCard } from './AccountCard'
import { AddAccountDialog } from './AddAccountDialog'
import type { AccountsScreenProps } from '@/types/components/settings'

export function AccountsScreen({
	accounts,
	onRemoveAccount,
	onSyncAccount,
	onAccountAdded,
}: AccountsScreenProps) {
	const { t } = useAccountsTranslation()
	const animationsEnabled = useAnimationsEnabled()

	return (
		<div className='flex h-full flex-col overflow-y-auto p-8'>
			<motion.div
				{...(animationsEnabled
					? {
							initial: { opacity: 0, y: -16 },
							animate: { opacity: 1, y: 0 },
							transition: { duration: 0.4, ease: [0.16, 1, 0.3, 1] },
						}
					: {})}
				className='mb-8 flex items-center justify-between'>
				<div>
					<h1 className='text-3xl font-bold tracking-tight text-slate-100'>
						{t('settings:accounts.title')}
					</h1>
					<p className='mt-1 text-sm text-slate-400'>{t('settings:accounts.subtitle')}</p>
				</div>
				<AddAccountDialog>
					<motion.div
						{...(animationsEnabled
							? { whileHover: { scale: 1.03 }, whileTap: { scale: 0.97 } }
							: {})}>
						<Button
							className='text-accent-contrast rounded-xl px-6 shadow-lg transition-shadow hover:shadow-xl'
							style={{
								background: `linear-gradient(to right, var(--accent-dark), var(--accent-color))`,
								boxShadow: `0 8px 24px -4px rgba(var(--accent-rgb), 0.2)`,
							}}>
							<Plus className='mr-2 h-4 w-4' />
							{t('settings:accounts.list.add')}
						</Button>
					</motion.div>
				</AddAccountDialog>
			</motion.div>

			<div className='stagger-children grid gap-5 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3'>
				{accounts.map((account, index) => (
					<motion.div
						key={account.id}
						{...(animationsEnabled
							? {
									initial: { opacity: 0, y: 20, scale: 0.96 },
									animate: { opacity: 1, y: 0, scale: 1 },
									transition: {
										delay: index * 0.08,
										duration: 0.4,
										ease: [0.16, 1, 0.3, 1],
									},
								}
							: {})}>
						<AccountCard
							account={account}
							onRemove={onRemoveAccount}
							onSync={onSyncAccount}
						/>
					</motion.div>
				))}

				{accounts.length === 0 && (
					<motion.div
						{...(animationsEnabled
							? {
									initial: { opacity: 0, scale: 0.95 },
									animate: { opacity: 1, scale: 1 },
									transition: { duration: 0.5, ease: [0.16, 1, 0.3, 1] },
								}
							: {})}
						className='col-span-full flex flex-col items-center justify-center rounded-2xl border-2 border-dashed border-white/[0.06] bg-white/[0.02] py-24 text-center transition-all duration-300 hover:border-white/[0.12] hover:bg-white/[0.04]'>
						<div className='mb-5 flex h-16 w-16 items-center justify-center rounded-2xl bg-slate-800/60 ring-1 ring-white/[0.06]'>
							<Mail className='h-7 w-7 text-slate-500' />
						</div>
						<h3 className='text-lg font-semibold text-slate-200'>
							No accounts connected
						</h3>
						<p className='mx-auto mt-2 mb-7 max-w-sm text-sm text-slate-500'>
							Connect your Gmail or Outlook account to start syncing messages.
						</p>
						<AddAccountDialog onAccountAdded={onAccountAdded} />
					</motion.div>
				)}
			</div>
		</div>
	)
}
