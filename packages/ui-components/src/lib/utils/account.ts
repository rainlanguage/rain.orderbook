import { isAddress, isAddressEqual, type Address } from 'viem';

// This function checks if two addresses are equal, handling undefined and null cases.
// It uses the viem library to ensure the addresses are valid before comparison.
// If either address is undefined or null, it returns false.
// If both addresses are valid, it checks for equality using isAddressEqual.
export const isAddressEq = (a?: Address | null, b?: Address | null) => {
	try {
		if (!a || !b) return false;
		return isAddress(a) && isAddress(b) && isAddressEqual(a, b);
	} catch {
		return false;
	}
};
