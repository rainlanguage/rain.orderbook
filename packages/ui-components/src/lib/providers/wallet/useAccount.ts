import { getUseAccountContext, getAccountContext } from './context';
import { readable } from 'svelte/store';

/**
 * Hook to access wallet account information from context
 * Must be used within a component that is a child of WalletProvider
 */
export function useAccount() {
	// Try to get the useAccount function from context first
	const useAccountFn = getUseAccountContext();
	if (useAccountFn) {
		return useAccountFn();
	}

	// Fallback to direct context access if needed
	const account = getAccountContext();

	return {
		account
	};
}

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('./context', () => ({
		getUseAccountContext: vi.fn(),
		getAccountContext: vi.fn()
	}));

	describe('useAccount', () => {
		const mockGetUseAccountContext = vi.mocked(getUseAccountContext);
		const mockGetAccountContext = vi.mocked(getAccountContext);

		beforeEach(() => {
			mockGetUseAccountContext.mockReset();
			mockGetAccountContext.mockReset();
		});

		it('should use the useAccount function from context when available', () => {
			const mockAccount = readable('0x123');
			const mockUseAccountFn = vi.fn().mockReturnValue({
				account: mockAccount
			});

			mockGetUseAccountContext.mockReturnValue(mockUseAccountFn);

			const result = useAccount();

			expect(mockGetUseAccountContext).toHaveBeenCalled();
			expect(mockUseAccountFn).toHaveBeenCalled();
			expect(result).toEqual({
				account: mockAccount
			});
			expect(mockGetAccountContext).not.toHaveBeenCalled();
		});

		it('should fall back to direct account context when useAccount function is not available', () => {
			const mockAccount = readable('0x456');

			mockGetUseAccountContext.mockReturnValue(undefined);
			mockGetAccountContext.mockReturnValue(mockAccount);

			const result = useAccount();

			expect(mockGetUseAccountContext).toHaveBeenCalled();
			expect(mockGetAccountContext).toHaveBeenCalled();
			expect(result).toEqual({
				account: mockAccount
			});
		});
	});
}
