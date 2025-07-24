import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { writable, type Writable } from 'svelte/store';
import { applySettings } from './applySettings';
import { SentrySeverityLevel, reportErrorToSentry } from './sentry';
import { checkSettingsErrors } from './config';

vi.mock('$lib/services/sentry', () => ({
  reportErrorToSentry: vi.fn(),
  SentrySeverityLevel: { Info: 'info', Error: 'error' },
}));

vi.mock('$lib/services/config', () => ({
  checkSettingsErrors: vi.fn(),
}));

describe('applySettings service', () => {
  let mockSettingsTextStore: Writable<string>;

  beforeEach(() => {
    vi.resetAllMocks();
    mockSettingsTextStore = writable<string>('');
    vi.spyOn(mockSettingsTextStore, 'set');
  });

  it('should successfully apply settings and update stores', async () => {
    const settingsContent = '{ "networks": {} }';
    (checkSettingsErrors as Mock).mockResolvedValue(undefined);

    const result = await applySettings(settingsContent, mockSettingsTextStore);

    expect(result.settingsStatus).toBe('success');
    expect(result.errorMessage).toBeUndefined();
    expect(mockSettingsTextStore.set).toHaveBeenCalledWith(settingsContent);
    expect(checkSettingsErrors).toHaveBeenCalledWith([settingsContent]);
    expect(vi.mocked(reportErrorToSentry)).not.toHaveBeenCalled();
  });

  it('should return error status if checkSettingsErrors throws an error', async () => {
    const settingsContent = 'invalid json';
    const parseError = new Error('Failed to parse');
    (checkSettingsErrors as Mock).mockRejectedValue(parseError);

    const result = await applySettings(settingsContent, mockSettingsTextStore);

    expect(result.settingsStatus).toBe('error');
    expect(result.errorMessage).toBe('Failed to parse');
    expect(checkSettingsErrors).toHaveBeenCalledWith([settingsContent]);
    expect(vi.mocked(reportErrorToSentry)).toHaveBeenCalledWith(
      parseError,
      SentrySeverityLevel.Info,
    );
  });

  it('should handle non-Error objects thrown by checkSettingsErrors', async () => {
    const settingsContent = 'another invalid input';
    const parseErrorString = 'Custom error string';
    (checkSettingsErrors as Mock).mockRejectedValue(parseErrorString);

    const result = await applySettings(settingsContent, mockSettingsTextStore);

    expect(result.settingsStatus).toBe('error');
    expect(result.errorMessage).toBe(parseErrorString);
    expect(vi.mocked(reportErrorToSentry)).toHaveBeenCalledWith(
      parseErrorString,
      SentrySeverityLevel.Info,
    );
  });
});
