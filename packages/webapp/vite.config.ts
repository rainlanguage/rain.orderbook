import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';
import { loadEnv } from 'vite';

export default defineConfig(({ mode }) => ({
	assetsInclude: ['**/*.rain'],
	plugins: [sveltekit()],
	resolve: {
		conditions: mode === 'test' ? ['browser'] : []
	},
	define: {
		'process.env': {},
		'import.meta.vitest': 'undefined'
	},
	build: {
		target: 'es2023'
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
				inline: [/@reown\/appkit/, /@tanstack\/svelte-query/, /@rainlanguage\/orderbook/]
			}
		},
		deps: {
			interopDefault: true
		}
	}
}));
