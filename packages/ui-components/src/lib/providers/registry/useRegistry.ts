import { getRegistryContext } from './context';
import type { RegistryContext } from './context';

/**
 * Hook to access registry manager information from context
 * Must be used within a component that is a child of RegistryProvider
 * @returns An object containing the registry manager
 */
export function useRegistry(): RegistryContext {
    const ctx = getRegistryContext();
    return ctx;
}

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('./context', () => ({
		getRegistryContext: vi.fn()
	}));

	describe('useRegistry', () => {
		const mockGetRegistryContext = vi.mocked(getRegistryContext);

		beforeEach(() => {
			mockGetRegistryContext.mockReset();
		});

        it('should return registry context', () => {
            const mockRegistry = {
                registry: null,
                loading: false,
                error: null,
                setRegistryUrl: () => {}
            } as unknown as RegistryContext;
            mockGetRegistryContext.mockReturnValue(mockRegistry as unknown as RegistryContext);

            const result = useRegistry();

            expect(mockGetRegistryContext).toHaveBeenCalled();
            expect(result).toEqual(mockRegistry);
        });
    });
}
