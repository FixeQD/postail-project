import { useEffect, useRef, useState } from 'react'
import { useThemeStore } from '@/stores/themeStore'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { openUrl } from '@tauri-apps/plugin-opener'

interface MessageViewBodyProps {
	htmlContent: string
	plainContent: string
	viewMode: 'html' | 'plain'
	allowExternalResources?: boolean
}

export const MessageViewBody = ({
	htmlContent,
	plainContent,
	viewMode,
	allowExternalResources = false,
}: MessageViewBodyProps) => {
	const { t } = useTypedTranslation(['security', 'common'])
	const accentColor = useThemeStore((s) => s.accentColor)
	const iframeRef = useRef<HTMLIFrameElement>(null)
	const containerRef = useRef<HTMLDivElement>(null)
	const [blobUrl, setBlobUrl] = useState<string>('')
	const [warningOpen, setWarningOpen] = useState(false)
	const [pendingUrl, setPendingUrl] = useState<string | null>(null)
	const [iframeWidth, setIframeWidth] = useState<string>('100%')
	const [iframeReady, setIframeReady] = useState(false)

	// Fallback to plain text if no HTML content
	const effectiveViewMode =
		!htmlContent || !htmlContent.trim() ? 'plain' : viewMode

	useEffect(() => {
		if (effectiveViewMode !== 'html') return

		// Detect if email supports dark mode
		const hasDarkModeSupport =
			htmlContent.includes('prefers-color-scheme: dark') ||
			htmlContent.includes('data-ogsc') ||
			htmlContent.includes('data-ogsb')

		const iframeColorScheme = hasDarkModeSupport ? 'dark light' : 'light'
		const iframeBg = hasDarkModeSupport ? 'transparent' : '#ffffff'
		const iframeTextColor = hasDarkModeSupport ? 'inherit' : '#1a1a1a'

		const csp = allowExternalResources
			? `
        default-src 'none';
        script-src 'unsafe-inline';
        style-src 'unsafe-inline';
        img-src * data: cid:;
        font-src * data:;
        connect-src 'none';
      `
			: `
        default-src 'none';
        script-src 'unsafe-inline';
        style-src 'unsafe-inline';
        img-src data: cid:;
        font-src data:;
        connect-src 'none';
      `

		const html = `
      <!DOCTYPE html>
      <html>
        <head>
          <meta charset="utf-8">
          <meta http-equiv="Content-Security-Policy" content="${csp}">
          <style>
            * {
              box-sizing: border-box;
            }
            body {
              margin: 0;
              padding: 24px;
              font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
              color: ${iframeTextColor};
              background: ${iframeBg};
              font-size: 14px;
              line-height: 1.6;
              word-break: break-word;
              color-scheme: ${iframeColorScheme};
            }
            a {
              color: ${accentColor};
              text-decoration: none;
            }
            a:hover {
              text-decoration: underline;
            }
            img {
              max-width: 100% !important;
              height: auto !important;
            }
            table {
              max-width: 100% !important;
            }
            pre {
              overflow-x: auto;
              max-width: 100%;
            }
            ::-webkit-scrollbar {
              width: 8px;
              height: 8px;
            }
            ::-webkit-scrollbar-track {
              background: transparent;
            }
            ::-webkit-scrollbar-thumb {
              background: rgba(0, 0, 0, 0.1);
              border-radius: 4px;
            }
            ::-webkit-scrollbar-thumb:hover {
              background: rgba(0, 0, 0, 0.2);
            }
          </style>
        </head>
        <body>
          <div class="email-wrapper">
            ${htmlContent}
          </div>
          <script>
            const measure = () => {
              // Temporarily remove constraints to measure natural width
              document.body.style.width = 'max-content';
              const naturalWidth = document.body.scrollWidth;
              document.body.style.width = '';

              return {
                height: document.body.scrollHeight,
                naturalWidth,
              };
            };

            const sendDimensions = () => {
              const { height, naturalWidth } = measure();
              window.parent.postMessage({ type: 'resize', height, naturalWidth }, '*');
            };

            // Wait for full layout before measuring
            if (document.readyState === 'complete') {
              sendDimensions();
            } else {
              window.addEventListener('load', sendDimensions);
            }

            // ResizeObserver only for height changes after initial render
            let initialSent = false;
            const ro = new ResizeObserver(() => {
              if (!initialSent) {
                initialSent = true;
                return; // skip first fire, already sent above
              }
              sendDimensions();
            });
            ro.observe(document.body);

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
		setIframeReady(false)
		setIframeWidth('100%')
		setBlobUrl(url)

		return () => {
			URL.revokeObjectURL(url)
		}
	}, [htmlContent, effectiveViewMode, accentColor, allowExternalResources])

	// Listen for messages from iframe (resize, links)
	useEffect(() => {
		const handler = (e: MessageEvent) => {
			if (!iframeRef.current) return

			// Resize
			if (e.data?.type === 'resize' && typeof e.data.height === 'number') {
				iframeRef.current.style.height = `${e.data.height}px`

				if (typeof e.data.naturalWidth === 'number' && containerRef.current) {
					const containerWidth =
						containerRef.current.getBoundingClientRect().width
					const targetWidth = Math.min(e.data.naturalWidth, containerWidth)
					setIframeWidth(`${targetWidth}px`)
				}

				setIframeReady(true)
			}

			// Open link warning
			if (e.data?.type === 'open-link' && e.data.url) {
				setPendingUrl(e.data.url)
				setWarningOpen(true)
			}
		}

		window.addEventListener('message', handler)
		return () => window.removeEventListener('message', handler)
	}, [])

	const handleConfirmOpenLink = () => {
		if (pendingUrl) {
			openUrl(pendingUrl)
		}
		setWarningOpen(false)
		setPendingUrl(null)
	}

	if (effectiveViewMode === 'plain') {
		return (
			<div className='flex flex-col items-center px-6 py-4'>
				<pre className='message-view-plain max-w-full whitespace-pre-wrap break-words rounded-xl border border-white/[0.08] bg-slate-950/50 p-6 font-mono text-sm text-slate-300 shadow-xl'>
					{plainContent || '(No content)'}
				</pre>
			</div>
		)
	}

	return (
		<>
			<div
				ref={containerRef}
				className='flex flex-col items-center overflow-x-auto px-6 py-4'>
				<div
					className='overflow-hidden rounded-xl border border-white/[0.08] shadow-2xl'
					style={{
						width: iframeWidth,
						opacity: iframeReady ? 1 : 0,
						transition: 'opacity 0.15s ease',
					}}>
					<iframe
						key={blobUrl}
						ref={iframeRef}
						title='Message Content'
						src={blobUrl}
						sandbox='allow-scripts allow-popups allow-popups-to-escape-sandbox'
						className='message-view-iframe block w-full border-none'
						style={{ minHeight: iframeReady ? undefined : '0px' }}
					/>
				</div>
			</div>

			<Dialog open={warningOpen} onOpenChange={setWarningOpen}>
				<DialogContent className='sm:max-w-[425px]'>
					<DialogHeader>
						<DialogTitle>{t('security:externalLink.title')}</DialogTitle>
						<DialogDescription>
							{t('security:externalLink.description')}
						</DialogDescription>
					</DialogHeader>
					<div className='bg-slate-950/50 flex flex-col gap-1.5 rounded-lg border border-white/[0.06] p-3'>
						<p className='text-[10px] font-bold tracking-wider text-slate-500 uppercase'>
							Target URL
						</p>
						<p className='break-all text-xs font-mono text-slate-300'>
							{pendingUrl}
						</p>
					</div>
					<DialogFooter>
						<Button variant='ghost' onClick={() => setWarningOpen(false)}>
							{t('security:externalLink.cancel')}
						</Button>
						<Button
							variant='default'
							onClick={handleConfirmOpenLink}
							className='bg-sky-500 font-semibold text-white hover:bg-sky-600 dark:bg-sky-600 dark:hover:bg-sky-700'>
							{t('security:externalLink.open')}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</>
	)
}