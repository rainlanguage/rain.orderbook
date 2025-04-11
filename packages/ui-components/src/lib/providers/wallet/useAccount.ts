import type { Hex } from 'viem';
import { get } from 'svelte/store';
import { isAddress, isAddressEqual } from 'viem';
import { getAccountContext } from './context';
import { readable } from 'svelte/store';

/**
 * Hook to access wallet account information from context
 * Must be used within a component that is a child of WalletProvider
 */
export function useAccount() {
	
	/**
	 * The account store containing the current wallet address (as a Hex string) or null if not connected.
	 * This is a readable Svelte store that can be subscribed to for reactive updates.
	 * @type {import('svelte/store').Readable<Hex | null>}
	 */
	const account = getAccountContext();

	/**
	 * Checks if the provided address matches the currently connected account.
	 * Returns false if no account is connected or if the provided address is invalid.
	 */
	const matchesAccount = (otherAddress: string): boolean => {
		const currentAccount = get(account);
		if (!currentAccount) {
			return false;
		}

		if (isAddress(currentAccount) && isAddress(otherAddress) && isAddressEqual(currentAccount, otherAddress)) {
			return true;
		}

		return false;
	};

	return {
		account,
		matchesAccount
	};
}

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	vi.mock('viem', async () => {
		const actual = await vi.importActual('viem');
		return {
			...actual,
			isAddress: vi.fn(),
			isAddressEqual: vi.fn()
		};
	});

	vi.mock('./context', () => ({
		getAccountContext: vi.fn()
	}));

	vi.mock('svelte/store', async () => {
		const actual = await vi.importActual('svelte/store');
		return {
			...actual,
			get: vi.fn()
		};
	});

	describe('useAccount', () => {
		const mockGetAccountContext = vi.mocked(getAccountContext);
		const mockGet = vi.mocked(get);
		const mockIsAddress = vi.mocked(isAddress);
		const mockIsAddressEqual = vi.mocked(isAddressEqual);

		beforeEach(() => {
			mockGetAccountContext.mockReset();
			mockGet.mockReset();
			mockIsAddress.mockReset();
			mockIsAddressEqual.mockReset();
		});

		it('should return account wrapped in an object', () => {
			const mockAccountStore = readable('0x123' as Hex);
			mockGetAccountContext.mockReturnValue(mockAccountStore);

			const result = useAccount();

			expect(mockGetAccountContext).toHaveBeenCalled();
			expect(result.account).toBe(mockAccountStore);
			expect(result.matchesAccount).toBeInstanceOf(Function);
		});

		describe('matchesAccount', () => {
			const mockAccountStore = readable('0x123' as Hex);
			const testAddress1 = '0xabc' as Hex;
			const testAddress2 = '0xdef' as Hex;
			const invalidAddress = 'invalid';

			it('should return true if addresses are valid and equal', () => {
				mockGetAccountContext.mockReturnValue(mockAccountStore);
				mockGet.mockReturnValue(testAddress1);
				mockIsAddress.mockImplementation((addr) => addr !== invalidAddress);
				mockIsAddressEqual.mockReturnValue(true);

				const { matchesAccount } = useAccount();
				const result = matchesAccount(testAddress1);

				expect(mockGet).toHaveBeenCalledWith(mockAccountStore);
				expect(mockIsAddress).toHaveBeenCalledWith(testAddress1);
				expect(mockIsAddress).toHaveBeenCalledWith(testAddress1);
				expect(mockIsAddressEqual).toHaveBeenCalledWith(testAddress1, testAddress1);
				expect(result).toBe(true);
			});

			it('should return false if addresses are valid but not equal', () => {
				mockGetAccountContext.mockReturnValue(mockAccountStore);
				mockGet.mockReturnValue(testAddress1);
				mockIsAddress.mockImplementation((addr) => addr !== invalidAddress);
				mockIsAddressEqual.mockReturnValue(false);

				const { matchesAccount } = useAccount();
				const result = matchesAccount(testAddress2);

				expect(mockGet).toHaveBeenCalledWith(mockAccountStore);
				expect(mockIsAddress).toHaveBeenCalledWith(testAddress1);
				expect(mockIsAddress).toHaveBeenCalledWith(testAddress2);
				expect(mockIsAddressEqual).toHaveBeenCalledWith(testAddress1, testAddress2);
				expect(result).toBe(false);
			});

			it('should return false if current account is not set', () => {
				mockGetAccountContext.mockReturnValue(mockAccountStore);
				mockGet.mockReturnValue(undefined); // Simulate no account connected

				const { matchesAccount } = useAccount();
				const result = matchesAccount(testAddress1);

				expect(mockGet).toHaveBeenCalledWith(mockAccountStore);
				expect(mockIsAddress).not.toHaveBeenCalled();
				expect(mockIsAddressEqual).not.toHaveBeenCalled();
				expect(result).toBe(false);
			});

			it('should return false if current account is invalid', () => {
				mockGetAccountContext.mockReturnValue(mockAccountStore);
				mockGet.mockReturnValue(invalidAddress as Hex);
				mockIsAddress.mockImplementation((addr) => addr === testAddress1); // Only testAddress1 is valid

				const { matchesAccount } = useAccount();
				const result = matchesAccount(testAddress1);

				expect(mockGet).toHaveBeenCalledWith(mockAccountStore);
				expect(mockIsAddress).toHaveBeenCalledWith(invalidAddress);
				expect(mockIsAddress).toHaveBeenCalledWith(testAddress1);
				expect(mockIsAddressEqual).not.toHaveBeenCalled();
				expect(result).toBe(false);
			});

			it('should return false if provided address is invalid', () => {
				mockGetAccountContext.mockReturnValue(mockAccountStore);
				mockGet.mockReturnValue(testAddress1);
				mockIsAddress.mockImplementation((addr) => addr === testAddress1); // Only testAddress1 is valid

				const { matchesAccount } = useAccount();
				const result = matchesAccount(invalidAddress);

				expect(mockGet).toHaveBeenCalledWith(mockAccountStore);
				expect(mockIsAddress).toHaveBeenCalledWith(testAddress1);
				expect(mockIsAddress).toHaveBeenCalledWith(invalidAddress);
				expect(mockIsAddressEqual).not.toHaveBeenCalled();
				expect(result).toBe(false);
			});
		});
	});
}
