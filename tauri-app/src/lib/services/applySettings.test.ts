import { describe, it, expect, vi, beforeEach } from 'vitest';
import { writable, type Writable } from 'svelte/store';
import type { ConfigSource } from '@rainlanguage/orderbook';
import { applySettings, type ParseConfigSourceFn } from './applySettings';
import { SentrySeverityLevel, reportErrorToSentry } from './sentry';

vi.mock('$lib/services/sentry', () => ({
  reportErrorToSentry: vi.fn(),
  SentrySeverityLevel: { Info: 'info', Error: 'error' },
}));

describe('applySettings service', () => {
  let mockSettingsStore: Writable<ConfigSource | undefined>;
  let mockSettingsTextStore: Writable<string>;
  let mockParseConfigSourceFn: ParseConfigSourceFn;

  beforeEach(() => {
    vi.resetAllMocks();
    mockSettingsStore = writable<ConfigSource | undefined>(undefined);
    mockSettingsTextStore = writable<string>('');
    vi.spyOn(mockSettingsStore, 'set');
    vi.spyOn(mockSettingsTextStore, 'set');
  });

  it('should successfully apply settings and update stores', async () => {
    const settingsContent = '{ "networks": {} }';
    const parsedConfig: ConfigSource = {
      'spec-version': '1',
      networks: { mainnet: { 'chain-id': 1, rpc: 'rpc' } },
    };
    mockParseConfigSourceFn = vi.fn().mockResolvedValue(parsedConfig);

    const result = await applySettings(
      settingsContent,
      mockSettingsStore,
      mockSettingsTextStore,
      mockParseConfigSourceFn,
    );

    expect(result.settingsStatus).toBe('success');
    expect(result.errorMessage).toBeUndefined();
    expect(mockSettingsTextStore.set).toHaveBeenCalledWith(settingsContent);
    expect(mockSettingsStore.set).toHaveBeenCalledWith(parsedConfig);
    expect(mockParseConfigSourceFn).toHaveBeenCalledWith(settingsContent);
    expect(vi.mocked(reportErrorToSentry)).not.toHaveBeenCalled();
  });

  it('should return error status if parseConfigSourceFn throws an error', async () => {
    const settingsContent = 'invalid json';
    const parseError = new Error('Failed to parse');
    mockParseConfigSourceFn = vi.fn().mockRejectedValue(parseError);

    const result = await applySettings(
      settingsContent,
      mockSettingsStore,
      mockSettingsTextStore,
      mockParseConfigSourceFn,
    );

    expect(result.settingsStatus).toBe('error');
    expect(result.errorMessage).toBe('Failed to parse');
    expect(mockSettingsTextStore.set).toHaveBeenCalledWith(settingsContent);
    expect(mockSettingsStore.set).not.toHaveBeenCalled();
    expect(mockParseConfigSourceFn).toHaveBeenCalledWith(settingsContent);
    expect(vi.mocked(reportErrorToSentry)).toHaveBeenCalledWith(
      parseError,
      SentrySeverityLevel.Info,
    );
  });

  it('should handle non-Error objects thrown by parseConfigSourceFn', async () => {
    const settingsContent = 'another invalid input';
    const parseErrorString = 'Custom error string';
    mockParseConfigSourceFn = vi.fn().mockRejectedValue(parseErrorString);

    const result = await applySettings(
      settingsContent,
      mockSettingsStore,
      mockSettingsTextStore,
      mockParseConfigSourceFn,
    );

    expect(result.settingsStatus).toBe('error');
    expect(result.errorMessage).toBe(parseErrorString);
    expect(vi.mocked(reportErrorToSentry)).toHaveBeenCalledWith(
      parseErrorString,
      SentrySeverityLevel.Info,
    );
  });
});
