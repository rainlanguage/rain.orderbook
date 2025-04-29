/// <reference types="vitest" />
import { sentrySvelteKit } from '@sentry/sveltekit';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';
import checker from 'vite-plugin-checker';
import { svelteTesting } from '@testing-library/svelte/vite';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');

  const sentryPlugins =
    env.VITE_SENTRY_ENVIRONMENT === 'release' && env.SENTRY_SOURCE_MAPS_ENABLED === 'true'
      ? [
          sentrySvelteKit({
            sourceMapsUploadOptions: {
              org: env.SENTRY_ORG,
              project: env.SENTRY_PROJECT,
              authToken: env.SENTRY_AUTH_TOKEN,
            },
          }),
        ]
      : [];

  return {
    plugins: [
      ...sentryPlugins,
      sveltekit(),
      svelteTesting(),
      checker({
        typescript: true,
        eslint: {
          lintCommand: 'eslint src',
        },
      }),
    ],
    resolve: {
		conditions: mode === 'test' ? ['browser'] : []
	  },

    // prevent vite from obscuring rust errors
    clearScreen: false,

    // tauri expects a fixed port, fail if that port is not available
    server: {
      port: 1420,
      strictPort: true,
      watch: {
        // tell vite to ignore watching `src-tauri`
        ignored: ['**/src-tauri/**'],
      },
    },

    build: {
      sourcemap: true,
    },

   test: {
		// Jest like globals
		includeSource: ['src/**/*.{js,ts}'],
		globals: true,
		environment: 'jsdom',
		include: ['src/**/*.{test,spec}.ts'],
		// Extend jest-dom matchers
		setupFiles: ['./vitest-setup.ts'],
		// load env vars
		env: loadEnv('', process.cwd(), ''),
		testTimeout: 10000,
		server: {
			deps: {
				inline: [/@tanstack\/svelte-query/]
			}
		}
	  },

    define: {
      'import.meta.vitest': 'undefined',
    },
  };
});
