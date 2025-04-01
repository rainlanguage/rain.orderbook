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

	it('should load strategies from default registry URL when no registry param is provided', async () => {
		vi.mocked(fetchRegistryDotrains).mockResolvedValue(mockDotrains);
		vi.mocked(validateStrategies).mockResolvedValue(mockValidated);

		const result = await load({
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue(null)
				}
			}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

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

		const result = await load({
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue(customRegistry)
				}
			}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(fetchRegistryDotrains).toHaveBeenCalledWith(customRegistry);
		expect(validateStrategies).toHaveBeenCalledWith(mockDotrains);

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
		vi.mocked(fetchRegistryDotrains).mockRejectedValue(new Error(errorMessage));

		const result = await load({
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue(null)
				}
			}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(fetchRegistryDotrains).toHaveBeenCalledWith(REGISTRY_URL);
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

		vi.mocked(fetchRegistryDotrains).mockResolvedValue(mockDotrains);
		vi.mocked(validateStrategies).mockRejectedValue(new Error(errorMessage));

		const result = await load({
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue(null)
				}
			}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(fetchRegistryDotrains).toHaveBeenCalledWith(REGISTRY_URL);
		expect(validateStrategies).toHaveBeenCalledWith(mockDotrains);

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: errorMessage
		});
	});

	it('should handle non-Error exceptions with an "Unknown error" message', async () => {
		vi.mocked(fetchRegistryDotrains).mockRejectedValue('Not an error object');

		const result = await load({
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue(null)
				}
			}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} as any);

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: 'Unknown error occurred'
		});
	});
});
