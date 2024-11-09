import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';
import { loadEnv } from 'vite';
import path from 'path';

export default defineConfig(({ mode }) => ({
	plugins: [sveltekit()],
	resolve: {
		alias: {
			'@rainlanguage/ui-components': path.resolve(__dirname, '../ui-components/dist')
		},
		conditions: mode === 'test' ? ['browser'] : []
	},
	define: {
		'process.env': {},
		'import.meta.vitest': 'undefined'
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
		testTimeout: 10000
	}
}));
