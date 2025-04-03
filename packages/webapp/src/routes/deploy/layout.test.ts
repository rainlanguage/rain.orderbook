import { describe, it, expect, vi, beforeEach } from 'vitest';
import { load } from './+layout';
import {
	validateStrategies,
	fetchRegistryDotrains,
	type RegistryDotrain
} from '@rainlanguage/ui-components/services';
import { REGISTRY_URL } from '$lib/constants';
import type { InvalidStrategyDetail } from '@rainlanguage/ui-components';
import type { ValidStrategyDetail } from '@rainlanguage/ui-components';

vi.mock('@rainlanguage/ui-components/services', () => ({
	validateStrategies: vi.fn(),
	fetchRegistryDotrains: vi.fn()
}));

const mockDotrains = ['dotrain1', 'dotrain2'] as unknown as RegistryDotrain[];
const mockValidated = {
	validStrategies: ['strategy1', 'strategy2'] as unknown as ValidStrategyDetail[],
	invalidStrategies: ['invalidStrategy'] as unknown as InvalidStrategyDetail[]
};

describe('Layout load function', () => {
	beforeEach(() => {
		vi.resetAllMocks();
	});

	const createUrlMock = (registryParam: string | null) => ({
		url: {
			searchParams: {
				get: vi.fn().mockReturnValue(registryParam)
			}
		}
	}) as any;

	const testLoadFunction = async ({
		registryParam = null,
		fetchError = null,
		validateError = null
	}: {
		registryParam?: string | null;
		fetchError?: Error | string | null;
		validateError?: Error | null;
	}) => {
		const expectedRegistry = registryParam || REGISTRY_URL;

		if (fetchError) {
			if (typeof fetchError === 'string') {
				vi.mocked(fetchRegistryDotrains).mockRejectedValue(fetchError);
			} else {
				vi.mocked(fetchRegistryDotrains).mockRejectedValue(fetchError);
			}
		} else {
			vi.mocked(fetchRegistryDotrains).mockResolvedValue(mockDotrains);
		}

		if (validateError) {
			vi.mocked(validateStrategies).mockRejectedValue(validateError);
		} else {
			vi.mocked(validateStrategies).mockResolvedValue(mockValidated);
		}

		const result = await load(createUrlMock(registryParam));

		expect(fetchRegistryDotrains).toHaveBeenCalledWith(expectedRegistry);
		
		if (!fetchError) {
			expect(validateStrategies).toHaveBeenCalledWith(mockDotrains);
		}

		return result;
	};

	it('should load strategies from default registry URL when no registry param is provided', async () => {
		const result = await testLoadFunction({});

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: mockDotrains,
			validStrategies: mockValidated.validStrategies,
			invalidStrategies: mockValidated.invalidStrategies,
			error: null
		});
	});

	it('should load strategies from custom registry URL when registry param is provided', async () => {
		const customRegistry = 'https://custom.registry.url';
		const result = await testLoadFunction({ registryParam: customRegistry });

		expect(result).toEqual({
			registry: customRegistry,
			registryDotrains: mockDotrains,
			validStrategies: mockValidated.validStrategies,
			invalidStrategies: mockValidated.invalidStrategies,
			error: null
		});
	});

	it('should handle errors when fetchRegistryDotrains fails', async () => {
		const errorMessage = 'Failed to fetch registry dotrains';
		const result = await testLoadFunction({ fetchError: new Error(errorMessage) });

		expect(validateStrategies).not.toHaveBeenCalled();

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: errorMessage
		});
	});

	it('should handle errors when validateStrategies fails', async () => {
		const errorMessage = 'Failed to validate strategies';
		const result = await testLoadFunction({ validateError: new Error(errorMessage) });

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: errorMessage
		});
	});

	it('should handle non-Error exceptions with an "Unknown error" message', async () => {
		const result = await testLoadFunction({ fetchError: 'Not an error object' });

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: 'Unknown error occurred'
		});
	});
});