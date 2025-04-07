import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
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

	const createUrlMock = (registryParam: string | null) =>
		({
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue(registryParam)
				}
			}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		}) as any;

	it('should load strategies from default registry URL when no registry param is provided', async () => {
		(validateStrategies as Mock).mockResolvedValue(mockValidated);
		(fetchRegistryDotrains as Mock).mockResolvedValue(mockDotrains);

		const result = await load(createUrlMock(null));

		expect(fetchRegistryDotrains).toHaveBeenCalledWith(REGISTRY_URL);

		expect(validateStrategies).toHaveBeenCalledWith(mockDotrains);

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
		vi.mocked(fetchRegistryDotrains).mockResolvedValue(mockDotrains);
		vi.mocked(validateStrategies).mockResolvedValue(mockValidated);

		const result = await load(createUrlMock(customRegistry));

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
		(fetchRegistryDotrains as Mock).mockRejectedValue(new Error(errorMessage));
		const result = await load(createUrlMock(null));

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
		(validateStrategies as Mock).mockRejectedValue(new Error(errorMessage));
		const result = await load(createUrlMock(null));

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: errorMessage
		});
	});

	it('should handle non-Error exceptions with an "Unknown error" message', async () => {
		(fetchRegistryDotrains as Mock).mockRejectedValue('Not an error object');
		const result = await load(createUrlMock(null));

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: 'Unknown error occurred'
		});
	});
});
