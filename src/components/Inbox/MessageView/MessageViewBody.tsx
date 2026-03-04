import { useEffect, useRef, useState } from 'react'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
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

import type { MessageViewBodyProps } from '@/types/components/shared'

export const MessageViewBody = ({
	htmlContent,
	plainContent,
	viewMode,
	allowExternalResources = false,
	inline_images = [],
	onCspBlocked,
}: MessageViewBodyProps) => {
	const { t } = useTypedTranslation(['security', 'common'])
	const accentColor = useThemeStore((s) => s.accentColor)
	const iframeRef = useRef<HTMLIFrameElement>(null)
	const containerRef = useRef<HTMLDivElement>(null)
	const [iframeSrc, setIframeSrc] = useState<string | null>(null)
	const [warningOpen, setWarningOpen] = useState(false)
	const [pendingUrl, setPendingUrl] = useState<string | null>(null)
	const [iframeWidth, setIframeWidth] = useState<string>('100%')
	const [iframeReady, setIframeReady] = useState(false)
	const [emailBg, setEmailBg] = useState<string>('#ffffff')

	const effectiveViewMode = !htmlContent || !htmlContent.trim() ? 'plain' : viewMode

	useEffect(() => {
		if (effectiveViewMode !== 'html') return

		const hasDarkModeSupport =
			htmlContent.includes('prefers-color-scheme: dark') ||
			htmlContent.includes('data-ogsc') ||
			htmlContent.includes('data-ogsb')

		const iframeColorScheme = hasDarkModeSupport ? 'dark light' : 'light'
		const iframeBg = hasDarkModeSupport ? 'transparent' : '#ffffff'
		const iframeTextColor = hasDarkModeSupport ? 'inherit' : '#1a1a1a'

		let processedHtml = htmlContent
		for (const img of inline_images) {
			if (img.cid && img.cached_path) {
				const rawCid = img.cid.replace(/[<>]/g, '')
				const localUrl = convertFileSrc(img.cached_path)
				const cidRegex = new RegExp(
					`cid:${rawCid.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`,
					'gi'
				)
				processedHtml = processedHtml.replace(cidRegex, localUrl)
			}
		}

		const html = `<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <style>
      * { box-sizing: border-box; }
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
      a { color: ${accentColor}; text-decoration: none; }
      a:hover { text-decoration: underline; }
      img { max-width: 100% !important; }
      img[width] { height: auto !important; }
      table, td, th { max-width: 100% !important; }
      table[width], td[width] { width: auto !important; }
      table[width="100%"], td[width="100%"] { width: 100% !important; }
      pre { overflow-x: auto; max-width: 100%; }
      ::-webkit-scrollbar { width: 8px; height: 8px; }
      ::-webkit-scrollbar-track { background: transparent; }
      ::-webkit-scrollbar-thumb { background: rgba(0,0,0,0.1); border-radius: 4px; }
      ::-webkit-scrollbar-thumb:hover { background: rgba(0,0,0,0.2); }
    </style>
  </head>
  <body>
    <div class="email-wrapper">${processedHtml}</div>
    <script>
      let naturalWidth = 0;

      const sendHeight = () => {
        window.parent.postMessage({
          type: 'resize',
          height: document.body.scrollHeight,
          naturalWidth,
        }, '*');
      };

      const ro = new ResizeObserver(() => sendHeight());

      const init = () => {
        ro.disconnect();
        document.body.style.width = 'max-content';
        naturalWidth = document.body.scrollWidth;
        document.body.style.width = '';
        ro.observe(document.body);
        sendHeight();
      };

      if (document.readyState === 'complete') {
        init();
      } else {
        window.addEventListener('load', init);
      }


      window.open = function(url) {
        if (url) window.parent.postMessage({ type: 'open-link', url: String(url) }, '*');
        return null;
      };

      if (window.navigation) {
        // Navigation API catches everything: clicks, JS navigation, meta refresh
        window.navigation.addEventListener('navigate', (e) => {
          const dest = e.destination.url;
          if (dest === location.href) return;
          e.preventDefault();
          window.parent.postMessage({ type: 'open-link', url: dest }, '*');
        });
      } else {
        // Fallback for older WebViews
        document.addEventListener('click', function(e) {
          const a = e.target.closest('a');
          if (a && a.href) {
            e.preventDefault();
            window.parent.postMessage({ type: 'open-link', url: a.href }, '*');
          }
        });
      }

      document.addEventListener('securitypolicyviolation', function(e) {
        if (e.blockedURI && e.blockedURI !== 'inline' && e.blockedURI !== 'eval') {
          window.parent.postMessage({ type: 'csp-blocked' }, '*');
        }
      });
    </script>
  </body>
</html>`

		const bodyBgMatch =
			htmlContent.match(/body[^{]*\{[^}]*background-color:\s*(#[0-9a-fA-F]{3,8})/)?.[1] ??
			htmlContent.match(
				/body[^>]*style="[^"]*background-color:\s*(#[0-9a-fA-F]{3,8})/
			)?.[1] ??
			'#ffffff'

		setEmailBg(bodyBgMatch)
		setIframeReady(false)
		setIframeWidth('100%')
		setIframeSrc(null)

		const isWindows = navigator.userAgent.includes('Windows')
		const baseUrl = isWindows ? 'http://postail.localhost' : 'postail://localhost'

		const src = `${baseUrl}/message/current?v=${Date.now()}`
		invoke('set_email_view_content', {
			html,
			allowExternal: allowExternalResources,
		}).then(() => {
			setIframeSrc(src)
		})
	}, [htmlContent, effectiveViewMode, accentColor, allowExternalResources, inline_images])

	useEffect(() => {
		const handler = (e: MessageEvent) => {
			if (!iframeRef.current) return

			if (e.data?.type === 'resize' && typeof e.data.height === 'number') {
				iframeRef.current.style.height = `${e.data.height}px`

				if (typeof e.data.naturalWidth === 'number' && containerRef.current) {
					const containerWidth = containerRef.current.getBoundingClientRect().width
					const targetWidth = Math.min(e.data.naturalWidth, containerWidth)
					setIframeWidth(`${targetWidth}px`)
				}

				setIframeReady(true)
			}

			if (e.data?.type === 'csp-blocked') {
				onCspBlocked?.()
			}

			if (e.data?.type === 'open-link' && e.data.url) {
				setPendingUrl(e.data.url)
				setWarningOpen(true)
			}
		}

		window.addEventListener('message', handler)
		return () => window.removeEventListener('message', handler)
	}, [])

	const handleConfirmOpenLink = () => {
		if (pendingUrl) openUrl(pendingUrl)
		setWarningOpen(false)
		setPendingUrl(null)
	}

	if (effectiveViewMode === 'plain') {
		return (
			<div className='px-5 py-5'>
				<pre className='message-view-plain w-full rounded-xl border border-white/[0.06] bg-slate-950/60 p-5 font-mono text-[13px] leading-relaxed break-words whitespace-pre-wrap text-slate-300'>
					{plainContent || '(No content)'}
				</pre>
			</div>
		)
	}

	return (
		<>
			<div ref={containerRef} className='overflow-x-auto px-5 py-5'>
				<div
					className='overflow-hidden rounded-xl border border-white/[0.06]'
					style={{
						width: iframeWidth,
						backgroundColor: emailBg,
						opacity: iframeReady ? 1 : 0,
						transition: 'opacity 0.2s ease',
					}}>
					{iframeSrc && (
						<iframe
							key={iframeSrc}
							ref={iframeRef}
							title='Message Content'
							src={iframeSrc}
							sandbox='allow-scripts'
							className='message-view-iframe block w-full border-none'
							style={{ minHeight: iframeReady ? undefined : '0px' }}
						/>
					)}
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
					<div className='flex flex-col gap-1.5 rounded-lg border border-white/[0.06] bg-slate-950/50 p-3'>
						<p className='text-[10px] font-bold tracking-wider text-slate-500 uppercase'>
							Target URL
						</p>
						<p className='font-mono text-xs break-all text-slate-300'>{pendingUrl}</p>
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
