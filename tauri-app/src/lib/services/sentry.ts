import * as Sentry from '@sentry/sveltekit';
import { enableSentry } from '$lib/stores/settings';
import { get } from "svelte/store";
import type { HandleClientError, HandleServerError } from '@sveltejs/kit';
import { handleErrorWithSentry } from '@sentry/sveltekit';

export async function applySentryEnable() {
  const $enableSentry = get(enableSentry);

  const sentryOptions = Sentry.getClient()?.getOptions();
  if(!sentryOptions) return;

  sentryOptions.enabled = $enableSentry;
}

export function handleErrorWithSentryIfEnabled<T extends HandleClientError | HandleServerError>(handleError: T) {
  const $enableSentry = get(enableSentry);

  if($enableSentry) {
    return handleErrorWithSentry(handleError);
  }
}
