import { defineConfig, type UserConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

const host = process.env.TAURI_DEV_HOST

export default defineConfig(
	(): UserConfig => ({
		plugins: [react(), tailwindcss()],
		esbuild: {
			drop: ['console', 'debugger'],
			legalComments: 'none',
		},

		clearScreen: false,
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
				ignored: ['**/src-tauri/**'],
			},
		},
		resolve: {
			alias: {
				'@': path.resolve(__dirname, './src'),
			},
		},
		build: {
			minify: 'esbuild',
			chunkSizeWarningLimit: 4000,
			rollupOptions: {
				treeshake: {
					moduleSideEffects: false,
					propertyReadSideEffects: false,
					tryCatchDeoptimization: false,
				},
				output: {
					manualChunks: {
						monaco: [
							'monaco-editor/esm/vs/editor/editor.api',
							'monaco-editor/esm/vs/language/html/monaco.contribution',
							'monaco-editor/esm/vs/language/css/monaco.contribution',
						],
						lexical: ['lexical', '@lexical/html'],
						icons: ['lucide-react'],
						vendor: ['react', 'react-dom', 'framer-motion'],
					},
				},
			},
		},
	})
)
