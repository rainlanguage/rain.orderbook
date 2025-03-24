import type { Hex } from 'viem';
import { getAccountContext } from './context';
import { readable } from 'svelte/store';

/**
 * Hook to access wallet account information from context
 * Must be used within a component that is a child of WalletProvider
 */
export function useAccount() {
	const account = getAccountContext();
	return {
		account
	};
}

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('./context', () => ({
		getAccountContext: vi.fn()
	}));

	describe('useAccount', () => {
		const mockGetAccountContext = vi.mocked(getAccountContext);

		beforeEach(() => {
			mockGetAccountContext.mockReset();
		});

		it('should return account wrapped in an object', () => {
			const mockAccount = readable('0x123' as unknown as Hex);
			mockGetAccountContext.mockReturnValue(mockAccount);

			const result = useAccount();

			expect(mockGetAccountContext).toHaveBeenCalled();
			expect(result).toEqual({
				account: mockAccount
			});
		});
	});
}