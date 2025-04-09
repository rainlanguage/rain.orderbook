import { isAddress, isAddressEqual } from 'viem';

export const accountIsOwner = (account: string, owner: string) => {
	if (isAddress(account) && isAddress(owner) && isAddressEqual(account, owner)) {
		return true;
	}
	return false;
};

if (import.meta.vitest) {
	const { test, expect } = import.meta.vitest;

	test('accountIsOwner', () => {
		const validAddress = '0x1234567890123456789012345678901234567890';
		expect(accountIsOwner(validAddress, validAddress)).toBe(true);

		expect(accountIsOwner('0x123', '0x456')).toBe(false);

		expect(accountIsOwner('0x123', '0x1234567890123456789012345678901234567890')).toBe(false);

		expect(
			accountIsOwner(
				'0x1234567890123456789012345678901234567890',
				'0x0987654321098765432109876543210987654321'
			)
		).toBe(false);
	});
}
