import { formatUnits } from 'viem';

export function bigintToFloat(value: bigint, decimals: number) {
  return parseFloat(formatUnits(value, decimals));
}

/**
 * Converts a bigint string 18point decimals value to a float string, optionally
 * keeping the given number of decimals digits after "."
 * @param value - The bigint string value
 * @param valueDecimals - The bigint string value decimals point
 * @param decimalPoint - (optional) The number of digits to keep after "." in final result, defaults to valueDecimals
 */
export function bigintStringToPercentage(
  value: string,
  valueDecimals: number,
  finalDecimalsDigits?: number,
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
