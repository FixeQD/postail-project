import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { Mailbox } from '@/types/mail'
import * as opener from '@tauri-apps/plugin-opener'
import { useAccountsTranslation } from '@/hooks/useTypedTranslation'
import { RefreshCw, Loader2, Save, AlertCircle, FolderCog } from 'lucide-react'
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
import { MailboxRoleStep } from './MailboxRoleStep'
import { useAccountStore } from '@/stores/accountStore'
import { useShellTransition } from '@/hooks/useShellTransition'
import { toast } from '@/components/ui/custom/Toaster'
import type { EditAccountDialogProps } from '@/types/components/shared'

type EditView = 'main' | 'mailbox-roles'

export function EditAccountDialog({ account, open, onOpenChange }: EditAccountDialogProps) {
	const { t } = useAccountsTranslation()
	const { updateAccount, setPendingReauthAccountId, pendingReauthAccountId } = useAccountStore()
	const [isLoading, setIsLoading] = useState(false)
	const [newName, setNewName] = useState(account.name)
	const [view, setView] = useState<EditView>('main')
	const [preloadedMailboxes, setPreloadedMailboxes] = useState<Mailbox[] | null>(null)
	const prevPendingRef = useRef(pendingReauthAccountId)
	const { shellScope, contentScope, transition, reset } = useShellTransition()

	const isReauthing = pendingReauthAccountId === account.id
	const isOAuth = account.auth_type === 'oauth2'

	useEffect(() => {
		// reauth done when pending clears
		if (prevPendingRef.current === account.id && pendingReauthAccountId === null) {
			onOpenChange(false)
		}
		prevPendingRef.current = pendingReauthAccountId
	}, [pendingReauthAccountId, account.id, onOpenChange])

	const handleOpenChange = (o: boolean) => {
		if (!o) {
			reset()
			setView('main')
			setPreloadedMailboxes(null)
		}
		onOpenChange(o)
	}

	const handleOpenMailboxRoles = () =>
		transition(async () => {
			const mbs = await invoke<Mailbox[]>('fetch_mailboxes', { accountId: account.id }).catch(
				() => []
			)
			setPreloadedMailboxes(mbs)
			setView('mailbox-roles')
		})

	const handleBackToMain = () => {
		setPreloadedMailboxes(null)
		transition(() => setView('main'))
	}

	const handleSaveName = async () => {
		setIsLoading(true)
		try {
			await invoke('update_account_name', { id: account.id, name: newName })
			updateAccount({ ...account, name: newName })
			toast.success(t('app.accountUpdated'))
		} catch (error) {
			console.error('Failed to update account name:', error)
			toast.error(t('errors.updateFailed'))
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
		} catch (error) {
			console.error('Failed to start re-auth:', error)
			setPendingReauthAccountId(null)
			toast.error(t('errors.reauthFailed'))
		} finally {
			setIsLoading(false)
		}
	}

	return (
		<Dialog open={open} onOpenChange={handleOpenChange}>
			<DialogContent className='glass overflow-hidden border-[var(--border-subtle)] bg-[var(--surface-glass)] p-0 text-[var(--text-primary)] sm:max-w-md'>
				<div ref={shellScope} className='w-full'>
					<div ref={contentScope} className='p-6'>
						{view === 'mailbox-roles' ? (
							<MailboxRoleStep
								accountId={account.id}
								onDone={handleBackToMain}
								initialMailboxes={preloadedMailboxes ?? undefined}
							/>
						) : (
							<>
								<DialogHeader className='mb-4'>
									<DialogTitle className='text-xl font-bold'>
										{t('settings:accounts.list.edit')}
									</DialogTitle>
									<DialogDescription className='text-[var(--text-secondary)]'>
										{isOAuth
											? t(
													'settings:accounts.list.editOAuthDesc',
													'Update name or re-connect your account.'
												)
											: t(
													'settings:accounts.list.editManualDesc',
													'Update your account settings and credentials.'
												)}
									</DialogDescription>
								</DialogHeader>

								<div className='space-y-4'>
									{isOAuth ? (
										<div className='space-y-6'>
											<div className='space-y-2'>
												<Label htmlFor='newName'>
													{t(
														'settings:accounts.form.name',
														'Account Name'
													)}
												</Label>
												<div className='flex gap-2'>
													<Input
														id='newName'
														value={newName}
														onChange={(e) => setNewName(e.target.value)}
														className='border-[var(--border-subtle)] bg-[var(--surface-panel)]'
													/>
													<Button
														size='icon'
														onClick={handleSaveName}
														disabled={
															isLoading || newName === account.name
														}
														className='shrink-0 bg-status-info hover:bg-status-info'>
														{isLoading ? (
															<Loader2 className='h-4 w-4 animate-spin' />
														) : (
															<Save className='h-4 w-4' />
														)}
													</Button>
												</div>
											</div>

											<div className='rounded-xl border border-status-info/30 bg-status-info/15 p-4'>
												<div className='mb-3 flex items-start gap-3'>
													<RefreshCw className='mt-0.5 h-5 w-5 text-status-info' />
													<div>
														<h4 className='text-sm font-semibold text-status-info'>
															{t(
																'settings:accounts.reauth.title',
																'Re-authentication Required?'
															)}
														</h4>
														<p className='mt-1 text-xs leading-relaxed text-status-info/80'>
															{t(
																'settings:accounts.reauth.description',
																"If you're having connection issues or your session expired, you can re-link your account."
															)}
														</p>
													</div>
												</div>
												<Button
													onClick={handleReauth}
													disabled={isLoading || isReauthing}
													className='w-full bg-status-info hover:bg-status-info'>
													{isReauthing ? (
														<>
															<Loader2 className='mr-2 h-4 w-4 animate-spin' />
															{t(
																'settings:accounts.reauth.waiting',
																'Waiting for browser...'
															)}
														</>
													) : (
														<>
															{isLoading && (
																<Loader2 className='mr-2 h-4 w-4 animate-spin' />
															)}
															{t(
																'settings:accounts.reauth.button',
																'Re-authenticate'
															)}
														</>
													)}
												</Button>
											</div>

											<div className='flex items-center gap-2 rounded-lg border border-status-warning/30 bg-status-warning/15 p-3 text-[11px] text-status-warning/70'>
												<AlertCircle className='h-4 w-4 shrink-0 text-status-warning' />
												<span>
													{t(
														'settings:accounts.reauth.notice',
														'Ensure you log in with the same email address: '
													)}
													<strong className='text-status-warning'>
														{account.email}
													</strong>
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

									<button
										type='button'
										onClick={handleOpenMailboxRoles}
										className='flex w-full items-center gap-2.5 rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] px-4 py-3 text-sm text-[var(--text-secondary)] transition-colors hover:border-[var(--text-tertiary)] hover:bg-[var(--surface-hover)] hover:text-[var(--text-primary)]'>
										<FolderCog className='h-4 w-4 shrink-0' />
										{t(
											'settings:accounts.list.manageRoles',
											'Manage mailbox roles'
										)}
									</button>
								</div>
							</>
						)}
					</div>
				</div>
			</DialogContent>
		</Dialog>
	)
}
