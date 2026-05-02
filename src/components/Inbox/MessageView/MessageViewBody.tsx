import { useEffect, useRef, useState, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
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

	const containerRef = useRef<HTMLDivElement>(null)
	// Tracks whether a rAF is already scheduled to avoid stacking frames
	const rafPendingRef = useRef(false)

	const [pendingUrl, setPendingUrl] = useState<string | null>(null)
	const [warningOpen, setWarningOpen] = useState(false)

	const effectiveMode = !htmlContent || !htmlContent.trim() ? 'plain' : viewMode

	const inlineImagesHash = JSON.stringify(inline_images)

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

	// 1. Push HTML content to the Rust protocol handler
	useEffect(() => {
		if (effectiveMode !== 'html') return

		onLoadingChange?.(true)

		const hasDarkMode =
			htmlContent.includes('prefers-color-scheme: dark') ||
			htmlContent.includes('data-ogsc') ||
			htmlContent.includes('data-ogsb')

		const iframeBg = hasDarkMode ? 'transparent' : '#ffffff'
		const iframeTextColor = hasDarkMode ? 'inherit' : '#1a1a1a'
		const colorScheme = hasDarkMode ? 'dark light' : 'light'

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
    document.addEventListener('click', (e) => {
      const a = e.target.closest('a');
      if (a && a.href) {
        e.preventDefault();
        window.parent.postMessage({ type: 'link', url: a.href }, '*');
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
			})
			.catch(console.error)
			.finally(() => onLoadingChange?.(false))
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [htmlContent, effectiveMode, accentColor, allowExternalResources, mappedImages])

	// 2. Spawn / destroy the child webview for HTML mode
	useEffect(() => {
		if (effectiveMode !== 'html') return

		const win = getCurrentWindow()
		invoke('create_email_webview', { window: win }).catch(console.error)

		return () => {
			invoke('destroy_email_webview').catch(console.error)
		}
	}, [effectiveMode])

	// 3. ResizeObserver — sync native webview bounds to the placeholder div
	useEffect(() => {
		if (effectiveMode !== 'html') return
		const el = containerRef.current
		if (!el) return

		const syncBounds = () => {
			rafPendingRef.current = false
			const rect = el.getBoundingClientRect()
			const dpr = window.devicePixelRatio ?? 1

			// Tauri 2 set_position / set_size expect logical coords,
			// so we divide out the pixel ratio.
			invoke('update_email_webview_bounds', {
				x: rect.x / dpr,
				y: rect.y / dpr,
				width: rect.width / dpr,
				height: rect.height / dpr,
			}).catch(console.error)
		}

		const ro = new ResizeObserver(() => {
			if (rafPendingRef.current) return
			rafPendingRef.current = true
			requestAnimationFrame(syncBounds)
		})

		ro.observe(el)
		// Initial sync as soon as the webview is mounted
		requestAnimationFrame(syncBounds)

		return () => ro.disconnect()
	}, [effectiveMode])

	// Render Plain Text
	if (effectiveMode === 'plain') {
		return (
			<div className='px-5 py-5'>
				<pre className='w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-6 font-mono text-[13px] leading-relaxed break-words whitespace-pre-wrap text-[var(--text-primary)] shadow-sm'>
					{plainContent || '(No content)'}
				</pre>
			</div>
		)
	}

	// Render HTML — blank placeholder div that the native child webview overlays
	return (
		<>
			<div
				ref={containerRef}
				id='email-webview-container'
				className='min-h-[400px] w-full flex-1'
			/>

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
