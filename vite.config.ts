import { defineConfig, type UserConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import monacoEditorPluginModule from 'vite-plugin-monaco-editor'
import path from 'path'

// fix for ESM/CommonJS mismatch in some environments
const monacoEditorPlugin = (monacoEditorPluginModule as any).default || monacoEditorPluginModule

const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig((): UserConfig => ({
	plugins: [
		react(),
		tailwindcss(),
		monacoEditorPlugin({
			languages: ['html', 'css'],
		}),
	],

	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent Vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: 'ws',
					host,
					port: 1421,
				}
			: undefined,
		watch: {
			// 3. tell Vite to ignore watching `src-tauri`
			ignored: ['**/src-tauri/**'],
		},
	},
	resolve: {
		alias: {
			'@': path.resolve(__dirname, './src'),
		},
	},
	build: {
		minify: 'terser',
		terserOptions: {
			compress: {
				drop_console: true,
				drop_debugger: true,
			},
		},
		chunkSizeWarningLimit: 1000,
		rollupOptions: {
			output: {
				manualChunks: {
					monaco: ['monaco-editor/esm/vs/editor/editor.api', 'monaco-editor/esm/vs/language/html/monaco.contribution', 'monaco-editor/esm/vs/language/css/monaco.contribution'],
					lexical: ['lexical', '@lexical/html'],
					vendor: ['react', 'react-dom', 'framer-motion', 'lucide-react'],
				},
			},
		},
	},
}))
