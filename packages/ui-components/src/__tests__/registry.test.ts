import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fetchParseRegistry, fetchRegistryDotrains, validateStrategies } from '../lib/services/registry';
import { DotrainOrderGui } from '@rainlanguage/orderbook';
import type { Mock } from 'vitest';

// Mock the DotrainOrderGui dependency
vi.mock('@rainlanguage/orderbook', () => ({
	DotrainOrderGui: {
		getStrategyDetails: vi.fn()
	}
}));

describe('fetchParseRegistry', () => {
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

describe('validateStrategies', () => {
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
