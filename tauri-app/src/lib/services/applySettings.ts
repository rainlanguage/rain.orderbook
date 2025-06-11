import type { Writable } from 'svelte/store';
import type { Config } from '@rainlanguage/orderbook';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';

export type ParseConfigFn = (text: string[]) => Promise<Config>;

export interface ApplySettingsResult {
  settingsStatus: 'checking' | 'success' | 'error';
  errorMessage?: string;
}

export async function applySettings(
  settingsContent: string,
  settingsStore: Writable<Config>,
  settingsTextStore: Writable<string>,
  ParseConfigFn: ParseConfigFn,
): Promise<ApplySettingsResult> {
  try {
    settingsTextStore.set(settingsContent);
    const parsedConfig = await ParseConfigFn([settingsContent]);
    settingsStore.set(parsedConfig);
    return { settingsStatus: 'success' };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    reportErrorToSentry(error, SentrySeverityLevel.Info);
    return { settingsStatus: 'error', errorMessage };
  }
}
