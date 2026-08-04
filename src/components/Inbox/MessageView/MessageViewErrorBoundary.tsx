import { Component, ErrorInfo, ReactNode } from 'react'
import { AlertCircle, FileText } from 'lucide-react'
import { Button } from '@/components/ui/button'

interface Props {
	children: ReactNode
	onFallback: () => void
	title: string
	description: string
	fallbackText: string
}

interface State {
	hasError: boolean
}

export class MessageViewErrorBoundary extends Component<Props, State> {
	public state: State = {
		hasError: false,
	}

	public static getDerivedStateFromError(_: Error): State {
		return { hasError: true }
	}

	public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
		console.error('MessageView rendering error:', error, errorInfo)
	}

	public render() {
		if (this.state.hasError) {
			return (
				<div className='flex min-h-[200px] flex-col items-center justify-center gap-4 rounded-xl border border-destructive/30 bg-destructive/15 p-8 text-center'>
					<div className='flex h-12 w-12 items-center justify-center rounded-full bg-destructive/15 ring-1 ring-destructive/30'>
						<AlertCircle className='h-6 w-6 text-destructive' />
					</div>
					<div className='space-y-1'>
						<h3 className='font-semibold text-[var(--text-primary)]'>
							{this.props.title}
						</h3>
						<p className='text-sm text-[var(--text-secondary)]'>
							{this.props.description}
						</p>
					</div>
					<Button
						variant='outline'
						onClick={() => {
							this.setState({ hasError: false })
							this.props.onFallback()
						}}
						className='gap-2 border-[var(--border-faint)] bg-[var(--surface-panel)] hover:bg-[var(--surface-hover)]'>
						<FileText className='h-4 w-4 text-[var(--text-secondary)]' />
						{this.props.fallbackText}
					</Button>
				</div>
			)
		}

		return this.props.children
	}
}
