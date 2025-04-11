import { getContext, setContext } from 'svelte';
import type { Readable } from 'svelte/store';
import type { RegistryManager } from './RegistryManager';

export const REGISTRY_KEY = 'registry_key';

/**
 * Type for the registry store
 */
export type Registry = Readable<RegistryManager>;

/**
 * Retrieves the registry manager store directly from Svelte's context
 */
export const getRegistryContext = (): Registry => {
  const registry = getContext<Registry>(REGISTRY_KEY);
  if (!registry) {
    throw new Error(
      'No registry manager was found in Svelte context. Did you forget to wrap your component with RegistryProvider?'
    );
  }
  return registry;
};

/**
 * Sets the registry manager store in Svelte's context
 */
export const setRegistryContext = (registry: Registry) => {
  setContext(REGISTRY_KEY, registry);
};

if (import.meta.vitest) {
  const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

  vi.mock('svelte', async (importOriginal) => ({
    ...((await importOriginal()) as object),
    getContext: vi.fn()
  }));

  describe('getRegistryContext', () => {
    const mockGetContext = vi.mocked(getContext);

    beforeEach(() => {
      mockGetContext.mockReset();
    });

    it('should return the registry from context when it exists', () => {
      const mockRegistry = {} as Registry;

      mockGetContext.mockImplementation((key) => {
        if (key === REGISTRY_KEY) return mockRegistry;
        return undefined;
      });

      const result = getRegistryContext();
      expect(mockGetContext).toHaveBeenCalledWith(REGISTRY_KEY);
      expect(result).toEqual(mockRegistry);
    });

    it('should throw an error when registry is not in context', () => {
      mockGetContext.mockReturnValue(undefined);

      expect(() => getRegistryContext()).toThrow(
        'No registry manager was found in Svelte context. Did you forget to wrap your component with RegistryProvider?'
      );
    });
  });
}