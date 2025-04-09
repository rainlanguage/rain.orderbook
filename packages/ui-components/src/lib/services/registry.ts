import type { InvalidStrategyDetail, ValidStrategyDetail } from '$lib/types/strategy';
import { DotrainOrderGui } from '@rainlanguage/orderbook';
import type { Mock } from 'vitest';

export type RegistryFile = {
	name: string;
	url: string;
};

export type RegistryDotrain = {
	name: string;
	dotrain: string;
};

export interface StrategyValidationResult {
	validStrategies: ValidStrategyDetail[];
	invalidStrategies: InvalidStrategyDetail[];
}

/**
 * Fetches and parses a file registry from a given URL.
 * The registry is expected to be a text file where each line contains a file name and URL separated by a space.
 *
 * @param url - The URL of the registry file to fetch
 * @returns A Promise that resolves to an array of objects containing file names and their corresponding URLs
 * @throws Will throw an error if the fetch fails, if the response is not ok, or if the registry format is invalid
 *
 * @example
 * const files = await fetchParseRegistryFile('https://example.com/registry');
 * // Returns: [{ name: 'file1', url: 'https://example.com/file1.rain' }, ...]
 */

export const fetchParseRegistry = async (url: string): Promise<{ name: string; url: string }[]> => {
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

export const fetchRegistryDotrains = async (url: string): Promise<RegistryDotrain[]> => {
	const files = await fetchParseRegistry(url);
	const dotrains = await Promise.all(
		files.map(async (file) => {
			try {
				const response = await fetch(file.url);
				if (!response.ok) {
					throw new Error(`Failed to fetch dotrain for ${file.name}`);
				}
				const dotrain = await response.text();
				return { name: file.name, dotrain };
			} catch (e) {
				throw new Error(
					e instanceof Error
						? `Error fetching dotrain for ${file.name}: ${e.message}`
						: `Unknown error fetching dotrain for ${file.name}`
				);
			}
		})
	);
	return dotrains;
};

export async function validateStrategies(
	registryDotrains: RegistryDotrain[]
): Promise<StrategyValidationResult> {
	const strategiesPromises = registryDotrains.map(async (registryDotrain) => {
		try {
			const result = await DotrainOrderGui.getStrategyDetails(registryDotrain.dotrain);

			if (result.error) {
				throw new Error(result.error.msg);
			}

			return {
				valid: true,
				data: {
					...registryDotrain,
					details: result.value
				}
			};
		} catch (error) {
			return {
				valid: false,
				data: {
					name: registryDotrain.name,
					error: error instanceof Error ? error.message : String(error)
				}
			};
		}
	});

	const strategiesResults = await Promise.all(strategiesPromises);

	const validStrategies = strategiesResults
		.filter((result) => result.valid)
		.map((result) => result.data as ValidStrategyDetail);

	const invalidStrategies = strategiesResults
		.filter((result) => !result.valid)
		.map((result) => result.data as InvalidStrategyDetail);

	return { validStrategies, invalidStrategies };
}

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

			const result = await fetchParseRegistry('https://example.com/registry');
			expect(result).toEqual([
				{ name: 'file1.js', url: 'https://example.com/file1.js' },
				{ name: 'file2.js', url: 'https://example.com/file2.js' }
			]);
		});

		it('should handle failed fetch response', async () => {
			global.fetch = vi.fn().mockResolvedValue({
				ok: false
			});

			await expect(fetchParseRegistry('https://example.com/registry')).rejects.toThrow(
				'Failed to fetch registry'
			);
		});

		it('should handle network errors', async () => {
			global.fetch = vi.fn().mockRejectedValue(new Error('Network error'));

			await expect(fetchParseRegistry('https://example.com/registry')).rejects.toThrow(
				'Network error'
			);
		});
	});

	describe('fetchRegistryDotrains', () => {
		it('should fetch and parse dotrains correctly', async () => {
			const mockRegistry = `file1.rain https://example.com/file1.rain
file2.rain https://example.com/file2.rain`;

			const mockDotrain1 = 'content of file1';
			const mockDotrain2 = 'content of file2';

			global.fetch = vi
				.fn()
				.mockResolvedValueOnce({
					ok: true,
					text: () => Promise.resolve(mockRegistry)
				})
				.mockResolvedValueOnce({
					ok: true,
					text: () => Promise.resolve(mockDotrain1)
				})
				.mockResolvedValueOnce({
					ok: true,
					text: () => Promise.resolve(mockDotrain2)
				});

			const result = await fetchRegistryDotrains('https://example.com/registry');
			expect(result).toEqual([
				{ name: 'file1.rain', dotrain: mockDotrain1 },
				{ name: 'file2.rain', dotrain: mockDotrain2 }
			]);
		});

		it('should handle failed dotrain fetch', async () => {
			const mockRegistry = `file1.rain https://example.com/file1.rain`;

			global.fetch = vi
				.fn()
				.mockResolvedValueOnce({
					ok: true,
					text: () => Promise.resolve(mockRegistry)
				})
				.mockResolvedValueOnce({
					ok: false
				});

			await expect(fetchRegistryDotrains('https://example.com/registry')).rejects.toThrow(
				'Failed to fetch dotrain for file1.rain'
			);
		});

		it('should handle network errors during dotrain fetch', async () => {
			const mockRegistry = `file1.rain https://example.com/file1.rain`;

			global.fetch = vi
				.fn()
				.mockResolvedValueOnce({
					ok: true,
					text: () => Promise.resolve(mockRegistry)
				})
				.mockRejectedValueOnce(new Error('Network error'));

			await expect(fetchRegistryDotrains('https://example.com/registry')).rejects.toThrow(
				'Error fetching dotrain for file1.rain: Network error'
			);
		});
	});

	describe('validateStrategies', async () => {
		// Mock the DotrainOrderGui dependency
		vi.mock('@rainlanguage/orderbook', () => ({
			DotrainOrderGui: {
				getStrategyDetails: vi.fn()
			}
		}));

		// Import DotrainOrderGui after mocking
		const { DotrainOrderGui } = await import('@rainlanguage/orderbook');

		beforeEach(() => {
			vi.resetAllMocks();
		});

		it('should validate strategies and categorize them properly', async () => {
			// Input data
			const registryDotrains = [
				{ name: 'valid.rain', dotrain: 'valid dotrain content' },
				{ name: 'invalid.rain', dotrain: 'invalid dotrain content' },
				{ name: 'another-valid.rain', dotrain: 'another valid content' }
			];

			// Set up mock responses for the DotrainOrderGui
			(DotrainOrderGui.getStrategyDetails as Mock)
				.mockResolvedValueOnce({
					value: { name: 'Valid Strategy', description: 'A valid strategy' },
					error: null
				})
				.mockResolvedValueOnce({
					error: { msg: 'Invalid syntax' },
					value: null
				})
				.mockResolvedValueOnce({
					value: { name: 'Another Valid', description: 'Another valid strategy' },
					error: null
				});

			// Call the function with our test data
			const result = await validateStrategies(registryDotrains);

			// Verify DotrainOrderGui was called correctly
			expect(DotrainOrderGui.getStrategyDetails).toHaveBeenCalledTimes(3);
			expect(DotrainOrderGui.getStrategyDetails).toHaveBeenCalledWith('valid dotrain content');
			expect(DotrainOrderGui.getStrategyDetails).toHaveBeenCalledWith('invalid dotrain content');
			expect(DotrainOrderGui.getStrategyDetails).toHaveBeenCalledWith('another valid content');

			// Verify the valid strategies are processed correctly
			expect(result.validStrategies).toHaveLength(2);
			expect(result.validStrategies[0].name).toBe('valid.rain');
			expect(result.validStrategies[0].dotrain).toBe('valid dotrain content');
			expect(result.validStrategies[0].details).toEqual({
				name: 'Valid Strategy',
				description: 'A valid strategy'
			});

			// Verify the invalid strategies are processed correctly
			expect(result.invalidStrategies).toHaveLength(1);
			expect(result.invalidStrategies[0].name).toBe('invalid.rain');
			expect(result.invalidStrategies[0].error).toBe('Invalid syntax');
		});

		it('should handle exceptions thrown during strategy validation', async () => {
			// Input data
			const registryDotrains = [{ name: 'error.rain', dotrain: 'will throw error' }];

			// Mock the DotrainOrderGui to throw an exception
			(DotrainOrderGui.getStrategyDetails as Mock).mockRejectedValueOnce(
				new Error('Unexpected parsing error')
			);

			// Call the function
			const result = await validateStrategies(registryDotrains);

			// Verify results
			expect(result.validStrategies).toHaveLength(0);
			expect(result.invalidStrategies).toHaveLength(1);
			expect(result.invalidStrategies[0].name).toBe('error.rain');
			expect(result.invalidStrategies[0].error).toBe('Unexpected parsing error');
		});

		it('should handle non-Error objects being thrown', async () => {
			// Input data
			const registryDotrains = [{ name: 'string-error.rain', dotrain: 'will throw string' }];

			// Mock the DotrainOrderGui to throw a string instead of an Error
			(DotrainOrderGui.getStrategyDetails as Mock).mockRejectedValueOnce('String error message');

			// Call the function
			const result = await validateStrategies(registryDotrains);

			// Verify results
			expect(result.validStrategies).toHaveLength(0);
			expect(result.invalidStrategies).toHaveLength(1);
			expect(result.invalidStrategies[0].name).toBe('string-error.rain');
			expect(result.invalidStrategies[0].error).toBe('String error message');
		});

		it('should process an empty array of strategies', async () => {
			const result = await validateStrategies([]);

			expect(result.validStrategies).toEqual([]);
			expect(result.invalidStrategies).toEqual([]);
			expect(DotrainOrderGui.getStrategyDetails).not.toHaveBeenCalled();
		});

		it('should handle mixed validation results correctly', async () => {
			// Create a mix of scenarios
			const registryDotrains = [
				{ name: 'valid1.rain', dotrain: 'valid content 1' },
				{ name: 'error.rain', dotrain: 'will throw error' },
				{ name: 'valid2.rain', dotrain: 'valid content 2' },
				{ name: 'invalid.rain', dotrain: 'invalid content' }
			];

			// Set up mock responses
			(DotrainOrderGui.getStrategyDetails as Mock)
				.mockResolvedValueOnce({
					value: { strategyName: 'Strategy 1', description: 'Description 1' },
					error: null
				})
				.mockRejectedValueOnce(new Error('Processing error'))
				.mockResolvedValueOnce({
					value: { strategyName: 'Strategy 2', description: 'Description 2' },
					error: null
				})
				.mockResolvedValueOnce({
					error: { msg: 'Validation failed' },
					value: null
				});

			// Call the function
			const result = await validateStrategies(registryDotrains);

			// Verify results
			expect(result.validStrategies).toHaveLength(2);
			expect(result.validStrategies[0].name).toBe('valid1.rain');
			expect(result.validStrategies[1].name).toBe('valid2.rain');

			expect(result.invalidStrategies).toHaveLength(2);
			expect(result.invalidStrategies[0].name).toBe('error.rain');
			expect(result.invalidStrategies[0].error).toBe('Processing error');
			expect(result.invalidStrategies[1].name).toBe('invalid.rain');
			expect(result.invalidStrategies[1].error).toBe('Validation failed');
		});
	});
}
