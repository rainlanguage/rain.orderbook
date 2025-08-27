import type { Writable } from 'svelte/store';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';
import { checkSettingsErrors } from './config';

export type ParseConfigSourceFn = (settingsContent: string) => Promise<void>;

export interface ApplySettingsResult {
  settingsStatus: 'checking' | 'success' | 'error';
  errorMessage?: string;
}

export async function applySettings(
  settingsContent: string,
  settingsTextStore: Writable<string>,
): Promise<ApplySettingsResult> {
  try {
    await checkSettingsErrors([settingsContent]);
    settingsTextStore.set(settingsContent);
    return { settingsStatus: 'success' };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    reportErrorToSentry(error, SentrySeverityLevel.Info);
    return { settingsStatus: 'error', errorMessage };
  }
}
