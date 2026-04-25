import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { 
    Users, 
    Plus, 
    ChevronDown, 
    ChevronRight, 
    Edit2, 
    Trash2,
    Check,
    X,
    Palette
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { toast } from '@/components/ui/custom/Toaster'
import { ColorPicker } from '@/components/ui/custom/ColorPicker'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import type { ContactGroup } from '@/types/components/compose'
import { useContactsTranslation } from '@/hooks/useTypedTranslation'

interface GroupsSidebarProps {
    selectedGroupId: number | null
    onSelectGroup: (id: number | null) => void
}

export function GroupsSidebar({ selectedGroupId, onSelectGroup }: GroupsSidebarProps) {
    const { t } = useContactsTranslation()
    const queryClient = useQueryClient()
    const [isExpanded, setIsExpanded] = useState(true)
    const [isCreating, setIsCreating] = useState(false)
    const [newGroupName, setNewGroupName] = useState('')
    const [newGroupColor, setNewGroupColor] = useState<string>('#3b82f6')
    const [editingGroupId, setEditingGroupId] = useState<number | null>(null)
    const [editName, setEditName] = useState('')
    const [dragOverGroupId, setDragOverGroupId] = useState<number | null>(null)

    const { data: groups = [] } = useQuery({
        queryKey: ['contact-groups'],
        queryFn: () => invoke<ContactGroup[]>('list_contact_groups')
    })

    const createMutation = useMutation({
        mutationFn: ({ name, color }: { name: string; color: string }) => 
            invoke('create_contact_group', { name, color }),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['contact-groups'] })
            setIsCreating(false)
            setNewGroupName('')
            toast.success(t('contacts:groups.createSuccess'))
        },
        onError: () => toast.error(t('contacts:groups.createFailed'))
    })

    const deleteMutation = useMutation({
        mutationFn: (id: number) => invoke('delete_contact_group', { id }),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['contact-groups'] })
            if (selectedGroupId === editingGroupId) onSelectGroup(null)
            toast.success(t('contacts:groups.deleteSuccess'))
        },
        onError: () => toast.error(t('contacts:groups.deleteFailed'))
    })

    const renameMutation = useMutation({
        mutationFn: ({ id, name }: { id: number; name: string }) => 
            invoke('rename_contact_group', { id, name }),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['contact-groups'] })
            setEditingGroupId(null)
            toast.success(t('contacts:groups.renameSuccess'))
        },
        onError: () => toast.error(t('contacts:groups.renameFailed'))
    })

    const handleCreate = () => {
        if (!newGroupName.trim()) return
        createMutation.mutate({ name: newGroupName.trim(), color: newGroupColor })
    }

    const handleRename = (id: number) => {
        if (!editName.trim()) return
        renameMutation.mutate({ id, name: editName.trim() })
    }

    const handleDragOver = (e: React.DragEvent, groupId: number) => {
        if (e.dataTransfer.types.includes('application/postail-contact-id')) {
            e.preventDefault()
            setDragOverGroupId(groupId)
            e.dataTransfer.dropEffect = 'link'
        }
    }

    const handleDragLeave = () => {
        setDragOverGroupId(null)
    }

    const handleDrop = async (e: React.DragEvent, groupId: number) => {
        e.preventDefault()
        setDragOverGroupId(null)
        
        const contactIdStr = e.dataTransfer.getData('application/postail-contact-id')
        if (contactIdStr) {
            const contactId = parseInt(contactIdStr)
            try {
                await invoke('add_contact_to_group', { groupId, contactId })
                queryClient.invalidateQueries({ queryKey: ['contact-groups'] })
                toast.success(t('contacts:groups.addSuccess'))
            } catch {
                toast.error(t('contacts:groups.addFailed'))
            }
        }
    }

    return (
        <div className='flex flex-col gap-1 py-2'>
            <div 
                className='group flex items-center justify-between px-4 py-2 cursor-pointer'
                onClick={() => setIsExpanded(!isExpanded)}
            >
                <div className='flex items-center gap-2 text-[11px] font-bold uppercase tracking-wider text-[var(--text-tertiary)]'>
                    {isExpanded ? <ChevronDown className='h-3 w-3' /> : <ChevronRight className='h-3 w-3' />}
                    {t('contacts:groups.title')}
                </div>
                <button
                    onClick={(e) => {
                        e.stopPropagation()
                        setIsCreating(true)
                    }}
                    className='opacity-0 group-hover:opacity-100 transition-opacity p-1 hover:bg-[var(--surface-hover)] rounded'
                >
                    <Plus className='h-3 w-3 text-[var(--text-tertiary)]' />
                </button>
            </div>

            <AnimatePresence>
                {isExpanded && (
                    <motion.div
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: 'auto', opacity: 1 }}
                        exit={{ height: 0, opacity: 0 }}
                        className='overflow-hidden'
                    >
                        <div 
                            className={`flex items-center gap-3 px-4 py-2 text-[13px] cursor-pointer transition-colors ${
                                selectedGroupId === null ? 'bg-[var(--surface-active)] text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-[var(--surface-hover)]'
                            }`}
                            onClick={() => onSelectGroup(null)}
                        >
                            <Users className='h-3.5 w-3.5' />
                            <span className='flex-1'>{t('contacts:groups.allContacts')}</span>
                        </div>

                        {groups.map(group => (
                            <div key={group.id} className='relative group/item'>
                                {editingGroupId === group.id ? (
                                    <div className='flex items-center gap-2 px-4 py-1.5'>
                                        <input
                                            autoFocus
                                            value={editName}
                                            onChange={(e) => setEditName(e.target.value)}
                                            onKeyDown={(e) => {
                                                if (e.key === 'Enter') handleRename(group.id)
                                                if (e.key === 'Escape') setEditingGroupId(null)
                                            }}
                                            className='flex-1 bg-[var(--surface-active)] border border-[var(--border-subtle)] rounded px-2 py-1 text-[13px] outline-none focus:border-rgb(var(--accent-rgb))'
                                        />
                                        <button onClick={() => handleRename(group.id)} className='text-green-500'><Check className='h-3.5 w-3.5' /></button>
                                        <button onClick={() => setEditingGroupId(null)} className='text-red-500'><X className='h-3.5 w-3.5' /></button>
                                    </div>
                                ) : (
                                    <div 
                                        className={`flex items-center gap-3 px-4 py-2 text-[13px] cursor-pointer transition-colors ${
                                            selectedGroupId === group.id 
                                                ? 'bg-[var(--surface-active)] text-[var(--text-primary)]' 
                                                : dragOverGroupId === group.id
                                                ? 'bg-[rgba(var(--accent-rgb),0.1)] text-[rgb(var(--accent-rgb))] shadow-[inset_0_0_0_1px_rgba(var(--accent-rgb),0.2)]'
                                                : 'text-[var(--text-secondary)] hover:bg-[var(--surface-hover)]'
                                        }`}
                                        onClick={() => onSelectGroup(group.id)}
                                        onDragOver={(e) => handleDragOver(e, group.id)}
                                        onDragLeave={handleDragLeave}
                                        onDrop={(e) => handleDrop(e, group.id)}
                                    >
                                        <div 
                                            className='h-2 w-2 rounded-full' 
                                            style={{ backgroundColor: group.color || 'rgb(var(--accent-rgb))' }} 
                                        />
                                        <span className='flex-1 truncate'>{group.name}</span>
                                        <span className='text-[10px] text-[var(--text-tertiary)]'>{group.member_count}</span>
                                        
                                        <div className='opacity-0 group-hover/item:opacity-100 flex items-center gap-1'>
                                            <button 
                                                onClick={(e) => {
                                                    e.stopPropagation()
                                                    setEditingGroupId(group.id)
                                                    setEditName(group.name)
                                                }}
                                                className='p-1 hover:bg-[var(--surface-active)] rounded'
                                            >
                                                <Edit2 className='h-3 w-3 text-[var(--text-tertiary)]' />
                                            </button>
                                            <button 
                                                onClick={(e) => {
                                                    e.stopPropagation()
                                                    if (confirm(t('contacts:groups.deleteConfirm'))) {
                                                        deleteMutation.mutate(group.id)
                                                    }
                                                }}
                                                className='p-1 hover:bg-[var(--surface-active)] rounded hover:text-red-500'
                                            >
                                                <Trash2 className='h-3 w-3 text-[var(--text-tertiary)]' />
                                            </button>
                                        </div>
                                    </div>
                                )}
                            </div>
                        ))}

                        {isCreating && (
                            <div className='flex flex-col gap-2 px-4 py-2 border-t border-[var(--border-subtle)] mt-1'>
                                <div className='flex items-center gap-2'>
                                    <input
                                        autoFocus
                                        placeholder={t('contacts:groups.placeholder')}
                                        value={newGroupName}
                                        onChange={(e) => setNewGroupName(e.target.value)}
                                        onKeyDown={(e) => {
                                            if (e.key === 'Enter') handleCreate()
                                            if (e.key === 'Escape') setIsCreating(false)
                                        }}
                                        className='flex-1 bg-[var(--surface-active)] border border-[var(--border-subtle)] rounded px-2 py-1 text-[13px] outline-none focus:border-rgb(var(--accent-rgb))'
                                    />
                                    <Popover>
                                        <PopoverTrigger asChild>
                                            <button className='p-1 hover:bg-[var(--surface-active)] rounded transition-colors'>
                                                <Palette className='h-3.5 w-3.5' style={{ color: newGroupColor }} />
                                            </button>
                                        </PopoverTrigger>
                                        <PopoverContent className='w-auto p-4' align='end'>
                                            <ColorPicker color={newGroupColor} onChange={setNewGroupColor} />
                                        </PopoverContent>
                                    </Popover>
                                </div>
                                <div className='flex items-center justify-end gap-2'>
                                    <button 
                                        onClick={() => setIsCreating(false)}
                                        className='px-2 py-1 text-[11px] font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                                    >
                                        {t('common:actions.cancel')}
                                    </button>
                                    <button 
                                        onClick={handleCreate}
                                        className='px-2 py-1 text-[11px] font-medium bg-[rgb(var(--accent-rgb))] text-white rounded hover:brightness-110'
                                    >
                                        {t('common:actions.save')}
                                    </button>
                                </div>
                            </div>
                        )}
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    )
}
