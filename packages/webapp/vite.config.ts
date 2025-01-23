import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';
import {loadEnv} from "vite";

export default defineConfig({
	plugins: [sveltekit()],

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
				inline: [/@tanstack\/svelte-query/]
			}
		}
	}
});
