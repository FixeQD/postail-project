import { useEffect, useRef, useState, useMemo } from 'react'
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
	const isReadyRef = useRef(false)

	const [iframeSrcDoc, setIframeSrcDoc] = useState<string | null>(null)

	const [pendingUrl, setPendingUrl] = useState<string | null>(null)
	const [warningOpen, setWarningOpen] = useState(false)

	const effectiveMode = !htmlContent || !htmlContent.trim() ? 'plain' : viewMode

	const inlineImagesHash = JSON.stringify(inline_images)

	// Memoize mapped images to prevent unnecessary re-renders
	const mappedImages = useMemo(() => {
		return inline_images
			.filter((img) => img.cid && img.cached_path)
			.map((img) => ({
				cid: img.cid!,
				cachedPath: img.cached_path!,
				mimeType: img.mime_type,
			}))
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [inlineImagesHash])

	// 1. Process HTML Content
	useEffect(() => {
		if (effectiveMode !== 'html') return

		isReadyRef.current = false
		onLoadingChange?.(true)

		const hasDarkMode =
			htmlContent.includes('prefers-color-scheme: dark') ||
			htmlContent.includes('data-ogsc') ||
			htmlContent.includes('data-ogsb')

		const iframeBg = hasDarkMode ? 'transparent' : '#ffffff'
		const iframeTextColor = hasDarkMode ? 'inherit' : '#1a1a1a'
		const colorScheme = hasDarkMode ? 'dark light' : 'light'

		// Minimalist HTML wrapper relying on ResizeObserver
		const htmlTemplate = `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline' data:; img-src data: blob:; font-src data:; connect-src 'none';">
  <style>
    * { box-sizing: border-box; }
    body {
      margin: 0; padding: 24px;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      color: ${iframeTextColor}; background: ${iframeBg};
      font-size: 14px; line-height: 1.6; word-wrap: break-word;
      color-scheme: ${colorScheme};
      overflow-x: hidden;
    }
    a { color: ${accentColor}; text-decoration: none; }
    a:hover { text-decoration: underline; }
    img, table, td, th { max-width: 100% !important; height: auto !important; }
    pre { overflow-x: auto; max-width: 100%; white-space: pre-wrap; }
    ::-webkit-scrollbar { width: 8px; height: 8px; }
    ::-webkit-scrollbar-track { background: transparent; }
    ::-webkit-scrollbar-thumb { background: rgba(0,0,0,0.1); border-radius: 4px; }
  </style>
</head>
<body>
  <div id="email-wrapper">${htmlContent}</div>
  <script>
    const post = (msg) => window.parent.postMessage(msg, '*');

    // Efficient resize observer
    let lastH = 0;
    let resizeTimer;
    const checkHeight = () => {
      const wrapper = document.getElementById('email-wrapper');
      const h = wrapper ? wrapper.scrollHeight : document.body.scrollHeight;
      if (Math.abs(h - lastH) > 2) {
        lastH = h;
        post({ type: 'resize', height: h });
      }
    };

    const ro = new ResizeObserver(() => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(checkHeight, 50);
    });
    const wrapper = document.getElementById('email-wrapper');
    if (wrapper) ro.observe(wrapper);
    else ro.observe(document.body);
    window.addEventListener('load', checkHeight);
    if (document.readyState === 'complete') checkHeight();

    // Link interception
    document.addEventListener('click', (e) => {
      const a = e.target.closest('a');
      if (a && a.href) {
        e.preventDefault();
        post({ type: 'link', url: a.href });
      }
    });
  </script>
</body>
</html>`

		invoke<{ hasExternalResources: boolean; processedHtml: string }>('set_email_view_content', {
			html: htmlTemplate,
			inlineImages: mappedImages,
			allowExternal: allowExternalResources,
		})
			.then((res) => {
				if (res.hasExternalResources) onExternalDetected?.()
				setIframeSrcDoc(res.processedHtml)
			})
			.catch(console.error)
			.finally(() => onLoadingChange?.(false))
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [htmlContent, effectiveMode, accentColor, allowExternalResources, mappedImages])

	// 2. Handle Iframe Messages
	useEffect(() => {
		const handler = (e: MessageEvent) => {
			if (e.source !== iframeRef.current?.contentWindow) return

			if (e.data?.type === 'resize' && typeof e.data.height === 'number') {
				// Bypass React state for height to prevent laggy re-renders
				if (iframeRef.current) {
					iframeRef.current.style.height = `${e.data.height}px`
				}
			}

			if (e.data?.type === 'link' && e.data.url) {
				setPendingUrl(e.data.url)
				setWarningOpen(true)
			}
		}

		window.addEventListener('message', handler)
		return () => window.removeEventListener('message', handler)
	}, [])

	// 3. Render Plain Text
	if (effectiveMode === 'plain') {
		return (
			<div className='px-5 py-5'>
				<pre className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-6 font-mono text-[13px] leading-relaxed break-words whitespace-pre-wrap text-[var(--text-primary)] shadow-sm'>
					{plainContent || '(No content)'}
				</pre>
			</div>
		)
	}

	// 4. Render HTML
	return (
		<>
			<div className='overflow-x-auto px-5 py-5'>
				<div className='bg-white'>
					{iframeSrcDoc && (
						<iframe
							ref={iframeRef}
							title='Message Content'
							srcDoc={iframeSrcDoc}
							sandbox='allow-scripts'
							className='block w-full border-none opacity-0 transition-opacity duration-500 ease-in-out'
							style={{ minHeight: '200px' }}
							onLoad={(e) => {
								;(e.target as HTMLIFrameElement).style.opacity = '1'
							}}
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
				onConfirm={() => {
					if (pendingUrl) openUrl(pendingUrl)
					setWarningOpen(false)
					setPendingUrl(null)
				}}
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
