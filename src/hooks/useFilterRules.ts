import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { FilterRule } from '@/types/filters'
import { toast } from '@/components/ui/custom/Toaster'

export function useFilterRules(accountId: string) {
	const queryClient = useQueryClient()

	const query = useQuery<FilterRule[]>({
		queryKey: ['filter-rules', accountId],
		queryFn: () => invoke<FilterRule[]>('get_filter_rules', { accountId }),
		enabled: !!accountId,
	})

	const saveRule = useMutation({
		mutationFn: (rule: FilterRule) => invoke('save_filter_rule', { rule }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ['filter-rules', accountId] })
			toast.success('Rule saved successfully')
		},
		onError: (error) => {
			toast.error('Failed to save rule', {
				description: String(error),
			})
		},
	})

	const deleteRule = useMutation({
		mutationFn: (ruleId: string) => invoke('delete_filter_rule', { accountId, ruleId }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ['filter-rules', accountId] })
			toast.success('Rule deleted')
		},
		onError: (error) => {
			toast.error('Failed to delete rule', {
				description: String(error),
			})
		},
	})

	const reorderRules = useMutation({
		mutationFn: (orderedIds: string[]) => invoke('reorder_filter_rules', { accountId, orderedIds }),
		onMutate: async (orderedIds) => {
			// Optimistic update
			await queryClient.cancelQueries({ queryKey: ['filter-rules', accountId] })
			const previousRules = queryClient.getQueryData<FilterRule[]>(['filter-rules', accountId])
			if (previousRules) {
				const reordered = [...previousRules].sort((a, b) => {
					const idxA = orderedIds.indexOf(a.id)
					const idxB = orderedIds.indexOf(b.id)
					return idxA - idxB
				})
				queryClient.setQueryData(['filter-rules', accountId], reordered)
			}
			return { previousRules }
		},
		onError: (err, _orderedIds, context) => {
			if (context?.previousRules) {
				queryClient.setQueryData(['filter-rules', accountId], context.previousRules)
			}
			toast.error('Failed to reorder rules', {
				description: String(err),
			})
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: ['filter-rules', accountId] })
		},
	})

	const applyRulesToMailbox = useMutation({
		mutationFn: (mailbox: string) => invoke('apply_filters_to_mailbox', { accountId, mailbox }),
		onSuccess: (count) => {
			toast.success(`Rules applied to ${count} messages`)
			queryClient.invalidateQueries({ queryKey: ['messages', accountId] })
		},
		onError: (error) => {
			toast.error('Failed to apply rules', {
				description: String(error),
			})
		},
	})

	return {
		rules: query.data ?? [],
		isLoading: query.isLoading,
		saveRule: saveRule.mutateAsync,
		isSaving: saveRule.isPending,
		deleteRule: deleteRule.mutateAsync,
		isDeleting: deleteRule.isPending,
		reorderRules: reorderRules.mutateAsync,
		applyRulesToMailbox: applyRulesToMailbox.mutateAsync,
		isApplying: applyRulesToMailbox.isPending,
	}
}
