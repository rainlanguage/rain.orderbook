import { sentrySvelteKit } from "@sentry/sveltekit";
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';
import checker from 'vite-plugin-checker';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  return {
    // vite config
    define: {
      plugins: [
        sentrySvelteKit({
          sourceMapsUploadOptions: {
            org: env.SENTRY_ORG,
            project: env.SENTRY_PROJECT,
          }
        }),
        sveltekit(),
        checker({
        typescript: true,
        eslint: {
            lintCommand: 'eslint src',
          },
      })],


      // prevent vite from obscuring rust errors
      clearScreen: false,

      // tauri expects a fixed port, fail if that port is not available
      server: {
        port: 1420,
        strictPort: true,
        watch: {
          // tell vite to ignore watching `src-tauri`
          ignored: ["**/src-tauri/**"],
        },
      },
    },
  }
})