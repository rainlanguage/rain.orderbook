import * as Sentry from '@sentry/sveltekit';
import { handleErrorWithSentryIfEnabled } from "$lib/services/sentry";

// Initialize sentry, disabled by default
Sentry.init({
  dsn: import.meta.env.VITE_SENTRY_DSN,
  tracesSampleRate: 1.0,
  replaysSessionSampleRate: 0.1,
  replaysOnErrorSampleRate: 1.0,
  enabled: false,
  integrations: [Sentry.replayIntegration()],
});


// If sentry is enabled, publish errors to it
export const handleError = handleErrorWithSentryIfEnabled;