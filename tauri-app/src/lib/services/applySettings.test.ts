import { describe, it, expect, vi, beforeEach } from 'vitest';
import { writable, type Writable } from 'svelte/store';
import type { Config } from '@rainlanguage/orderbook';
import { applySettings, type ParseConfigFn } from './applySettings';
import { SentrySeverityLevel, reportErrorToSentry } from './sentry';

vi.mock('$lib/services/sentry', () => ({
  reportErrorToSentry: vi.fn(),
  SentrySeverityLevel: { Info: 'info', Error: 'error' },
}));

describe('applySettings service', () => {
  let mockSettingsStore: Writable<Config>;
  let mockSettingsTextStore: Writable<string>;
  let mockParseConfigFn: ParseConfigFn;

  beforeEach(() => {
    vi.resetAllMocks();
    mockSettingsStore = writable<Config>({} as Config);
    mockSettingsTextStore = writable<string>('');
    vi.spyOn(mockSettingsStore, 'set');
    vi.spyOn(mockSettingsTextStore, 'set');
  });

  it('should successfully apply settings and update stores', async () => {
    const settingsContent = `
version: 1
networks:
  mainnet:
    key: mainnet
    chainId: 1
    rpc: rpc
`;
    const parsedConfig: Config = {
      orderbook: {
        version: '1',
        networks: { mainnet: { key: 'mainnet', chainId: 1, rpc: 'rpc' } },
      },
    } as unknown as Config;
    mockParseConfigFn = vi.fn().mockResolvedValue(parsedConfig);

    const result = await applySettings(
      settingsContent,
      mockSettingsStore,
      mockSettingsTextStore,
      mockParseConfigFn,
    );

    expect(result.settingsStatus).toBe('success');
    expect(result.errorMessage).toBeUndefined();
    expect(mockSettingsTextStore.set).toHaveBeenCalledWith(settingsContent);
    expect(mockSettingsStore.set).toHaveBeenCalledWith(parsedConfig);
    expect(mockParseConfigFn).toHaveBeenCalledWith([settingsContent]);
    expect(vi.mocked(reportErrorToSentry)).not.toHaveBeenCalled();
  });

  it('should return error status if ParseConfigFn throws an error', async () => {
    const settingsContent = 'invalid json';
    const parseError = new Error('Failed to parse');
    mockParseConfigFn = vi.fn().mockRejectedValue(parseError);

    const result = await applySettings(
      settingsContent,
      mockSettingsStore,
      mockSettingsTextStore,
      mockParseConfigFn,
    );

    expect(result.settingsStatus).toBe('error');
    expect(result.errorMessage).toBe('Failed to parse');
    expect(mockSettingsTextStore.set).toHaveBeenCalledWith(settingsContent);
    expect(mockSettingsStore.set).not.toHaveBeenCalled();
    expect(mockParseConfigFn).toHaveBeenCalledWith([settingsContent]);
    expect(vi.mocked(reportErrorToSentry)).toHaveBeenCalledWith(
      parseError,
      SentrySeverityLevel.Info,
    );
  });

  it('should handle non-Error objects thrown by ParseConfigFn', async () => {
    const settingsContent = 'another invalid input';
    const parseErrorString = 'Custom error string';
    mockParseConfigFn = vi.fn().mockRejectedValue(parseErrorString);

    const result = await applySettings(
      settingsContent,
      mockSettingsStore,
      mockSettingsTextStore,
      mockParseConfigFn,
    );

    expect(result.settingsStatus).toBe('error');
    expect(result.errorMessage).toBe(parseErrorString);
    expect(vi.mocked(reportErrorToSentry)).toHaveBeenCalledWith(
      parseErrorString,
      SentrySeverityLevel.Info,
    );
  });
});
