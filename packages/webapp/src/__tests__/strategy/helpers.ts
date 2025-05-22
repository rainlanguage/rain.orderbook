import { REGISTRY_URL } from '$lib/constants';

export const fetchRegistry = async () => {
	const response = await fetch(REGISTRY_URL);
	const registry = await response.text();
	const linksMap = Object.fromEntries(
		registry
			.split('\n')
			.map((line) => line.trim().split(' '))
			.filter((parts) => parts.length === 2)
	);
	return linksMap;
};
export const fetchStrategy = async (url: string) => {
	try {
		const response = await fetch(url);
		return await response.text();
	} catch (error) {
		assert.fail(error as string);
	}
};
export function findLockRegion(a: string, b: string): { prefixEnd: number; suffixEnd: number } {
	expect(a.length).toEqual(b.length);
	const length = a.length;
	// Find prefix end
	let prefixEnd = 0;
	while (prefixEnd < length && a[prefixEnd] === b[prefixEnd]) {
		prefixEnd++;
	}
	// Find suffix start
	let suffixEnd = length;
	while (suffixEnd > prefixEnd && a[suffixEnd - 1] === b[suffixEnd - 1]) {
		suffixEnd--;
	}
	return { prefixEnd, suffixEnd };
}
