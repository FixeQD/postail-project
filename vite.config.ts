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
      host: "127.0.0.1",
			port: 1420,
			strictPort: true,
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
			modulePreload: { polyfill: false },
			rollupOptions: {
				treeshake: {
					moduleSideEffects: false,
					propertyReadSideEffects: false,
					tryCatchDeoptimization: false,
				},
				output: {
					manualChunks: (id) => {
						if (id.includes('monaco-editor') || id.includes('@monaco-editor')) {
							return 'monaco'
						}
						if (id.includes('js-beautify')) {
							return 'beautify'
						}
						if (id.includes('lucide-react')) {
							return 'icons'
						}
						if (
							id.includes('node_modules/react/') ||
							id.includes('node_modules/react-dom/')
						) {
							return 'react'
						}
						if (id.includes('framer-motion') || id.includes('/motion/')) {
							return 'motion'
						}
					},
				},
			},
		},
	})
)
