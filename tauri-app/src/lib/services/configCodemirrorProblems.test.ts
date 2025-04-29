import { describe, it, expect, vi, beforeEach } from 'vitest';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';
import { mergeDotrainConfigWithSettings, parseConfigSource } from './config';
import {
  parseConfigSourceProblems,
  mergeDotrainConfigWithSettingsProblems,
  convertErrorToProblem,
} from './configCodemirrorProblems';

// Mock dependencies without referencing codemirror-rainlang
vi.mock('$lib/services/sentry', () => ({
  reportErrorToSentry: vi.fn(),
  SentrySeverityLevel: { Info: 'info' },
}));

vi.mock('./config', () => ({
  parseConfigSource: vi.fn(),
  mergeDotrainConfigWithSettings: vi.fn(),
}));

describe('parseConfigSourceProblems', () => {
  const mockParseConfigSource = vi.mocked(parseConfigSource);
  const mockReportErrorToSentry = vi.mocked(reportErrorToSentry);

  beforeEach(() => {
    mockParseConfigSource.mockReset();
    mockReportErrorToSentry.mockReset();
  });

  it('should return empty problems array when parsing succeeds', async () => {
    mockParseConfigSource.mockResolvedValue({});

    const result = await parseConfigSourceProblems('valid config');

    expect(mockParseConfigSource).toHaveBeenCalledWith('valid config');
    expect(result).toEqual([]);
    expect(mockReportErrorToSentry).not.toHaveBeenCalled();
  });

  it('should return problems array with error when parsing fails with string error', async () => {
    const errorMessage = 'Invalid config';
    mockParseConfigSource.mockRejectedValue(errorMessage);

    const result = await parseConfigSourceProblems('invalid config');

    expect(mockParseConfigSource).toHaveBeenCalledWith('invalid config');
    expect(mockReportErrorToSentry).toHaveBeenCalledWith(errorMessage, SentrySeverityLevel.Info);

    // Test shape of response without referencing specific error code
    expect(result).toHaveLength(1);
    expect(result[0]).toHaveProperty('msg', errorMessage);
    expect(result[0]).toHaveProperty('position');
    expect(result[0].position).toEqual([0, 0]);
    expect(result[0]).toHaveProperty('code');
  });

  it('should return problems array with error when parsing fails with Error object', async () => {
    const error = new Error('Invalid config error');
    mockParseConfigSource.mockRejectedValue(error);

    const result = await parseConfigSourceProblems('invalid config');

    expect(mockParseConfigSource).toHaveBeenCalledWith('invalid config');
    expect(mockReportErrorToSentry).toHaveBeenCalledWith(error, SentrySeverityLevel.Info);

    expect(result).toHaveLength(1);
    expect(result[0]).toHaveProperty('msg', 'Invalid config error');
    expect(result[0]).toHaveProperty('position');
    expect(result[0].position).toEqual([0, 0]);
    expect(result[0]).toHaveProperty('code');
  });

  it('should return problems array with generic message when parsing fails with unknown error type', async () => {
    const unknownError = { foo: 'bar' };
    mockParseConfigSource.mockRejectedValue(unknownError);

    const result = await parseConfigSourceProblems('invalid config');

    expect(mockParseConfigSource).toHaveBeenCalledWith('invalid config');
    expect(mockReportErrorToSentry).toHaveBeenCalledWith(unknownError, SentrySeverityLevel.Info);

    expect(result).toHaveLength(1);
    expect(result[0]).toHaveProperty('msg', 'something went wrong!');
    expect(result[0]).toHaveProperty('position');
    expect(result[0].position).toEqual([0, 0]);
    expect(result[0]).toHaveProperty('code');
  });
});

describe('mergeDotrainConfigWithSettingsProblems', () => {
  const mockMergeDotrainConfigWithSettings = vi.mocked(mergeDotrainConfigWithSettings);
  const mockReportErrorToSentry = vi.mocked(reportErrorToSentry);

  beforeEach(() => {
    mockMergeDotrainConfigWithSettings.mockReset();
    mockReportErrorToSentry.mockReset();
  });

  it('should return empty problems array when merging succeeds', async () => {
    mockMergeDotrainConfigWithSettings.mockResolvedValue({});

    const result = await mergeDotrainConfigWithSettingsProblems('valid dotrain');

    expect(mockMergeDotrainConfigWithSettings).toHaveBeenCalledWith('valid dotrain');
    expect(result).toEqual([]);
    expect(mockReportErrorToSentry).not.toHaveBeenCalled();
  });

  it('should return problems array with error when merging fails with string error', async () => {
    const errorMessage = 'Merge error string';
    mockMergeDotrainConfigWithSettings.mockRejectedValue(errorMessage);

    const result = await mergeDotrainConfigWithSettingsProblems('invalid dotrain');

    expect(mockMergeDotrainConfigWithSettings).toHaveBeenCalledWith('invalid dotrain');
    expect(mockReportErrorToSentry).toHaveBeenCalledWith(errorMessage, SentrySeverityLevel.Info);

    expect(result).toHaveLength(1);
    expect(result[0]).toHaveProperty('msg', errorMessage);
    expect(result[0]).toHaveProperty('position');
    expect(result[0].position).toEqual([0, 0]);
    expect(result[0]).toHaveProperty('code');
  });

  it('should return problems array with error when merging fails with Error object', async () => {
    const error = new Error('Merge error');
    mockMergeDotrainConfigWithSettings.mockRejectedValue(error);

    const result = await mergeDotrainConfigWithSettingsProblems('invalid dotrain');

    expect(mockMergeDotrainConfigWithSettings).toHaveBeenCalledWith('invalid dotrain');
    expect(mockReportErrorToSentry).toHaveBeenCalledWith(error, SentrySeverityLevel.Info);

    expect(result).toHaveLength(1);
    expect(result[0]).toHaveProperty('msg', 'Merge error');
    expect(result[0]).toHaveProperty('position');
    expect(result[0].position).toEqual([0, 0]);
    expect(result[0]).toHaveProperty('code');
  });
});

describe('convertErrorToProblem', () => {
  it('should convert string error to Problem with correct shape', () => {
    const result = convertErrorToProblem('test error');

    expect(result).toHaveProperty('msg', 'test error');
    expect(result).toHaveProperty('position');
    expect(result.position).toEqual([0, 0]);
    expect(result).toHaveProperty('code');
  });

  it('should convert Error object to Problem with correct shape', () => {
    const error = new Error('error message');
    const result = convertErrorToProblem(error);

    expect(result).toHaveProperty('msg', 'error message');
    expect(result).toHaveProperty('position');
    expect(result.position).toEqual([0, 0]);
    expect(result).toHaveProperty('code');
  });

  it('should convert unknown error type to Problem with generic message', () => {
    const result = convertErrorToProblem({ unknown: 'type' });

    expect(result).toHaveProperty('msg', 'something went wrong!');
    expect(result).toHaveProperty('position');
    expect(result.position).toEqual([0, 0]);
    expect(result).toHaveProperty('code');
  });
});
