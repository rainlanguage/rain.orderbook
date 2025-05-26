import type { Writable } from 'svelte/store';
import type { ConfigSource } from '@rainlanguage/orderbook';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';

export type ParseConfigSourceFn = (settingsContent: string) => Promise<ConfigSource>;

export interface ApplySettingsResult {
  settingsStatus: 'checking' | 'success' | 'error';
  errorMessage?: string;
}

export async function applySettings(
  settingsContent: string,
  settingsStore: Writable<ConfigSource | undefined>,
  settingsTextStore: Writable<string>,
  parseConfigSourceFn: ParseConfigSourceFn,
): Promise<ApplySettingsResult> {
  try {
    settingsTextStore.set(settingsContent);
    const parsedConfig = await parseConfigSourceFn(settingsContent);
    settingsStore.set(parsedConfig);
    return { settingsStatus: 'success' };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    reportErrorToSentry(error, SentrySeverityLevel.Info);
    return { settingsStatus: 'error', errorMessage };
  }
}
