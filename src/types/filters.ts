export type MatchMode = 'all' | 'any'

export type ConditionField = 'from' | 'to' | 'subject' | 'body'

export type ConditionOperator =
	| 'contains'
	| 'not_contains'
	| 'equals'
	| 'not_equals'
	| 'starts_with'
	| 'ends_with'

export interface RuleCondition {
	field: ConditionField
	operator: ConditionOperator
	value: string
}

export type ActionType = 'move_to' | 'add_tag' | 'star' | 'mark_read' | 'delete'

export interface RuleAction {
	action_type: ActionType
	value?: string
}

export interface FilterRule {
	id: string
	account_id: string
	name: string
	match_mode: MatchMode
	conditions: RuleCondition[]
	actions: RuleAction[]
	position: number
	enabled: boolean
}
