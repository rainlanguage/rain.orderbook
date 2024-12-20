export const TIME_DELTA_24_HOURS = 60 * 60 * 24;
export const TIME_DELTA_48_HOURS = TIME_DELTA_24_HOURS * 2;
export const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
export const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
export const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;

export function dateTimestamp(date: Date): number {
	return Math.floor(date.getTime() / 1000);
}

if (import.meta.vitest) {
	const { it, expect } = import.meta.vitest;

	it('should get date timestamp in seconds', () => {
		const date = new Date(2022, 1, 16, 17, 32, 11, 168);
		const result = dateTimestamp(date);
		const expected = Math.floor(date.getTime() / 1000);

		expect(result).toEqual(expected);
	});
}
