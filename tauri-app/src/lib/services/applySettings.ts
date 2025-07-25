import type { Writable } from 'svelte/store';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';

export type ParseConfigSourceFn = (settingsContent: string) => Promise<void>;

export interface ApplySettingsResult {
  settingsStatus: 'checking' | 'success' | 'error';
  errorMessage?: string;
}

export async function applySettings(
  settingsContent: string,
  settingsTextStore: Writable<string>,
  parseConfigSourceFn: ParseConfigSourceFn,
): Promise<ApplySettingsResult> {
  try {
    await parseConfigSourceFn(settingsContent);
    settingsTextStore.set(settingsContent);
    return { settingsStatus: 'success' };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    reportErrorToSentry(error, SentrySeverityLevel.Info);
    return { settingsStatus: 'error', errorMessage };
  }
}
