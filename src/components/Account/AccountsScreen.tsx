import { motion } from 'framer-motion'
import { Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useAccountsTranslation } from '@/hooks/useTypedTranslation'
import { AccountCard } from './AccountCard'
import { AddAccountDialog } from './AddAccountDialog'
import type { AccountMeta } from '@/types/accounts'

interface AccountsScreenProps {
	accounts: AccountMeta[]
	onAccountAdded: () => void
	onRemoveAccount: (id: string) => void
	onSyncAccount: (id: string) => void
}

export function AccountsScreen({
	accounts,
	onRemoveAccount,
	onSyncAccount
}: AccountsScreenProps) {
	const { t } = useAccountsTranslation()

	return (
		<div className="flex h-full flex-col p-8 overflow-y-auto">
			<motion.div
				initial={{ opacity: 0, y: -20 }}
				animate={{ opacity: 1, y: 0 }}
				className="mb-8 flex items-center justify-between"
			>
				<div>
					<h1 className="text-3xl font-bold text-slate-100 tracking-tight">{t('accounts:title')}</h1>
					<p className="text-slate-400 mt-1">{t('accounts:subtitle')}</p>
				</div>
				<AddAccountDialog>
					<Button className="bg-slate-100 text-slate-900 hover:bg-white shadow-lg hover:shadow-xl transition-all rounded-full px-6">
						<Plus className="mr-2 h-4 w-4" />
						{t('accounts:list.add')}
					</Button>
				</AddAccountDialog>
			</motion.div>

			<div className="grid gap-6 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
				{accounts.map((account) => (
					<AccountCard
						key={account.id}
						account={account}
						onRemove={onRemoveAccount}
						onSync={onSyncAccount}
					/>
				))}
				
				{/* Empty state or "Add" card if list is empty */}
				{accounts.length === 0 && (
					<motion.div
						initial={{ opacity: 0, scale: 0.95 }}
						animate={{ opacity: 1, scale: 1 }}
						className="col-span-full flex flex-col items-center justify-center rounded-2xl border-2 border-dashed border-slate-700 bg-slate-800/20 py-24 text-center transition-colors hover:border-slate-600 hover:bg-slate-800/40"
					>
						<div className="rounded-full bg-slate-800 p-4 mb-4">
							<Plus className="h-8 w-8 text-slate-400" />
						</div>
						<h3 className="text-lg font-semibold text-slate-200">No accounts connected</h3>
						<p className="text-slate-500 mb-6 max-w-sm mx-auto">
							Connect your Gmail or Outlook account to start syncing messages.
						</p>
						<AddAccountDialog />
					</motion.div>
				)}
			</div>
		</div>
	)
}
