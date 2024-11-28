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
      includeSource: ['src/**/*.{js,ts}'],
      environment: 'jsdom',
      setupFiles: ['./vitest-setup.ts'],
      server: {
        deps: {
          inline: [/@sveltejs\/kit/, /@tanstack\/svelte-query/, /codemirror-rainlang/],
        }
      },
    },

    define: {
      'import.meta.vitest': 'undefined',
    },
  };
});
