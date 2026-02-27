import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import * as opener from '@tauri-apps/plugin-opener'
import { useAccountsTranslation } from '../../../../hooks/useTypedTranslation'
import { RefreshCw, Loader2, Save, AlertCircle } from 'lucide-react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ManualAccountForm } from './ManualAccountForm'
import { useAccountStore } from '@/stores/accountStore'
import type { AccountMeta } from '@/types/accounts'
import { toast } from '@/components/ui/custom/Toaster'

interface EditAccountDialogProps {
	account: AccountMeta
	open: boolean
	onOpenChange: (open: boolean) => void
}

export function EditAccountDialog({ account, open, onOpenChange }: EditAccountDialogProps) {
	const { t } = useAccountsTranslation()
	const { updateAccount, setPendingReauthAccountId, pendingReauthAccountId } = useAccountStore()
	const [isLoading, setIsLoading] = useState(false)
	const [newName, setNewName] = useState(account.name)
	const prevPendingRef = useRef(pendingReauthAccountId)

	const isReauthing = pendingReauthAccountId === account.id

	useEffect(() => {
		// If we were reauthing and it became null, it means it's done
		if (prevPendingRef.current === account.id && pendingReauthAccountId === null) {
			onOpenChange(false)
		}
		prevPendingRef.current = pendingReauthAccountId
	}, [pendingReauthAccountId, account.id, onOpenChange])

	const isOAuth = account.auth_type === 'oauth2'

	const handleSaveName = async () => {
		setIsLoading(true)
		try {
			await invoke('update_account_name', { id: account.id, name: newName })
			updateAccount({ ...account, name: newName })
			toast.success(t('app.accountUpdated', 'Account updated successfully'))
		} catch (error) {
			console.error('Failed to update account name:', error)
			toast.error(t('errors.updateFailed', 'Failed to update account'))
		} finally {
			setIsLoading(false)
		}
	}

	const handleReauth = async () => {
		setIsLoading(true)
		setPendingReauthAccountId(account.id)
		try {
			const { url } = await invoke<{ url: string }>('start_oauth_flow', {
				provider: account.provider_type,
			})
			await opener.openUrl(url)
			// Don't close immediately anymore - wait for the callback
		} catch (error) {
			console.error('Failed to start re-auth:', error)
			setPendingReauthAccountId(null)
			toast.error(t('errors.reauthFailed', 'Failed to start re-authentication'))
		} finally {
			setIsLoading(false)
		}
	}

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className='border-slate-800 bg-slate-900/95 text-slate-100 backdrop-blur-xl sm:max-w-md'>
				<DialogHeader>
					<DialogTitle className='text-xl font-bold'>
						{t('settings:accounts.list.edit', 'Edit Account')}
					</DialogTitle>
					<DialogDescription className='text-slate-400'>
						{isOAuth 
							? t('settings:accounts.list.editOAuthDesc', 'Update name or re-connect your account.')
							: t('settings:accounts.list.editManualDesc', 'Update your account settings and credentials.')
						}
					</DialogDescription>
				</DialogHeader>

				<div className='py-4'>
					{isOAuth ? (
						<div className='space-y-6'>
							<div className='space-y-2'>
								<Label htmlFor='newName'>{t('settings:accounts.form.name', 'Account Name')}</Label>
								<div className='flex gap-2'>
									<Input
										id='newName'
										value={newName}
										onChange={(e) => setNewName(e.target.value)}
										className='border-slate-700 bg-slate-800/50'
									/>
									<Button 
										size='icon' 
										onClick={handleSaveName} 
										disabled={isLoading || newName === account.name}
										className='shrink-0 bg-blue-600 hover:bg-blue-500'
									>
										{isLoading ? <Loader2 className='h-4 w-4 animate-spin' /> : <Save className='h-4 w-4' />}
									</Button>
								</div>
							</div>

							<div className='rounded-xl border border-blue-500/20 bg-blue-500/5 p-4'>
								<div className='mb-3 flex items-start gap-3'>
									<RefreshCw className='mt-0.5 h-5 w-5 text-blue-400' />
									<div>
										<h4 className='text-sm font-semibold text-blue-100'>
											{t('settings:accounts.reauth.title', 'Re-authentication Required?')}
										</h4>
										<p className='mt-1 text-xs text-blue-300/80 leading-relaxed'>
											{t('settings:accounts.reauth.description', 'If you\'re having connection issues or your session expired, you can re-link your account.')}
										</p>
									</div>
								</div>
								<Button 
									onClick={handleReauth}
									disabled={isLoading || isReauthing}
									className='w-full bg-blue-600 hover:bg-blue-500'
								>
									{isReauthing ? (
										<>
											<Loader2 className='mr-2 h-4 w-4 animate-spin' />
											{t('settings:accounts.reauth.waiting', 'Waiting for browser...')}
										</>
									) : (
										<>
											{isLoading && <Loader2 className='mr-2 h-4 w-4 animate-spin' />}
											{t('settings:accounts.reauth.button', 'Re-authenticate')}
										</>
									)}
								</Button>
							</div>

							<div className='flex items-center gap-2 rounded-lg border border-yellow-500/20 bg-yellow-500/5 p-3 text-[11px] text-yellow-200/70'>
								<AlertCircle className='h-4 w-4 shrink-0 text-yellow-500/70' />
								<span>
									{t('settings:accounts.reauth.notice', 'Ensure you log in with the same email address: ')}
									<strong className='text-yellow-200'>{account.email}</strong>
								</span>
							</div>
						</div>
					) : (
						<ManualAccountForm 
							editAccount={account} 
							onSuccess={() => onOpenChange(false)} 
							onCancel={() => onOpenChange(false)} 
						/>
					)}
				</div>
			</DialogContent>
		</Dialog>
	)
}
