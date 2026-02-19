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
				<div className='flex min-h-[200px] flex-col items-center justify-center gap-4 rounded-xl border border-red-500/20 bg-red-500/5 p-8 text-center'>
					<div className='flex h-12 w-12 items-center justify-center rounded-full bg-red-500/10 ring-1 ring-red-500/20'>
						<AlertCircle className='h-6 w-6 text-red-400' />
					</div>
					<div className='space-y-1'>
						<h3 className='font-semibold text-slate-200'>
							{this.props.title}
						</h3>
						<p className='text-sm text-slate-400'>
							{this.props.description}
						</p>
					</div>
					<Button
						variant='outline'
						onClick={() => {
							this.setState({ hasError: false })
							this.props.onFallback()
						}}
						className='gap-2 border-white/5 bg-white/5 hover:bg-white/10'>
						<FileText className='h-4 w-4 text-slate-400' />
						{this.props.fallbackText}
					</Button>
				</div>
			)
		}

		return this.props.children
	}
}
