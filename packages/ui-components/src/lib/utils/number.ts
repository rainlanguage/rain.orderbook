import { formatUnits } from 'viem';

/**
 * Converts a bigint string value to a percentage with optionally given number of decimal points
 * @param value - The bigint string value
 * @param valueDecimals - The bigint string value decimals point
 * @param decimalPoint - (optional) The number of digits to keep after "." in final result, defaults to valueDecimals
 */
export function bigintStringToPercentage(
	value: string,
	valueDecimals: number,
	finalDecimalsDigits?: number
): string {
	const finalDecimals =
		typeof finalDecimalsDigits !== 'undefined' ? finalDecimalsDigits : valueDecimals;
	let valueString = formatUnits(BigInt(value) * 100n, valueDecimals);
	const index = valueString.indexOf('.');
	if (index > -1) {
		valueString = valueString.substring(0, finalDecimals === 0 ? index : index + finalDecimals + 1);
	}
	return valueString;
}

if (import.meta.vitest) {
	const { it, expect } = import.meta.vitest;

	it('should get percentage string from bigint string', () => {
		const value = '123456000000000000';
		const decimals = 18;
		const finalDecimalsDigits = 4;

		const result = bigintStringToPercentage(value, decimals, finalDecimalsDigits);
		const expected = '12.3456';

		expect(result).toEqual(expected);
	});
}
