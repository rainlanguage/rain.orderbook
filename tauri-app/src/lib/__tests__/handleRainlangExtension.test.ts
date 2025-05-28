import { describe, it, expect, vi, type Mock } from 'vitest';
import { createRainlangExtension } from '../services/handleRainlangExtension';
import { RawRainlangExtension } from 'codemirror-rainlang';

vi.mock('codemirror-rainlang', () => ({
  RawRainlangExtension: vi.fn(),
}));

vi.mock('@rainlanguage/ui-components', async () => {
  const actual = await vi.importActual("@rainlanguage/ui-components");
  return {
    ...actual,
    promiseTimeout: vi.fn().mockResolvedValue([])
  };
});

describe('createRainlangExtension', () => {
  it('should create a RawRainlangExtension instance', () => {
    createRainlangExtension({}, undefined);
    expect(RawRainlangExtension as Mock).toHaveBeenCalled();
  });
}); 