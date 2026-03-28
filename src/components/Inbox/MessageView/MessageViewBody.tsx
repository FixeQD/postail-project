import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useThemeStore } from '@/stores/themeStore'
import { useTypedTranslation } from '@/hooks/useTypedTranslation'
import { ConfirmationDialog } from '@/components/ui/custom/ConfirmationDialog'
import { openUrl } from '@tauri-apps/plugin-opener'

import type { MessageViewBodyProps } from '@/types/components/shared'

export const MessageViewBody = ({
	htmlContent,
	plainContent,
	viewMode,
	allowExternalResources = false,
	inline_images = [],
	onExternalDetected,
	onLoadingChange,
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
    <div class="email-wrapper">${htmlContent}</div>
    <script>
      const sendHeight = () => {
        window.parent.postMessage({
          type: 'resize',
          height: document.body.scrollHeight,
        }, '*');
      };

      const ro = new ResizeObserver(() => sendHeight());

      const init = () => {
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
		onLoadingChange?.(true)
		invoke<{ hasExternalResources: boolean; failedResources: string[] }>(
			'set_email_view_content',
			{
				html,
				inlineImages: inline_images
					.filter((img) => img.cid && img.cached_path)
					.map((img) => ({
						cid: img.cid!,
						cachedPath: img.cached_path!,
						mimeType: img.mime_type,
					})),
				allowExternal: allowExternalResources,
			}
		)
			.then((result) => {
				if (result.hasExternalResources) {
					onExternalDetected?.()
				}
				setIframeSrc(src)
				onLoadingChange?.(false)
			})
			.catch(() => {
				onLoadingChange?.(false)
			})
	}, [htmlContent, effectiveViewMode, accentColor, allowExternalResources, inline_images])

	useEffect(() => {
		const handler = (e: MessageEvent) => {
			const validOrigins = ['http://postail.localhost', 'postail://localhost', 'null']
			if (!validOrigins.includes(e.origin) || e.source !== iframeRef.current?.contentWindow) {
				return
			}

			if (e.data?.type === 'resize' && typeof e.data.height === 'number') {
				iframeRef.current.style.height = `${e.data.height}px`
				setIframeWidth('100%')
				setIframeReady(true)
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
			<div className='animate-in fade-in slide-in-from-bottom-2 px-5 py-5 transition-all duration-300'>
				<pre className='message-view-plain w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-6 font-mono text-[13px] leading-relaxed break-words whitespace-pre-wrap text-[var(--text-primary)] shadow-sm'>
					{plainContent || '(No content)'}
				</pre>
			</div>
		)
	}

	return (
		<>
			<div
				ref={containerRef}
				className='overflow-x-auto px-5 py-5'
				style={{ contain: 'content' }}>
				<div
					className={`overflow-hidden rounded-xl border border-[var(--border-faint)] transition-all duration-300 ease-out ${
						iframeReady
							? 'translate-y-0 opacity-100 shadow-sm'
							: 'translate-y-2 opacity-0'
					}`}
					style={{
						width: iframeWidth,
						backgroundColor: emailBg,
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

			<ConfirmationDialog
				open={warningOpen}
				onOpenChange={setWarningOpen}
				title={t('security:externalLink.title')}
				description={t('security:externalLink.description')}
				cancelLabel={t('security:externalLink.cancel')}
				confirmLabel={t('security:externalLink.open')}
				onConfirm={handleConfirmOpenLink}
				confirmClassName='w-full border-0 font-semibold shadow-lg bg-sky-500 text-white hover:bg-sky-600'>
				<div className='flex flex-col gap-1.5 rounded-lg border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-3'>
					<p className='text-[10px] font-bold tracking-wider text-[var(--text-tertiary)] uppercase'>
						Target URL
					</p>
					<p className='font-mono text-xs break-all text-[var(--text-primary)]'>
						{pendingUrl}
					</p>
				</div>
			</ConfirmationDialog>
		</>
	)
}
