export function isStringValidNumber(val: string) {
  return /^\d+.?[\d]+$/.test(val);
}