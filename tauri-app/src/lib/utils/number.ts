import { formatUnits } from 'viem';

export function bigintToFloat(value: bigint, decimals: number) {
  return parseFloat(formatUnits(value, decimals));
}

/**
 * Converts a bigint string 18point decimals value to a float string, optionally
 * keeping the given number of decimals digits after "."
 * @param value - The bigint string value
 * @param decimalPoint - (optional) the number of digits to keep after "."
 */
export function bigintString18ToPercentage(value: string, decimalPoint?: number): string {
  let valueString = formatUnits(BigInt(value) * 100n, 18);
  const index = valueString.indexOf('.');
  if (decimalPoint !== undefined && index > -1) {
    valueString = valueString.substring(0, decimalPoint === 0 ? index : index + decimalPoint);
  }
  return valueString;
}
