import { getDotrainRegistryContext } from './context';

/**
 * Hook to access the current Dotrain registry context.
 */
export function useDotrainRegistry() {
	return getDotrainRegistryContext();
}

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('./context', () => ({
		getDotrainRegistryContext: vi.fn()
	}));

	describe('useDotrainRegistry', () => {
		const mockGetContext = vi.mocked(getDotrainRegistryContext);

		beforeEach(() => {
			mockGetContext.mockReset();
		});

		it('should return the registry context', () => {
			const mockContext = { registry: null };
			mockGetContext.mockReturnValue(mockContext as never);

			const result = useDotrainRegistry();

			expect(mockGetContext).toHaveBeenCalled();
			expect(result).toEqual(mockContext);
		});
	});
}
