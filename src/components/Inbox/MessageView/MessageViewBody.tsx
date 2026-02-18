
import { useEffect, useRef, useState } from 'react'
import { useThemeStore } from '@/stores/themeStore'

interface MessageViewBodyProps {
	htmlContent: string
	plainContent: string
	viewMode: 'html' | 'plain'
}

export const MessageViewBody = ({
	htmlContent,
	plainContent,
	viewMode,
}: MessageViewBodyProps) => {
	const accentColor = useThemeStore((s) => s.accentColor)
	const iframeRef = useRef<HTMLIFrameElement>(null)
	const [blobUrl, setBlobUrl] = useState<string>('')

	// Fallback to plain text if no HTML content
	const effectiveViewMode =
		!htmlContent || !htmlContent.trim() ? 'plain' : viewMode

	useEffect(() => {
		if (effectiveViewMode !== 'html') return

		const html = `
      <!DOCTYPE html>
      <html>
        <head>
          <meta charset="utf-8">
          <style>
            body {
              margin: 0;
              padding: 16px;
              font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
              color: #e2e8f0;
              background: transparent;
              font-size: 14px;
              line-height: 1.6;
              word-break: break-word;
            }
            a {
              color: ${accentColor};
              text-decoration: none;
            }
            a:hover {
              text-decoration: underline;
            }
            img {
              max-width: 100%;
              height: auto;
            }
            /* Custom scrollbar for iframe content */
            ::-webkit-scrollbar {
              width: 8px;
              height: 8px;
            }
            ::-webkit-scrollbar-track {
              background: transparent;
            }
            ::-webkit-scrollbar-thumb {
              background: rgba(255, 255, 255, 0.1);
              border-radius: 4px;
            }
            ::-webkit-scrollbar-thumb:hover {
              background: rgba(255, 255, 255, 0.2);
            }
          </style>
        </head>
        <body>
          ${htmlContent}
          <script>
            // Resize observer to communicate height to parent
            const resizeObserver = new ResizeObserver(() => {
              window.parent.postMessage({ type: 'resize', height: document.body.scrollHeight }, '*')
            });
            resizeObserver.observe(document.body);
            // Initial height
            window.parent.postMessage({ type: 'resize', height: document.body.scrollHeight }, '*');

            // Link click handler
            document.addEventListener('click', function(e) {
              const a = e.target.closest('a');
              if (a && a.href) {
                e.preventDefault();
                window.parent.postMessage({ type: 'open-link', url: a.href }, '*');
              }
            });
          </script>
        </body>
      </html>
    `

		const blob = new Blob([html], { type: 'text/html' })
		const url = URL.createObjectURL(blob)
		setBlobUrl(url)

		return () => {
			URL.revokeObjectURL(url)
		}
	}, [htmlContent, effectiveViewMode, accentColor])

	// Listen for messages from iframe (resize, links)
	useEffect(() => {
		const handler = (e: MessageEvent) => {
			if (!iframeRef.current) return

			// Resize
			if (e.data?.type === 'resize' && typeof e.data.height === 'number') {
				iframeRef.current.style.height = `${e.data.height}px`
			}

			// Open link
			if (e.data?.type === 'open-link' && e.data.url) {
				import('@tauri-apps/plugin-opener').then(({ openUrl }) => {
					openUrl(e.data.url)
				})
			}
		}

		window.addEventListener('message', handler)
		return () => window.removeEventListener('message', handler)
	}, [])

	if (effectiveViewMode === 'plain') {
		return (
			<pre className='message-view-plain whitespace-pre-wrap break-words p-4 font-mono text-sm text-slate-300'>
				{plainContent || '(No content)'}
			</pre>
		)
	}

	return (
		<iframe
			ref={iframeRef}
			title='Message Content'
			src={blobUrl}
			// sandbox must include allow-scripts for the resize/link logic inside blob
			sandbox='allow-scripts allow-popups allow-popups-to-escape-sandbox'
			className='message-view-iframe w-full border-none'
			style={{ minHeight: '300px' }}
		/>
	)
}
