import fc from 'fast-check';
import { test } from '@fast-check/vitest';
/**
 * Validates an amount against a balance and returns validation results
 *
 * @param amount The amount being entered
 * @param balance The available balance
 * @param action The action type ('deposit' or 'withdraw')
 * @returns Validation results object
 */

export function validateAmount(
	amount: bigint,
	balance: bigint
): {
	isValid: boolean;
	isZero: boolean;
	exceedsBalance: boolean;
	errorMessage: string | null;
} {
	const isZero = amount <= 0n;
	const exceedsBalance = amount > balance;
	const isValid = !isZero && !exceedsBalance;

	let errorMessage = null;
	if (isZero) {
		errorMessage = 'Amount must be greater than zero.';
	} else if (exceedsBalance) {
		errorMessage = 'Amount cannot exceed available balance.';
	}

	return {
		isValid,
		isZero,
		exceedsBalance,
		errorMessage
	};
}

if (import.meta.vitest) {
	const { expect } = import.meta.vitest;

	test.prop([fc.bigInt(), fc.bigInt()])(
		'validates amounts against balances correctly',
		(amount, balance) => {
			const result = validateAmount(amount, balance);

			expect(result.isZero).toBe(amount <= 0n);
			expect(result.exceedsBalance).toBe(amount > balance);
			expect(result.isValid).toBe(amount > 0n && amount <= balance);

			if (amount <= 0n) {
				expect(result.isValid).toBe(false);
				expect(result.errorMessage).toContain('greater than zero');
			} else if (amount > balance) {
				expect(result.isValid).toBe(false);
				expect(result.errorMessage).toContain('exceed available balance');
			} else {
				expect(result.isValid).toBe(true);
				expect(result.errorMessage).toBeNull();
			}
		}
	);
}
