/**
 * Fetches and parses a file registry from a given URL.
 * The registry is expected to be a text file where each line contains a file name and URL separated by a space.
 *
 * @param url - The URL of the registry file to fetch
 * @returns A Promise that resolves to an array of objects containing file names and their corresponding URLs
 * @throws Will throw an error if the fetch fails, if the response is not ok, or if the registry format is invalid
 *
 * @example
 * const files = await getFileRegistry('https://example.com/registry');
 * // Returns: [{ name: 'file1', url: 'https://example.com/file1.rain' }, ...]
 */

export const getFileRegistry = async (url: string) => {
	try {
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error('Failed to fetch registry.');
		}
		const filesList = await response.text();
		const files = filesList
			.split('\n')
			.filter((line) => line.trim())
			.map((line) => {
				const [name, url] = line.split(' ');
				return { name, url };
			});
		if (!files) {
			throw new Error('Invalid stategy registry.');
		}
		return files;
	} catch (e) {
		throw new Error(e instanceof Error ? e.message : 'Unknown error.');
	}
};

if (import.meta.vitest) {
	const { describe, it, expect, vi } = import.meta.vitest;

	describe('getFileRegistry', () => {
		it('should parse registry file content correctly', async () => {
			const mockResponse = `file1.js https://example.com/file1.js
file2.js https://example.com/file2.js`;

			global.fetch = vi.fn().mockResolvedValue({
				ok: true,
				text: () => Promise.resolve(mockResponse)
			});

			const result = await getFileRegistry('https://example.com/registry');
			expect(result).toEqual([
				{ name: 'file1.js', url: 'https://example.com/file1.js' },
				{ name: 'file2.js', url: 'https://example.com/file2.js' }
			]);
		});

		it('should handle failed fetch response', async () => {
			global.fetch = vi.fn().mockResolvedValue({
				ok: false
			});

			await expect(getFileRegistry('https://example.com/registry')).rejects.toThrow(
				'Failed to fetch registry'
			);
		});

		it('should handle network errors', async () => {
			global.fetch = vi.fn().mockRejectedValue(new Error('Network error'));

			await expect(getFileRegistry('https://example.com/registry')).rejects.toThrow(
				'Network error'
			);
		});
	});
}
