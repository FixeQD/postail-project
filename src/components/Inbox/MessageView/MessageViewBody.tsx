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

	// Memoize mapped images to prevent unnecessary re-renders
	const mappedImages = useMemo(() => {
		return inline_images
			.filter((img) => img.cid && img.cached_path)
			.map((img) => ({
				cid: img.cid!,
				cachedPath: img.cached_path!,
				mimeType: img.mime_type,
			}))
	}, [inline_images])

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

		// Minimalist, fast HTML wrapper
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
  ${htmlContent}
  <script>
    const post = (msg) => window.parent.postMessage(msg, '*');

    // Efficient resize observer
    let lastH = 0;
    let resizeTimer;
    const checkHeight = () => {
      const h = document.documentElement.scrollHeight || document.body.scrollHeight;
      if (Math.abs(h - lastH) > 2) {
        lastH = h;
        post({ type: 'resize', height: h });
      }
    };

    const ro = new ResizeObserver(() => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(checkHeight, 50);
    });
    ro.observe(document.documentElement);
    ro.observe(document.body);
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
	}, [
		htmlContent,
		effectiveMode,
		accentColor,
		allowExternalResources,
		mappedImages,
		onExternalDetected,
		onLoadingChange,
	])

	// 2. Handle Iframe Messages
	useEffect(() => {
		const handler = (e: MessageEvent) => {
			if (e.source !== iframeRef.current?.contentWindow) return

			if (e.data?.type === 'resize' && typeof e.data.height === 'number') {
				// Bypass React state for height to prevent laggy re-renders
				if (iframeRef.current) {
					iframeRef.current.style.height = `${e.data.height}px`
				}

				if (!isReadyRef.current) {
					isReadyRef.current = true
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
				<div className='rounded-xl border border-[var(--border-faint)] bg-white opacity-100 shadow-sm transition-opacity duration-200'>
					{iframeSrcDoc && (
						<iframe
							ref={iframeRef}
							title='Message Content'
							srcDoc={iframeSrcDoc}
							sandbox='allow-scripts'
							scrolling='no'
							className='block w-full border-none'
							style={{ minHeight: '200px' }}
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
