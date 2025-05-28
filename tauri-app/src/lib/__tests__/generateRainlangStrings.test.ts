import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { generateRainlangStrings } from '../services/generateRainlangStrings';
import type { ScenarioCfg } from '@rainlanguage/orderbook';
import { orderAddComposeRainlang } from '$lib/services/order';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';

// Mock dependencies
vi.mock('$lib/services/order', () => ({
  orderAddComposeRainlang: vi.fn(),
}));

vi.mock('$lib/services/sentry', async (importOriginal) => {
  const original = await importOriginal<typeof import('$lib/services/sentry')>();
  return {
    ...original,
    reportErrorToSentry: vi.fn(),
  };
});

const mockDotrainText = 'source test;';
const mockSettingsStrings = ['setting1: value1'];

describe('generateRainlangStrings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should return undefined if scenarios is undefined', async () => {
    const result = await generateRainlangStrings(mockDotrainText, mockSettingsStrings, undefined);
    expect(result).toBeUndefined();
  });

  it('should return undefined if scenarios is an empty object', async () => {
    const result = await generateRainlangStrings(mockDotrainText, mockSettingsStrings, {});
    expect(result).toBeUndefined();
  });

  it('should call orderAddComposeRainlang for each scenario and return a map of results', async () => {
    const scenario1: ScenarioCfg = { key: 's1' } as ScenarioCfg;
    const scenario2: ScenarioCfg = { key: 's2' } as ScenarioCfg;
    const scenarios = { s1: scenario1, s2: scenario2 };

    (orderAddComposeRainlang as Mock)
      .mockResolvedValueOnce('composed_s1')
      .mockResolvedValueOnce('composed_s2');

    const result = await generateRainlangStrings(mockDotrainText, mockSettingsStrings, scenarios);

    expect(orderAddComposeRainlang).toHaveBeenCalledTimes(2);
    expect(orderAddComposeRainlang).toHaveBeenNthCalledWith(
      1,
      mockDotrainText,
      mockSettingsStrings,
      scenario1,
    );
    expect(orderAddComposeRainlang).toHaveBeenNthCalledWith(
      2,
      mockDotrainText,
      mockSettingsStrings,
      scenario2,
    );

    expect(result).toBeInstanceOf(Map);
    expect(result?.get(scenario1)).toBe('composed_s1');
    expect(result?.get(scenario2)).toBe('composed_s2');
    expect(reportErrorToSentry).not.toHaveBeenCalled();
  });

  it('should handle errors from orderAddComposeRainlang for a specific scenario', async () => {
    const scenario1: ScenarioCfg = { key: 's1' } as ScenarioCfg;
    const scenario2: ScenarioCfg = { key: 's2' } as ScenarioCfg;
    const scenarios = { s1: scenario1, s2: scenario2 };
    const errorMsg = 'Composition failed for s1';

    (orderAddComposeRainlang as Mock)
      .mockRejectedValueOnce(new Error(errorMsg))
      .mockResolvedValueOnce('composed_s2');

    const result = await generateRainlangStrings(mockDotrainText, mockSettingsStrings, scenarios);

    expect(orderAddComposeRainlang).toHaveBeenCalledTimes(2);
    expect(result?.get(scenario1)).toBe(`Error: ${errorMsg}`);
    expect(result?.get(scenario2)).toBe('composed_s2');
    expect(reportErrorToSentry).not.toHaveBeenCalled();
  });

  it('should call reportErrorToSentry if an unexpected error occurs in the main function body', async () => {
    const scenario1: ScenarioCfg = { key: 's1' } as ScenarioCfg;
    const scenarios = { s1: scenario1 };
    const unexpectedError = new Error('Unexpected boom!');

    const originalObjectValues = Object.values;
    Object.values = vi.fn().mockImplementationOnce(() => {
      throw unexpectedError;
    });

    const result = await generateRainlangStrings(mockDotrainText, mockSettingsStrings, scenarios);

    expect(result).toBeUndefined();
    expect(reportErrorToSentry).toHaveBeenCalledWith(unexpectedError, SentrySeverityLevel.Error);

    Object.values = originalObjectValues;
  });

  it('should correctly pass dotrainText and settingsStrings to orderAddComposeRainlang', async () => {
    const scenario1: ScenarioCfg = { key: 's1' } as ScenarioCfg;
    const scenarios = { s1: scenario1 };
    const customDotrain = 'unique source;';
    const customSettings = ['settingA: valA'];

    (orderAddComposeRainlang as Mock).mockResolvedValueOnce('composed_custom');

    await generateRainlangStrings(customDotrain, customSettings, scenarios);

    expect(orderAddComposeRainlang).toHaveBeenCalledWith(customDotrain, customSettings, scenario1);
  });
});
