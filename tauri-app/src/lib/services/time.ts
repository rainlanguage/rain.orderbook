export const TIME_DELTA_24_HOURS = 60 * 60 * 24;
export const TIME_DELTA_48_HOURS = TIME_DELTA_24_HOURS * 2;
export const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
export const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
export const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;

export function nowTimestamp(): number {
  return Math.floor(new Date().getTime() / 1000);
}
