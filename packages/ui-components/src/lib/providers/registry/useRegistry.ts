import { getRegistryContext } from './context';
import type { RegistryStore } from '$lib/types/registry';

/**
 * Hook to access registry manager information from context
 * Must be used within a component that is a child of RegistryProvider
 * @returns An object containing the registry manager store
 */
export function useRegistry() {
  const registry = getRegistryContext();
  return registry;
}

if (import.meta.vitest) {
  const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

  vi.mock('./RegistryContext', () => ({
    getRegistryContext: vi.fn()
  }));

  describe('useRegistry', () => {
    const mockGetRegistryContext = vi.mocked(getRegistryContext);

    beforeEach(() => {
      mockGetRegistryContext.mockReset();
    });

    it('should return registry wrapped in an object', () => {
      const mockRegistry = {} as RegistryStore;
      mockGetRegistryContext.mockReturnValue(mockRegistry);

      const result = useRegistry();

      expect(mockGetRegistryContext).toHaveBeenCalled();
      expect(result).toEqual({
        registry: mockRegistry
      });
    });
  });
}