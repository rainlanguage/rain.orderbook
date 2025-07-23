import * as Sentry from '@sentry/sveltekit';
import type { HandleClientError, HandleServerError } from '@sveltejs/kit';
import { handleErrorWithSentry } from '@sentry/sveltekit';
import { arch, platform, type, version } from '@tauri-apps/api/os';
import { getTauriVersion } from '@tauri-apps/api/app';
import { isSentryEnabled } from '$lib/utils/raindexClient/isSentryEnabled';

// Copy of Sentry.SeverityLevel allowed string values as an enum (to avoid spreading magic strings)
export enum SentrySeverityLevel {
  Fatal = 'fatal',
  Error = 'error',
  Warning = 'warning',
  Log = 'log',
  Info = 'info',
  Debug = 'debug',
}

export async function initSentry() {
  if (import.meta.env.VITE_SENTRY_FORCE_DISABLED === 'true') return;

  const enableSentry = isSentryEnabled();

  // Include system data in sentry issues, as both tags and context (for easy view & search)
  const [Arch, OsType, Platform, OsVersion, TauriVersion] = await Promise.all([
    arch(),
    type(),
    platform(),
    version(),
    getTauriVersion(),
  ]);
  const context = {
    Arch,
    OsType,
    Platform,
    OsVersion,
    TauriVersion,
  };
  Sentry.setTag('Arch', context.Arch);
  Sentry.setTag('OsType', context.OsType);
  Sentry.setTag('Platform', context.Platform);
  Sentry.setTag('OsVersion', context.OsVersion);
  Sentry.setTag('TauriVersion', context.TauriVersion);
  Sentry.setContext('System Context', context);

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
      return enableSentry ? event : null;
    },
  });
}

export function handleErrorWithSentryIfEnabled<T extends HandleClientError | HandleServerError>(
  handleError: T,
) {
  const enableSentry = isSentryEnabled();
  if (enableSentry) {
    return handleErrorWithSentry(handleError);
  }
}

export function reportErrorToSentry(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  e: any,
  level: SentrySeverityLevel = SentrySeverityLevel.Error,
) {
  const enableSentry = isSentryEnabled();
  if (enableSentry) {
    Sentry.withScope(function (scope) {
      scope.setLevel(level);
      Sentry.captureException(e);
    });
  }
}
