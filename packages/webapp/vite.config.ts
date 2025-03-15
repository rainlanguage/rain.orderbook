import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';
import {loadEnv} from "vite";
import {svelteTesting} from '@testing-library/svelte/vite'

export default defineConfig(({ mode }) => ({
	assetsInclude: ['**/*.rain'],
	plugins: [
		sveltekit(),
		svelteTesting({
			// disable auto cleanup
			autoCleanup: false,
			// disable browser resolution condition
			resolveBrowser: false
		})
	],
	resolve: {
		conditions: mode === 'test' ? ['browser'] : []
	},
	define: {
		'process.env': {},
		'import.meta.vitest': 'undefined'
	},

	optimizeDeps: {
		exclude: ['@rainlanguage/orderbook/js_api']
	},
	test: {
		// Jest like globals
		includeSource: ['src/**/*.{js,ts}'],
		globals: true,
		environment: 'jsdom',
		include: ['src/**/*.{test,spec}.ts'],
		// Extend jest-dom matchers
		setupFiles: ['./test-setup.ts'],
		// load env vars
		env: loadEnv('', process.cwd(), ''),
		testTimeout: 10000,
		server: {
			deps: {
				inline: [
					/@reown\/appkit/, /@tanstack\/svelte-query/
				]
			}
		},
		deps: {
			interopDefault: true
		}
	}
}));
