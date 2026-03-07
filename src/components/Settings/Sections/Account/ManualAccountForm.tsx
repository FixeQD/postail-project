import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Loader2, AlertCircle, Check } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useAccountsTranslation } from '@/hooks/useTypedTranslation'
import type { AccountMeta } from '@/types/accounts'
import { useAccountStore } from '@/stores/accountStore'
import type { ManualAccountFormProps } from '@/types/components/shared'

interface FormData {
	accountName: string
	email: string
	useSeparateUsername: boolean
	username: string
	password: string
	imapHost: string
	imapPort: string
	imapTls: boolean
	smtpHost: string
	smtpPort: string
	smtpTls: boolean
}

export function ManualAccountForm({ onSuccess, onCancel, editAccount }: ManualAccountFormProps) {
	const { t } = useTranslation()
	useAccountsTranslation()
	const updateAccount = useAccountStore((state) => state.updateAccount)
	const [isLoading, setIsLoading] = useState(false)
	const [error, setError] = useState<string | null>(null)
	const [formData, setFormData] = useState<FormData>({
		accountName: editAccount?.name || '',
		email: editAccount?.email || '',
		useSeparateUsername: false,
		username: '',
		password: '',
		imapHost: editAccount?.imap_host || '',
		imapPort: editAccount?.imap_port?.toString() || '993',
		imapTls: editAccount?.imap_tls ?? true,
		smtpHost: editAccount?.smtp_host || '',
		smtpPort: editAccount?.smtp_port?.toString() || '587',
		smtpTls: editAccount?.smtp_tls ?? true,
	})

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault()
		setError(null)
		setIsLoading(true)

		try {
			const config = {
				account_name: formData.accountName,
				email: formData.email,
				use_separate_username: formData.useSeparateUsername,
				username: formData.useSeparateUsername ? formData.username : null,
				password: formData.password,
				imap_host: formData.imapHost,
				imap_port: parseInt(formData.imapPort, 10),
				imap_tls: formData.imapTls,
				smtp_host: formData.smtpHost,
				smtp_port: parseInt(formData.smtpPort, 10),
				smtp_tls: formData.smtpTls,
			}

			if (editAccount) {
				const result = await invoke<AccountMeta>('update_custom_account', {
					id: editAccount.id,
					config,
				})
				updateAccount(result)
				onSuccess(editAccount.id)
			} else {
				const account = await invoke<AccountMeta>('add_custom_account', { config })
				onSuccess(account.id)
			}
		} catch (err) {
			const action = editAccount ? 'update' : 'add'
			console.error(`Failed to ${action} account:`, err)
			setError(err instanceof Error ? err.message : String(err))
		} finally {
			setIsLoading(false)
		}
	}

	const handleChange = (field: keyof FormData, value: string | boolean) => {
		setFormData((prev) => ({ ...prev, [field]: value }))
	}

	return (
		<form onSubmit={handleSubmit} className='space-y-6'>
			{error && (
				<div className='flex items-center gap-2 rounded-lg border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400'>
					<AlertCircle className='h-4 w-4 shrink-0' />
					<span>{error}</span>
				</div>
			)}

			<div className='space-y-4'>
				<div className='space-y-2'>
					<Label htmlFor='accountName'>{t('settings:accounts.manual.accountName')}</Label>
					<Input
						id='accountName'
						type='text'
						placeholder={t('settings:accounts.manual.placeholders.accountName')}
						value={formData.accountName}
						onChange={(e) => handleChange('accountName', e.target.value)}
						required
						className='border-black/10 bg-black/[0.03] dark:border-slate-700 dark:bg-slate-800/50'
					/>
				</div>

				<div className='space-y-2'>
					<Label htmlFor='email'>{t('settings:accounts.manual.emailAddress')}</Label>
					<Input
						id='email'
						type='email'
						placeholder={t('settings:accounts.manual.placeholders.email')}
						value={formData.email}
						onChange={(e) => handleChange('email', e.target.value)}
						required
						className='border-black/10 bg-black/[0.03] dark:border-slate-700 dark:bg-slate-800/50'
					/>
				</div>

				<div className='flex items-center gap-2'>
					<input
						type='checkbox'
						id='useSeparateUsername'
						checked={formData.useSeparateUsername}
						onChange={(e) => handleChange('useSeparateUsername', e.target.checked)}
						className='h-4 w-4 rounded border-black/20 bg-black/[0.03] text-blue-600 focus:ring-blue-500 dark:border-slate-600 dark:bg-slate-800'
					/>
					<Label
						htmlFor='useSeparateUsername'
						className='cursor-pointer text-sm font-normal'>
						Username is different than email
					</Label>
				</div>

				{formData.useSeparateUsername && (
					<div className='space-y-2'>
						<Label htmlFor='username'>{t('settings:accounts.manual.username')}</Label>
						<Input
							id='username'
							type='text'
							placeholder={t('settings:accounts.manual.placeholders.username')}
							value={formData.username}
							onChange={(e) => handleChange('username', e.target.value)}
							required={formData.useSeparateUsername}
							className='border-black/10 bg-black/[0.03] dark:border-slate-700 dark:bg-slate-800/50'
						/>
					</div>
				)}

				<div className='space-y-2'>
					<Label htmlFor='password'>
						{editAccount ? 'Verify/Update Password' : 'Password'}
					</Label>
					<Input
						id='password'
						type='password'
						placeholder={t('settings:accounts.manual.placeholders.password')}
						value={formData.password}
						onChange={(e) => handleChange('password', e.target.value)}
						required
						className='border-black/10 bg-black/[0.03] dark:border-slate-700 dark:bg-slate-800/50'
					/>
				</div>

				<div className='border-t border-black/[0.08] pt-4 dark:border-slate-800'>
					<h3 className='mb-4 text-sm font-semibold text-slate-600 dark:text-slate-300'>
						IMAP Settings
					</h3>
					<div className='grid gap-4 sm:grid-cols-2'>
						<div className='space-y-2 sm:col-span-2'>
							<Label htmlFor='imapHost'>{t('settings:accounts.manual.server')}</Label>
							<Input
								id='imapHost'
								type='text'
								placeholder={t('settings:accounts.manual.placeholders.imapHost')}
								value={formData.imapHost}
								onChange={(e) => handleChange('imapHost', e.target.value)}
								required
								className='border-black/10 bg-black/[0.03] dark:border-slate-700 dark:bg-slate-800/50'
							/>
						</div>
						<div className='space-y-2'>
							<Label htmlFor='imapPort'>{t('settings:accounts.manual.port')}</Label>
							<Input
								id='imapPort'
								type='number'
								placeholder={t('settings:accounts.manual.placeholders.imapPort')}
								value={formData.imapPort}
								onChange={(e) => handleChange('imapPort', e.target.value)}
								required
								min='1'
								max='65535'
								className='border-black/10 bg-black/[0.03] dark:border-slate-700 dark:bg-slate-800/50'
							/>
						</div>
						<div className='flex items-center gap-2 pt-6'>
							<input
								type='checkbox'
								id='imapTls'
								checked={formData.imapTls}
								onChange={(e) => handleChange('imapTls', e.target.checked)}
								className='h-4 w-4 rounded border-black/20 bg-black/[0.03] text-blue-600 focus:ring-blue-500 dark:border-slate-600 dark:bg-slate-800'
							/>
							<Label htmlFor='imapTls' className='cursor-pointer text-sm font-normal'>
								Use TLS
							</Label>
						</div>
					</div>
				</div>

				<div className='border-t border-black/[0.08] pt-4 dark:border-slate-800'>
					<h3 className='mb-4 text-sm font-semibold text-slate-600 dark:text-slate-300'>
						SMTP Settings
					</h3>
					<div className='grid gap-4 sm:grid-cols-2'>
						<div className='space-y-2 sm:col-span-2'>
							<Label htmlFor='smtpHost'>{t('settings:accounts.manual.server')}</Label>
							<Input
								id='smtpHost'
								type='text'
								placeholder={t('settings:accounts.manual.placeholders.smtpHost')}
								value={formData.smtpHost}
								onChange={(e) => handleChange('smtpHost', e.target.value)}
								required
								className='border-black/10 bg-black/[0.03] dark:border-slate-700 dark:bg-slate-800/50'
							/>
						</div>
						<div className='space-y-2'>
							<Label htmlFor='smtpPort'>{t('settings:accounts.manual.port')}</Label>
							<Input
								id='smtpPort'
								type='number'
								placeholder={t('settings:accounts.manual.placeholders.smtpPort')}
								value={formData.smtpPort}
								onChange={(e) => handleChange('smtpPort', e.target.value)}
								required
								min='1'
								max='65535'
								className='border-black/10 bg-black/[0.03] dark:border-slate-700 dark:bg-slate-800/50'
							/>
						</div>
						<div className='flex items-center gap-2 pt-6'>
							<input
								type='checkbox'
								id='smtpTls'
								checked={formData.smtpTls}
								onChange={(e) => handleChange('smtpTls', e.target.checked)}
								className='h-4 w-4 rounded border-black/20 bg-black/[0.03] text-blue-600 focus:ring-blue-500 dark:border-slate-600 dark:bg-slate-800'
							/>
							<Label htmlFor='smtpTls' className='cursor-pointer text-sm font-normal'>
								Use TLS
							</Label>
						</div>
					</div>
				</div>
			</div>

			<div className='flex gap-3 pt-4'>
				<Button
					type='button'
					variant='outline'
					onClick={onCancel}
					disabled={isLoading}
					className='flex-1 border-black/10 bg-black/[0.03] hover:bg-black/[0.06] dark:border-slate-700 dark:bg-slate-800/50 dark:hover:bg-slate-800'>
					Cancel
				</Button>
				<Button
					type='submit'
					disabled={isLoading}
					className='flex-1 bg-blue-600 hover:bg-blue-500'>
					{isLoading ? (
						<>
							<Loader2 className='mr-2 h-4 w-4 animate-spin' />
							Testing...
						</>
					) : (
						<>
							<Check className='mr-2 h-4 w-4' />
							{editAccount ? 'Test & Save Changes' : 'Test & Add Account'}
						</>
					)}
				</Button>
			</div>
		</form>
	)
}
