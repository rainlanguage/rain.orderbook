import { formatUnits } from 'viem';

export function bigintToFloat(value: bigint, decimals: number) {
	return parseFloat(formatUnits(value, decimals));
}
