import { handleErrorWithSentryIfEnabled, initSentry } from '$lib/services/sentry';

// Initialize sentry, disabled by default
initSentry();

// If sentry is enabled, publish errors to it
export const handleError = handleErrorWithSentryIfEnabled;
