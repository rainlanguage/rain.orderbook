import * as Sentry from '@sentry/sveltekit';
import { enableSentry } from '$lib/stores/settings';
import { get } from "svelte/store";
import type { HandleClientError, HandleServerError } from '@sveltejs/kit';
import { handleErrorWithSentry } from '@sentry/sveltekit';

export function initSentry() {
  Sentry.init({
    dsn: import.meta.env.VITE_SENTRY_DSN,
    environment: import.meta.env.VITE_SENTRY_ENVIRONMENT,
    tracesSampleRate: 1.0,
    replaysSessionSampleRate: 0.1,
    replaysOnErrorSampleRate: 1.0,
    enabled: true,
    integrations: [Sentry.replayIntegration()],
    release: import.meta.env.VITE_SENTRY_RELEASE,

    // This is a recommended way to conditionally disable/enable Sentry at runtime
    // See https://github.com/getsentry/sentry-javascript/issues/2039#issuecomment-487490204
    beforeSend(event) {
      const $enableSentry = get(enableSentry);
      return $enableSentry ? event : null;
    },
  });
}

export function handleErrorWithSentryIfEnabled<T extends HandleClientError | HandleServerError>(handleError: T) {
  const $enableSentry = get(enableSentry);

  if($enableSentry) {
    return handleErrorWithSentry(handleError);
  }
}
