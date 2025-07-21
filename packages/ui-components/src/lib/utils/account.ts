import { isAddress, isAddressEqual, type Address } from 'viem';

/**
 * Safely compares two Ethereum addresses for equality.
 *
 * @param a - First address to compare
 * @param b - Second address to compare
 * @returns true if both addresses are valid and equal, false otherwise
 *
 * Returns false if:
 * - Either address is null/undefined
 * - Either address is invalid
 * - Addresses are valid but not equal
 * - Any error occurs during comparison
 */
export const isAddressEq = (a?: Address | null, b?: Address | null) => {
	try {
		if (!a || !b) return false;
		return isAddress(a) && isAddress(b) && isAddressEqual(a, b);
	} catch {
		return false;
	}
};
